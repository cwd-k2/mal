use std::collections::{HashMap, HashSet};

use crate::ast::Node;
use crate::diagnostic::Diagnostic;
use crate::resolve::ast::{self as resolved, FALSE_VALUE, TRUE_VALUE, TypeId, ValueId};
use crate::source::Span;

pub mod ast;
mod control;
mod expression;
mod float;
mod initializer;
mod integer;
mod interface;
mod lambda;
mod memory;
mod operator;
mod product;
mod types;

pub fn type_name(ty: &ast::Type) -> String {
    types::type_name(ty)
}

use self::ast::{AbruptExpression, Binding, BodyItem, Completion, Pattern, Program, TopItem, Type};
use self::interface::ExternalSignature;
use self::types::AliasDefinition;

pub fn check(program: &resolved::Program) -> Result<Program, Diagnostic> {
    Checker::new()
        .check_program(program)
        .map_err(|error| match error {
            CheckFailure::Diagnostic(diagnostic) => diagnostic,
            CheckFailure::Abrupt(abrupt) => Diagnostic::error("abrupt completion outside a lambda")
                .with_primary(
                    abrupt.span,
                    "this control expression has no return boundary",
                ),
        })
}

enum CheckFailure {
    Diagnostic(Diagnostic),
    Abrupt(Box<AbruptExpression>),
}

impl From<Diagnostic> for CheckFailure {
    fn from(value: Diagnostic) -> Self {
        Self::Diagnostic(value)
    }
}

type CheckResult<T> = Result<T, CheckFailure>;

struct Checker {
    aliases: HashMap<TypeId, AliasDefinition>,
    external_types: HashMap<TypeId, resolved::TypeBinding>,
    expanded_aliases: HashMap<TypeId, Type>,
    aggregate_alias_sources: HashMap<TypeId, Option<Vec<Node<resolved::TypeExpression>>>>,
    expanding: HashSet<TypeId>,
    values: HashMap<ValueId, Type>,
    external_values: HashSet<ValueId>,
    externals: HashMap<resolved::ExternalOperationId, ExternalSignature>,
    result_targets: HashMap<ValueId, ResultTarget>,
    used_result_targets: HashSet<ValueId>,
}

#[derive(Clone)]
struct ResultTarget {
    parameter: Type,
    result: Type,
    variant: Option<usize>,
    boundary: ValueId,
}

impl Checker {
    fn new() -> Self {
        let bool_type = Type::Sum(vec![Type::Unit, Type::Unit].into());
        Self {
            aliases: HashMap::new(),
            external_types: HashMap::new(),
            expanded_aliases: HashMap::new(),
            aggregate_alias_sources: HashMap::new(),
            expanding: HashSet::new(),
            values: HashMap::from([(FALSE_VALUE, bool_type.clone()), (TRUE_VALUE, bool_type)]),
            external_values: HashSet::new(),
            externals: HashMap::new(),
            result_targets: HashMap::new(),
            used_result_targets: HashSet::new(),
        }
    }

    fn check_program(mut self, program: &resolved::Program) -> CheckResult<Program> {
        self.collect_aliases(program);
        for definition in self.aliases.values().cloned().collect::<Vec<_>>() {
            self.expand_type_id(definition.binding.id, definition.binding.name.span)?;
        }
        self.collect_external_signatures(program)?;

        let mut items = Vec::with_capacity(program.items.len());
        for item in &program.items {
            let kind = match &item.kind {
                resolved::TopItem::TypeAlias { binding, value } => {
                    let ty = self.expand_type_id(binding.id, binding.name.span)?;
                    let target_alias = self.alias_name(value);
                    let element_aliases = match &ty {
                        Type::Product(_) | Type::Sum(_) => self.aggregate_aliases(value, &ty),
                        _ => Vec::new(),
                    };
                    TopItem::TypeAlias {
                        binding: binding.clone(),
                        ty,
                        target_alias,
                        element_aliases,
                    }
                }
                resolved::TopItem::ExternalType { binding } => TopItem::ExternalType {
                    binding: binding.clone(),
                },
                resolved::TopItem::ExternalOperation {
                    id,
                    binding,
                    lambda_id,
                    ..
                } => {
                    let signature = self
                        .externals
                        .get(id)
                        .expect("external signatures are collected before checking values");
                    TopItem::ExternalOperation {
                        id: *id,
                        binding: binding.clone(),
                        lambda_id: *lambda_id,
                        parameter: signature.parameter.clone(),
                        parameter_alias: signature.parameter_alias.clone(),
                        parameter_aliases: signature.parameter_aliases.clone(),
                        result: signature.result.clone(),
                        result_alias: signature.result_alias.clone(),
                    }
                }
                resolved::TopItem::Binding(binding) => {
                    let checked = self.check_binding(binding, item.span)?;
                    self.check_top_level_initializer(&binding.value)?;
                    TopItem::Binding(Box::new(checked))
                }
            };
            items.push(Node::new(kind, item.span));
        }
        Ok(Program {
            items,
            span: program.span,
        })
    }

    fn check_binding(&mut self, binding: &resolved::Binding, span: Span) -> CheckResult<Binding> {
        let annotation = binding
            .annotation
            .as_ref()
            .map(|ty| self.expand_type(ty))
            .transpose()?;
        if let (
            Some(annotation),
            resolved::Pattern::Binding(pattern_binding),
            resolved::Expression::Lambda(lambda),
        ) = (&annotation, &binding.pattern.kind, &binding.value.kind)
            && lambda.self_binding == Some(pattern_binding.id)
        {
            self.values.insert(pattern_binding.id, annotation.clone());
        }
        let value = match self.check_value_expression(&binding.value, annotation.as_ref()) {
            Ok(value) => value,
            Err(CheckFailure::Abrupt(abrupt)) => {
                return Err(
                    Diagnostic::error("binding initializer must produce a value")
                        .with_primary(abrupt.span, "this initializer completes abruptly")
                        .into(),
                );
            }
            Err(error) => return Err(error),
        };
        let pattern = self.check_pattern(&binding.pattern, &value.ty)?;
        Ok(Binding {
            pattern,
            annotation,
            value,
            span,
        })
    }

    fn check_pattern(
        &mut self,
        pattern: &Node<resolved::Pattern>,
        ty: &Type,
    ) -> CheckResult<Pattern> {
        match &pattern.kind {
            resolved::Pattern::Binding(binding) => {
                self.values.insert(binding.id, ty.clone());
                Ok(Pattern::Binding {
                    binding: binding.clone(),
                    ty: ty.clone(),
                })
            }
            resolved::Pattern::Wildcard => Ok(Pattern::Wildcard {
                ty: ty.clone(),
                span: pattern.span,
            }),
            resolved::Pattern::Product(elements) => {
                let Type::Product(element_types) = ty else {
                    return Err(
                        Diagnostic::error("product pattern requires a product value")
                            .with_primary(
                                pattern.span,
                                format!("this value has type `{}`", types::type_name(ty)),
                            )
                            .into(),
                    );
                };
                if elements.len() != element_types.len() {
                    return Err(Diagnostic::error("product pattern has the wrong arity")
                        .with_primary(
                            pattern.span,
                            format!(
                                "expected {} elements, found {}",
                                element_types.len(),
                                elements.len()
                            ),
                        )
                        .into());
                }
                Ok(Pattern::Product {
                    elements: elements
                        .iter()
                        .zip(element_types.iter())
                        .map(|(element, ty)| self.check_pattern(element, ty))
                        .collect::<Result<_, _>>()?,
                    ty: ty.clone(),
                    span: pattern.span,
                })
            }
        }
    }

    fn check_body_item(&mut self, item: &resolved::BodyItem) -> CheckResult<BodyItem> {
        match item {
            resolved::BodyItem::Binding(binding) => Ok(BodyItem::Binding(
                self.check_binding(&binding.kind, binding.span)?,
            )),
            resolved::BodyItem::Expression(expression) => Ok(BodyItem::Expression(
                self.check_value_expression(expression, None)?,
            )),
        }
    }

    fn check_body_items(
        &mut self,
        items: &[resolved::BodyItem],
        result_span: crate::source::Span,
    ) -> CheckResult<Vec<BodyItem>> {
        let mut checked = Vec::with_capacity(items.len());
        for (index, item) in items.iter().enumerate() {
            match self.check_body_item(item) {
                Ok(item) => checked.push(item),
                Err(CheckFailure::Abrupt(abrupt)) => {
                    let unreachable_span = items
                        .get(index + 1)
                        .map(|item| match item {
                            resolved::BodyItem::Binding(binding) => binding.span,
                            resolved::BodyItem::Expression(expression) => expression.span,
                        })
                        .unwrap_or(result_span);
                    return Err(Diagnostic::error("unreachable code after abrupt completion")
                        .with_primary(
                            unreachable_span,
                            format!(
                                "this expression cannot be reached after control leaves at byte {}",
                                abrupt.span.start()
                            ),
                        )
                        .into());
                }
                Err(error) => return Err(error),
            }
        }
        Ok(checked)
    }

    fn value_type(&self, reference: &resolved::ValueReference) -> Result<Type, Diagnostic> {
        if self.result_targets.contains_key(&reference.id) {
            return Err(Diagnostic::error("result binder is not a value")
                .with_primary(reference.name.span, "call this binder in callee position"));
        }
        self.values.get(&reference.id).cloned().ok_or_else(|| {
            Diagnostic::error(format!(
                "value `{}` has no inferred type",
                reference.name.text
            ))
            .with_primary(reference.name.span, "its binding is not available here")
        })
    }

    fn check_completion(
        &mut self,
        expression: &Node<resolved::Expression>,
        expected: Option<&Type>,
    ) -> Result<Completion, Diagnostic> {
        match self.check_expression(expression, expected) {
            Ok(value) => Ok(Completion::Value(value)),
            Err(CheckFailure::Abrupt(abrupt)) => Ok(Completion::Abrupt(*abrupt)),
            Err(CheckFailure::Diagnostic(diagnostic)) => Err(diagnostic),
        }
    }

    fn check_value_expression(
        &mut self,
        expression: &Node<resolved::Expression>,
        expected: Option<&Type>,
    ) -> CheckResult<self::ast::Expression> {
        self.check_expression(expression, expected)
    }
}
