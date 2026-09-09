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
mod memory;
mod operator;
mod product;
mod types;

pub fn type_name(ty: &ast::Type) -> String {
    types::type_name(ty)
}

use self::ast::{Binding, BodyItem, Pattern, Program, TopItem, Type};
use self::interface::ExternalSignature;
use self::types::AliasDefinition;

pub fn check(program: &resolved::Program) -> Result<Program, Diagnostic> {
    Checker::new().check_program(program)
}

struct Checker {
    aliases: HashMap<TypeId, AliasDefinition>,
    external_types: HashMap<TypeId, resolved::TypeBinding>,
    expanded_aliases: HashMap<TypeId, Type>,
    expanding: Vec<TypeId>,
    values: HashMap<ValueId, Type>,
    external_values: HashSet<ValueId>,
    externals: HashMap<resolved::ExternalOperationId, ExternalSignature>,
}

impl Checker {
    fn new() -> Self {
        let bool_type = Type::Sum(vec![Type::Unit, Type::Unit]);
        Self {
            aliases: HashMap::new(),
            external_types: HashMap::new(),
            expanded_aliases: HashMap::new(),
            expanding: Vec::new(),
            values: HashMap::from([(FALSE_VALUE, bool_type.clone()), (TRUE_VALUE, bool_type)]),
            external_values: HashSet::new(),
            externals: HashMap::new(),
        }
    }

    fn check_program(mut self, program: &resolved::Program) -> Result<Program, Diagnostic> {
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

    fn check_binding(
        &mut self,
        binding: &resolved::Binding,
        span: Span,
    ) -> Result<Binding, Diagnostic> {
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
        let value = self.check_expression(&binding.value, annotation.as_ref())?;
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
    ) -> Result<Pattern, Diagnostic> {
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
                        Diagnostic::error("product pattern requires a product value").with_primary(
                            pattern.span,
                            format!("this value has type `{}`", types::type_name(ty)),
                        ),
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
                        ));
                }
                Ok(Pattern::Product {
                    elements: elements
                        .iter()
                        .zip(element_types)
                        .map(|(element, ty)| self.check_pattern(element, ty))
                        .collect::<Result<_, _>>()?,
                    ty: ty.clone(),
                    span: pattern.span,
                })
            }
        }
    }

    fn check_body_item(&mut self, item: &resolved::BodyItem) -> Result<BodyItem, Diagnostic> {
        match item {
            resolved::BodyItem::Binding(binding) => Ok(BodyItem::Binding(
                self.check_binding(&binding.kind, binding.span)?,
            )),
            resolved::BodyItem::Expression(expression) => Ok(BodyItem::Expression(
                self.check_expression(expression, None)?,
            )),
        }
    }

    fn value_type(&self, reference: &resolved::ValueReference) -> Result<Type, Diagnostic> {
        self.values.get(&reference.id).cloned().ok_or_else(|| {
            Diagnostic::error(format!(
                "value `{}` has no inferred type",
                reference.name.text
            ))
            .with_primary(reference.name.span, "its binding is not available here")
        })
    }
}
