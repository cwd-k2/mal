use std::collections::HashMap;

use crate::ast::{Node, UnaryOperator};
use crate::diagnostic::Diagnostic;
use crate::resolve::ast::{
    self as resolved, BOOL_TYPE, FALSE_VALUE, INT8_TYPE, INT16_TYPE, INT32_TYPE, INT64_TYPE,
    STRING_TYPE, TRUE_VALUE, TypeId, UINT8_TYPE, UINT16_TYPE, UINT32_TYPE, UINT64_TYPE, UNIT_TYPE,
    ValueId,
};
use crate::source::Span;

pub mod ast;
mod control;
mod expression;
mod integer;
mod product;
mod string;
mod types;

use self::ast::{Binding, BodyItem, Pattern, Program, TopItem, Type};

pub fn check(program: &resolved::Program) -> Result<Program, Diagnostic> {
    Checker::new().check_program(program)
}

#[derive(Clone)]
struct AliasDefinition {
    binding: resolved::TypeBinding,
    value: Node<resolved::TypeExpression>,
}

#[derive(Clone)]
struct ExternalSignature {
    parameter: Type,
    result: Type,
}

struct Checker {
    aliases: HashMap<TypeId, AliasDefinition>,
    external_types: HashMap<TypeId, resolved::TypeBinding>,
    expanded_aliases: HashMap<TypeId, Type>,
    expanding: Vec<TypeId>,
    values: HashMap<ValueId, Type>,
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
            externals: HashMap::new(),
        }
    }

    fn check_program(mut self, program: &resolved::Program) -> Result<Program, Diagnostic> {
        self.collect_aliases(program)?;
        for definition in self.aliases.values().cloned().collect::<Vec<_>>() {
            self.expand_type_id(definition.binding.id, definition.binding.name.span)?;
        }
        self.collect_external_signatures(program)?;

        let mut items = Vec::with_capacity(program.items.len());
        for item in &program.items {
            let kind = match &item.kind {
                resolved::TopItem::TypeAlias { binding, .. } => TopItem::TypeAlias {
                    binding: binding.clone(),
                    ty: self.expand_type_id(binding.id, binding.name.span)?,
                },
                resolved::TopItem::ExternalType { binding } => TopItem::ExternalType {
                    binding: binding.clone(),
                },
                resolved::TopItem::ExternalOperation { id, name, .. } => {
                    let signature = self
                        .externals
                        .get(id)
                        .expect("external signatures are collected before checking values");
                    TopItem::ExternalOperation {
                        id: *id,
                        name: name.clone(),
                        parameter: signature.parameter.clone(),
                        result: signature.result.clone(),
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

    fn collect_aliases(&mut self, program: &resolved::Program) -> Result<(), Diagnostic> {
        for item in &program.items {
            match &item.kind {
                resolved::TopItem::TypeAlias { binding, value } => {
                    self.aliases.insert(
                        binding.id,
                        AliasDefinition {
                            binding: binding.clone(),
                            value: value.clone(),
                        },
                    );
                }
                resolved::TopItem::ExternalType { binding } => {
                    self.external_types.insert(binding.id, binding.clone());
                }
                _ => {}
            }
        }
        Ok(())
    }

    fn collect_external_signatures(
        &mut self,
        program: &resolved::Program,
    ) -> Result<(), Diagnostic> {
        for item in &program.items {
            let resolved::TopItem::ExternalOperation { id, name, ty } = &item.kind else {
                continue;
            };
            let signature = self.expand_type(ty)?;
            let Type::Function { parameter, result } = signature else {
                return Err(Diagnostic::error(format!(
                    "external operation `{}` must have a function type",
                    name.text
                ))
                .with_primary(ty.span, "expected `parameter -> result`"));
            };
            if contains_function(&parameter) || contains_function(&result) {
                return Err(Diagnostic::error(format!(
                    "external operation `{}` uses a function value",
                    name.text
                ))
                .with_primary(ty.span, "function types cannot cross the extern boundary"));
            }
            self.externals.insert(
                *id,
                ExternalSignature {
                    parameter: *parameter,
                    result: *result,
                },
            );
        }
        Ok(())
    }

    fn expand_type(&mut self, ty: &Node<resolved::TypeExpression>) -> Result<Type, Diagnostic> {
        match &ty.kind {
            resolved::TypeExpression::Named(reference) => {
                self.expand_type_id(reference.id, reference.name.span)
            }
            resolved::TypeExpression::Unit => Ok(Type::Unit),
            resolved::TypeExpression::Parenthesized(inner) => self.expand_type(inner),
            resolved::TypeExpression::Product(elements) => Ok(Type::Product(
                elements
                    .iter()
                    .map(|element| self.expand_type(element))
                    .collect::<Result<_, _>>()?,
            )),
            resolved::TypeExpression::Sum(members) => Ok(Type::Sum(
                members
                    .iter()
                    .map(|member| self.expand_type(member))
                    .collect::<Result<_, _>>()?,
            )),
            resolved::TypeExpression::Function { parameter, result } => Ok(Type::Function {
                parameter: Box::new(self.expand_type(parameter)?),
                result: Box::new(self.expand_type(result)?),
            }),
        }
    }

    fn expand_type_id(&mut self, id: TypeId, use_span: Span) -> Result<Type, Diagnostic> {
        match id {
            UNIT_TYPE => return Ok(Type::Unit),
            INT8_TYPE => return Ok(Type::Int8),
            INT16_TYPE => return Ok(Type::Int16),
            INT32_TYPE => return Ok(Type::Int32),
            INT64_TYPE => return Ok(Type::Int64),
            UINT8_TYPE => return Ok(Type::UInt8),
            UINT16_TYPE => return Ok(Type::UInt16),
            UINT32_TYPE => return Ok(Type::UInt32),
            UINT64_TYPE => return Ok(Type::UInt64),
            BOOL_TYPE => return Ok(Type::Sum(vec![Type::Unit, Type::Unit])),
            STRING_TYPE => return Ok(Type::String),
            _ => {}
        }
        if let Some(binding) = self.external_types.get(&id) {
            return Ok(Type::External {
                id,
                name: binding.name.text.clone(),
            });
        }
        if let Some(expanded) = self.expanded_aliases.get(&id) {
            return Ok(expanded.clone());
        }
        if self.expanding.contains(&id) {
            return Err(Diagnostic::error("recursive type alias")
                .with_primary(use_span, "this reference forms an alias cycle"));
        }
        let definition = self
            .aliases
            .get(&id)
            .cloned()
            .expect("resolved type IDs must have a definition");
        self.expanding.push(id);
        let expanded = self.expand_type(&definition.value)?;
        self.expanding.pop();
        self.expanded_aliases.insert(id, expanded.clone());
        Ok(expanded)
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
            && lambda.self_binding.as_ref().map(|binding| binding.id) == Some(pattern_binding.id)
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

    fn external_signature(&self, id: resolved::ExternalOperationId) -> ExternalSignature {
        self.externals
            .get(&id)
            .cloned()
            .expect("resolved external calls have a collected signature")
    }

    fn check_top_level_initializer(
        &self,
        expression: &Node<resolved::Expression>,
    ) -> Result<(), Diagnostic> {
        let allowed = match &expression.kind {
            resolved::Expression::Integer(_)
            | resolved::Expression::Byte(_)
            | resolved::Expression::String(_)
            | resolved::Expression::Unit => true,
            resolved::Expression::Reference(reference) => {
                matches!(reference.id, FALSE_VALUE | TRUE_VALUE)
            }
            resolved::Expression::Parenthesized(inner) => {
                self.check_top_level_initializer(inner).is_ok()
            }
            resolved::Expression::Product(elements) => elements
                .iter()
                .all(|element| self.check_top_level_initializer(element).is_ok()),
            resolved::Expression::SumInjection { value, .. } => {
                self.check_top_level_initializer(value).is_ok()
            }
            resolved::Expression::Conversion { value, .. } => {
                self.check_top_level_initializer(value).is_ok()
            }
            resolved::Expression::Lambda(_) => true,
            resolved::Expression::Unary {
                operator, operand, ..
            } => {
                operator.kind == UnaryOperator::Negate
                    && matches!(operand.kind, resolved::Expression::Integer(_))
            }
            _ => false,
        };
        if allowed {
            Ok(())
        } else {
            Err(
                Diagnostic::error("unsupported top-level initializer").with_primary(
                    expression.span,
                    "top-level values must be closed literals, sums, or lambdas",
                ),
            )
        }
    }
}

fn contains_function(ty: &Type) -> bool {
    match ty {
        Type::Function { .. } => true,
        Type::Product(elements) => elements.iter().any(contains_function),
        Type::Sum(members) => members.iter().any(contains_function),
        Type::External { .. }
        | Type::Unit
        | Type::Int8
        | Type::Int16
        | Type::Int32
        | Type::Int64
        | Type::UInt8
        | Type::UInt16
        | Type::UInt32
        | Type::UInt64 => false,
        Type::String => false,
    }
}
