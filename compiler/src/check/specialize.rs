use std::collections::HashMap;

use crate::ast::Node;
use crate::diagnostic::Diagnostic;
use crate::resolve::ast::{LambdaId, ValueBinding, ValueId, ValueReference};

use super::GenericDefinition;
use super::ast::*;
use super::specialization_identity::next_identities;
use super::types::substitute_type;

const LIMIT: usize = 65_536;

pub(super) fn specialize(
    mut program: Program,
    definitions: Vec<GenericDefinition>,
) -> Result<Program, Diagnostic> {
    if definitions.is_empty() {
        return Ok(program);
    }
    let identities = next_identities(&program, &definitions).ok_or_else(|| {
        Diagnostic::error("compiler identity space exhausted").with_primary(
            program.span,
            "cannot allocate identities for generic specializations",
        )
    })?;
    let definitions = definitions
        .into_iter()
        .map(|definition| (definition.binding.id, definition))
        .collect();
    let mut specializer = Specializer {
        definitions,
        instances: Vec::new(),
        pending: Vec::new(),
        next_value: identities.value,
        next_lambda: identities.lambda,
    };
    for item in &mut program.items {
        if let TopItem::Binding(binding) = &mut item.kind {
            specializer.expression(&mut binding.value, &HashMap::new(), None)?;
        }
    }
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
        program.items.push(Node::new(
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
    Ok(program)
}

struct Specializer {
    definitions: HashMap<ValueId, GenericDefinition>,
    instances: Vec<(ValueId, Vec<Type>, ValueBinding)>,
    pending: Vec<(ValueId, Vec<Type>, ValueBinding)>,
    next_value: u32,
    next_lambda: u32,
}

impl Specializer {
    fn request(
        &mut self,
        reference: &ValueReference,
        arguments: &[Type],
    ) -> Result<ValueReference, Diagnostic> {
        if let Some((_, _, binding)) = self
            .instances
            .iter()
            .find(|(id, existing, _)| *id == reference.id && existing == arguments)
        {
            return Ok(ValueReference {
                id: binding.id,
                name: reference.name.clone(),
            });
        }
        if self.instances.len() == LIMIT {
            return Err(
                Diagnostic::error("specialization limit exceeded").with_primary(
                    reference.name.span,
                    format!("one program may contain at most {LIMIT} specialization nodes"),
                ),
            );
        }
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
        self.instances.push(entry.clone());
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
                lambda.id = LambdaId(self.next_lambda);
                self.next_lambda = self.next_lambda.checked_add(1).ok_or_else(|| {
                    Diagnostic::error("compiler identity space exhausted").with_primary(
                        expression.span,
                        "cannot allocate a specialized lambda identity",
                    )
                })?;
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
            ExpressionKind::Binary { left, right, .. } => {
                self.expression(left, substitutions, self_instance)?;
                self.expression(right, substitutions, self_instance)?;
            }
            ExpressionKind::StorageSize(ty) => *ty = substitute_type(ty, substitutions),
            ExpressionKind::Integer(_)
            | ExpressionKind::Float(_)
            | ExpressionKind::Symbol(_)
            | ExpressionKind::Unit => {}
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
