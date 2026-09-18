use std::collections::{HashMap, HashSet};

use crate::ast::Node;
use crate::diagnostic::Diagnostic;
use crate::resolve::ast::{LambdaId, ValueBinding, ValueId, ValueReference};

use super::ast::*;
use super::specialization_identity::next_identities;
use super::type_fingerprint::TypeFingerprints;
use super::types::substitute_type;

mod admission;

use admission::{admit_specialization, collect_pattern_bindings, entry_binding};

pub(super) fn specialize(program: Program) -> Result<MonomorphicProgram, Diagnostic> {
    let identities = next_identities(&program).ok_or_else(|| {
        Diagnostic::error("compiler identity space exhausted").with_primary(
            program.span,
            "cannot allocate identities for generic specializations",
        )
    })?;
    let program_span = program.span;
    let mut definitions = HashMap::new();
    let mut bindings = Vec::new();
    let mut binding_items = HashMap::new();
    let mut items = Vec::new();
    for item in program.items {
        match item.kind {
            TopItem::GenericBinding(definition) => {
                definitions.insert(definition.binding.id, *definition);
            }
            TopItem::Binding(binding) => {
                let index = bindings.len();
                collect_pattern_bindings(&binding.pattern, index, &mut binding_items);
                bindings.push(Some(Node::new(TopItem::Binding(binding), item.span)));
            }
            _ => items.push(item),
        }
    }
    let mut specializer = Specializer {
        definitions,
        bindings,
        binding_items,
        selected_bindings: HashSet::new(),
        reachable: HashMap::new(),
        specializations: Vec::new(),
        instances: Vec::new(),
        instance_buckets: HashMap::new(),
        fingerprints: TypeFingerprints::default(),
        pending: Vec::new(),
        next_value: identities.value,
        next_lambda: identities.lambda,
    };
    let main = specializer.main_binding(program_span)?;
    specializer.request_binding(main)?;
    let mut cursor = 0;
    while cursor < specializer.pending.len() {
        let (generic, arguments, binding) = specializer.pending[cursor].clone();
        cursor += 1;
        let definition = specializer
            .definitions
            .get(&generic)
            .cloned()
            .expect("checked generic reference has a definition");
        let substitutions = definition
            .parameters
            .iter()
            .map(|parameter| parameter.id)
            .zip(arguments)
            .collect::<HashMap<_, _>>();
        let mut value = definition.value;
        specializer.expression(&mut value, &substitutions, Some((generic, binding.id)))?;
        let ty = substitute_type(&definition.ty, &substitutions);
        specializer.specializations.push(Node::new(
            TopItem::Binding(Box::new(Binding {
                pattern: Pattern::Binding {
                    binding,
                    ty: ty.clone(),
                },
                annotation: Some(ty),
                value,
                span: definition.span,
            })),
            definition.span,
        ));
    }
    for index in 0..specializer.bindings.len() {
        if let Some(binding) = specializer.reachable.remove(&index) {
            items.push(binding);
        }
    }
    items.extend(specializer.specializations);
    Ok(MonomorphicProgram::new(Program {
        items,
        span: program_span,
    }))
}

struct Specializer {
    definitions: HashMap<ValueId, GenericBinding>,
    bindings: Vec<Option<Node<TopItem>>>,
    binding_items: HashMap<ValueId, usize>,
    selected_bindings: HashSet<usize>,
    reachable: HashMap<usize, Node<TopItem>>,
    specializations: Vec<Node<TopItem>>,
    instances: Vec<(ValueId, Vec<Type>, ValueBinding)>,
    instance_buckets: HashMap<(ValueId, u64), Vec<usize>>,
    fingerprints: TypeFingerprints,
    pending: Vec<(ValueId, Vec<Type>, ValueBinding)>,
    next_value: u32,
    next_lambda: u32,
}

impl Specializer {
    fn main_binding(&self, program_span: crate::source::Span) -> Result<ValueId, Diagnostic> {
        entry_binding(self.bindings.iter().flatten(), program_span)
    }

    fn request_binding(&mut self, id: ValueId) -> Result<(), Diagnostic> {
        let Some(&index) = self.binding_items.get(&id) else {
            return Ok(());
        };
        if !self.selected_bindings.insert(index) {
            return Ok(());
        }
        let mut item = self.bindings[index]
            .take()
            .expect("a selected binding is taken exactly once");
        let TopItem::Binding(binding) = &mut item.kind else {
            unreachable!("the binding table contains only bindings")
        };
        self.expression(&mut binding.value, &HashMap::new(), None)?;
        self.reachable.insert(index, item);
        Ok(())
    }

    fn request(
        &mut self,
        reference: &ValueReference,
        arguments: &[Type],
    ) -> Result<ValueReference, Diagnostic> {
        let fingerprint = self.fingerprints.arguments(arguments);
        if let Some((_, _, binding)) = self
            .instance_buckets
            .get(&(reference.id, fingerprint))
            .into_iter()
            .flatten()
            .filter_map(|&index| self.instances.get(index))
            .find(|(id, existing, _)| *id == reference.id && existing == arguments)
        {
            return Ok(ValueReference {
                id: binding.id,
                name: reference.name.clone(),
            });
        }
        admit_specialization(self.instances.len(), reference.name.span)?;
        let definition = self
            .definitions
            .get(&reference.id)
            .expect("checked generic reference has a definition");
        let binding = ValueBinding {
            id: ValueId(self.next_value),
            name: definition.binding.name.clone(),
            owner: definition.binding.owner,
        };
        self.next_value = self.next_value.checked_add(1).ok_or_else(|| {
            Diagnostic::error("compiler identity space exhausted").with_primary(
                reference.name.span,
                "cannot allocate a specialization identity",
            )
        })?;
        let entry = (reference.id, arguments.to_vec(), binding.clone());
        let index = self.instances.len();
        self.instances.push(entry.clone());
        self.instance_buckets
            .entry((reference.id, fingerprint))
            .or_default()
            .push(index);
        self.pending.push(entry);
        Ok(ValueReference {
            id: binding.id,
            name: reference.name.clone(),
        })
    }

    fn expression(
        &mut self,
        expression: &mut Expression,
        substitutions: &HashMap<crate::resolve::ast::TypeId, Type>,
        self_instance: Option<(ValueId, ValueId)>,
    ) -> Result<(), Diagnostic> {
        if matches!(&expression.kind, ExpressionKind::Binary { .. }) {
            return self.binary_expression(expression, substitutions, self_instance);
        }
        expression.ty = substitute_type(&expression.ty, substitutions);
        match &mut expression.kind {
            ExpressionKind::GenericReference {
                reference,
                arguments,
            } => {
                for argument in arguments.iter_mut() {
                    *argument = substitute_type(argument, substitutions);
                }
                let reference = self.request(reference, arguments)?;
                expression.kind = ExpressionKind::Reference(reference);
            }
            ExpressionKind::Reference(reference) => {
                if let Some((generic, specialized)) = self_instance
                    && reference.id == generic
                {
                    reference.id = specialized;
                } else {
                    self.request_binding(reference.id)?;
                }
            }
            ExpressionKind::Product(values) => {
                for value in values {
                    self.expression(value, substitutions, self_instance)?;
                }
            }
            ExpressionKind::Parenthesized(value)
            | ExpressionKind::SymbolLength { value }
            | ExpressionKind::NumericConversion { value }
            | ExpressionKind::SumInjection { value, .. } => {
                self.expression(value, substitutions, self_instance)?
            }
            ExpressionKind::Block(block) => self.block(block, substitutions, self_instance)?,
            ExpressionKind::ResultBlock {
                result_binders,
                body,
                ..
            } => {
                for binder in result_binders {
                    binder.parameter_type = substitute_type(&binder.parameter_type, substitutions);
                }
                self.block(body, substitutions, self_instance)?;
            }
            ExpressionKind::Lambda(lambda) => {
                if self_instance.is_some() {
                    lambda.id = LambdaId(self.next_lambda);
                    self.next_lambda = self.next_lambda.checked_add(1).ok_or_else(|| {
                        Diagnostic::error("compiler identity space exhausted").with_primary(
                            expression.span,
                            "cannot allocate a specialized lambda identity",
                        )
                    })?;
                }
                lambda.parameter_type = substitute_type(&lambda.parameter_type, substitutions);
                lambda.result_type = substitute_type(&lambda.result_type, substitutions);
                if let Some(parameter) = &mut lambda.parameter {
                    pattern(parameter, substitutions);
                }
                for capture in &mut lambda.captures {
                    capture.ty = substitute_type(&capture.ty, substitutions);
                }
                if let Some((generic, specialized)) = self_instance
                    && lambda.self_binding == Some(generic)
                {
                    lambda.self_binding = Some(specialized);
                }
                self.body(&mut lambda.body, substitutions, self_instance)?;
            }
            ExpressionKind::Call { callee, argument } => {
                self.expression(callee, substitutions, self_instance)?;
                self.expression(argument, substitutions, self_instance)?;
            }
            ExpressionKind::SymbolAt { argument } | ExpressionKind::Memory { argument, .. } => {
                self.expression(argument, substitutions, self_instance)?
            }
            ExpressionKind::SumElimination {
                scrutinee,
                continuations,
            } => {
                self.expression(scrutinee, substitutions, self_instance)?;
                for continuation in continuations {
                    self.expression(continuation, substitutions, self_instance)?;
                }
            }
            ExpressionKind::If {
                condition,
                then_branch,
                else_branch,
            } => {
                self.expression(condition, substitutions, self_instance)?;
                self.block(then_branch, substitutions, self_instance)?;
                self.block(else_branch, substitutions, self_instance)?;
            }
            ExpressionKind::Unary { operand, .. } => {
                self.expression(operand, substitutions, self_instance)?
            }
            ExpressionKind::Binary { .. } => {
                unreachable!("binary expressions are walked iteratively")
            }
            ExpressionKind::StorageSize(ty) => *ty = substitute_type(ty, substitutions),
            ExpressionKind::Integer(_)
            | ExpressionKind::Float(_)
            | ExpressionKind::Symbol(_)
            | ExpressionKind::Unit => {}
        }
        Ok(())
    }

    fn binary_expression(
        &mut self,
        expression: &mut Expression,
        substitutions: &HashMap<crate::resolve::ast::TypeId, Type>,
        self_instance: Option<(ValueId, ValueId)>,
    ) -> Result<(), Diagnostic> {
        let mut pending = vec![expression];
        while let Some(expression) = pending.pop() {
            expression.ty = substitute_type(&expression.ty, substitutions);
            if matches!(&expression.kind, ExpressionKind::Binary { .. }) {
                let ExpressionKind::Binary { left, right, .. } = &mut expression.kind else {
                    unreachable!()
                };
                pending.push(right);
                pending.push(left);
                continue;
            }
            self.expression(expression, substitutions, self_instance)?;
        }
        Ok(())
    }

    fn block(
        &mut self,
        block: &mut ExpressionBlock,
        substitutions: &HashMap<crate::resolve::ast::TypeId, Type>,
        self_instance: Option<(ValueId, ValueId)>,
    ) -> Result<(), Diagnostic> {
        for item in &mut block.items {
            match item {
                BodyItem::Binding(binding) => {
                    pattern(&mut binding.pattern, substitutions);
                    self.expression(&mut binding.value, substitutions, self_instance)?;
                }
                BodyItem::Expression(value) => {
                    self.expression(value, substitutions, self_instance)?
                }
            }
        }
        completion(&mut block.result, substitutions);
        match block.result.as_mut() {
            Completion::Value(value) => self.expression(value, substitutions, self_instance)?,
            Completion::Abrupt(abrupt) => self.abrupt(abrupt, substitutions, self_instance)?,
        }
        Ok(())
    }

    fn body(
        &mut self,
        body: &mut LambdaBody,
        substitutions: &HashMap<crate::resolve::ast::TypeId, Type>,
        self_instance: Option<(ValueId, ValueId)>,
    ) -> Result<(), Diagnostic> {
        let mut block = ExpressionBlock {
            items: std::mem::take(&mut body.items),
            result: std::mem::replace(
                &mut body.result,
                Box::new(Completion::Value(Expression {
                    kind: ExpressionKind::Unit,
                    ty: Type::Unit,
                    span: body.span,
                })),
            ),
            span: body.span,
        };
        self.block(&mut block, substitutions, self_instance)?;
        body.items = block.items;
        body.result = block.result;
        Ok(())
    }

    fn abrupt(
        &mut self,
        abrupt: &mut AbruptExpression,
        substitutions: &HashMap<crate::resolve::ast::TypeId, Type>,
        self_instance: Option<(ValueId, ValueId)>,
    ) -> Result<(), Diagnostic> {
        for value in &mut abrupt.preceding {
            self.expression(value, substitutions, self_instance)?;
        }
        match &mut abrupt.kind {
            AbruptExpressionKind::ResultTransfer { value, .. }
            | AbruptExpressionKind::EmptyElimination { scrutinee: value } => {
                self.expression(value, substitutions, self_instance)?
            }
            AbruptExpressionKind::If {
                condition,
                then_branch,
                else_branch,
            } => {
                self.expression(condition, substitutions, self_instance)?;
                self.block(then_branch, substitutions, self_instance)?;
                self.block(else_branch, substitutions, self_instance)?;
            }
            AbruptExpressionKind::Block(block) => {
                self.block(block, substitutions, self_instance)?
            }
        }
        Ok(())
    }
}

fn pattern(value: &mut Pattern, substitutions: &HashMap<crate::resolve::ast::TypeId, Type>) {
    match value {
        Pattern::Binding { ty, .. } | Pattern::Wildcard { ty, .. } => {
            *ty = substitute_type(ty, substitutions)
        }
        Pattern::Product { elements, ty, .. } => {
            *ty = substitute_type(ty, substitutions);
            for element in elements {
                pattern(element, substitutions);
            }
        }
    }
}

fn completion(
    completion: &mut Completion,
    substitutions: &HashMap<crate::resolve::ast::TypeId, Type>,
) {
    if let Completion::Value(value) = completion {
        value.ty = substitute_type(&value.ty, substitutions);
    }
}
