use crate::ast::{BinaryOperator, UnaryOperator};
use crate::check::ast as checked;
use crate::resolve::ast::{FALSE_VALUE, TRUE_VALUE};
use crate::source::Span;

pub mod ast;
mod bool;
mod pattern;
mod primitive;

use self::bool::bool_type;
use self::primitive::lower_binary_primitive;

use self::ast::{
    Binding, Capture, CaseArm, Expression, ExpressionKind, ExternalOperation, ExternalType, Lambda,
    Parameter, Pattern, Program, ProgramInterface, TopLevelBinding, TypeAlias, UnaryPrimitive,
    ValueId,
};

pub fn lower(program: &checked::Program) -> Program {
    Lowerer::new().lower_program(program)
}

struct Lowerer {
    next_temporary: u32,
}

impl Lowerer {
    fn new() -> Self {
        Self { next_temporary: 0 }
    }

    fn lower_program(&mut self, program: &checked::Program) -> Program {
        let mut externals = Vec::new();
        let mut external_types = Vec::new();
        let mut type_aliases = Vec::new();
        let mut bindings = Vec::new();
        for item in &program.items {
            match &item.kind {
                checked::TopItem::TypeAlias { binding, ty } => type_aliases.push(TypeAlias {
                    name: binding.name.text.clone(),
                    ty: ty.clone(),
                }),
                checked::TopItem::ExternalType { binding } => external_types.push(ExternalType {
                    name: binding.name.text.clone(),
                }),
                checked::TopItem::ExternalOperation {
                    id,
                    name,
                    parameter,
                    parameter_aliases,
                    result,
                    result_alias,
                    ..
                } => externals.push(ExternalOperation {
                    id: *id,
                    name: name.text.clone(),
                    parameter: parameter.clone(),
                    parameter_aliases: parameter_aliases.clone(),
                    result: result.clone(),
                    result_alias: result_alias.clone(),
                    span: item.span,
                }),
                checked::TopItem::Binding(binding) => {
                    bindings.push(self.lower_top_level_binding(binding));
                }
            }
        }
        Program {
            interface: ProgramInterface {
                type_aliases,
                external_types,
                externals,
            },
            bindings,
            span: program.span,
        }
    }

    fn lower_top_level_binding(&mut self, binding: &checked::Binding) -> TopLevelBinding {
        let pattern = self.lower_top_level_pattern(&binding.pattern);
        TopLevelBinding {
            pattern,
            value: self.lower_expression(&binding.value),
            span: binding.span,
        }
    }

    fn lower_binding(&mut self, binding: &checked::Binding) -> Binding {
        Binding {
            pattern: self.lower_pattern(&binding.pattern),
            value: self.lower_expression(&binding.value),
            span: binding.span,
        }
    }

    fn lower_expression(&mut self, expression: &checked::Expression) -> Expression {
        let kind = match &expression.kind {
            checked::ExpressionKind::Reference(reference) => match reference.id {
                FALSE_VALUE => return self.bool_value(false, expression.span),
                TRUE_VALUE => return self.bool_value(true, expression.span),
                id => ExpressionKind::Reference(ValueId::Source(id)),
            },
            checked::ExpressionKind::Integer(value) => ExpressionKind::Integer(*value),
            checked::ExpressionKind::Float(bits) => ExpressionKind::Float(*bits),
            checked::ExpressionKind::Engram(value) => ExpressionKind::Engram(value.clone()),
            checked::ExpressionKind::StorageSize(ty) => ExpressionKind::StorageSize(ty.clone()),
            checked::ExpressionKind::Unit => ExpressionKind::Unit,
            checked::ExpressionKind::Product(elements) => ExpressionKind::Product(
                elements
                    .iter()
                    .map(|element| self.lower_expression(element))
                    .collect(),
            ),
            checked::ExpressionKind::Parenthesized(inner) => return self.lower_expression(inner),
            checked::ExpressionKind::Lambda(lambda) => {
                ExpressionKind::Lambda(self.lower_lambda(lambda))
            }
            checked::ExpressionKind::Call { callee, argument } => ExpressionKind::Call {
                callee: Box::new(self.lower_expression(callee)),
                argument: Box::new(self.lower_expression(argument)),
            },
            checked::ExpressionKind::EngramLength { value } => ExpressionKind::EngramLength {
                value: Box::new(self.lower_expression(value)),
            },
            checked::ExpressionKind::EngramAt { argument } => ExpressionKind::EngramAt {
                argument: Box::new(self.lower_expression(argument)),
            },
            checked::ExpressionKind::MemoryFunction { primitive, .. } => {
                ExpressionKind::MemoryFunction {
                    primitive: *primitive,
                }
            }
            checked::ExpressionKind::Memory {
                primitive,
                argument,
            } => ExpressionKind::Memory {
                primitive: *primitive,
                argument: Box::new(self.lower_expression(argument)),
            },
            checked::ExpressionKind::ExternalCall { id, argument, .. } => {
                ExpressionKind::ExternalCall {
                    id: *id,
                    argument: Box::new(self.lower_expression(argument)),
                }
            }
            checked::ExpressionKind::NumericConversion { value } => {
                ExpressionKind::NumericConversion {
                    value: Box::new(self.lower_expression(value)),
                }
            }
            checked::ExpressionKind::SumInjection { index, value } => {
                ExpressionKind::SumInjection {
                    index: *index,
                    value: Box::new(self.lower_expression(value)),
                }
            }
            checked::ExpressionKind::If {
                condition,
                then_branch,
                else_branch,
            } => {
                return self.lower_if(
                    condition,
                    then_branch,
                    else_branch,
                    &expression.ty,
                    expression.span,
                );
            }
            checked::ExpressionKind::Case { scrutinee, arms } => ExpressionKind::Case {
                scrutinee: Box::new(self.lower_expression(scrutinee)),
                arms: arms.iter().map(|arm| self.lower_case_arm(arm)).collect(),
            },
            checked::ExpressionKind::Unary { operator, operand } => {
                if operator.kind == UnaryOperator::LogicalNot {
                    return self.lower_logical_not(operand, expression.span);
                }
                ExpressionKind::PrimitiveUnary {
                    operator: match operator.kind {
                        UnaryOperator::Negate => UnaryPrimitive::Negate,
                        UnaryOperator::BitwiseNot => UnaryPrimitive::BitwiseNot,
                        UnaryOperator::LogicalNot | UnaryOperator::EngramLength => {
                            unreachable!("type checking rejects non-numeric core primitives")
                        }
                    },
                    operand: Box::new(self.lower_expression(operand)),
                }
            }
            checked::ExpressionKind::Binary {
                operator,
                left,
                right,
            } => {
                if operator.kind == BinaryOperator::LogicalAnd
                    || operator.kind == BinaryOperator::LogicalOr
                {
                    return self.lower_short_circuit(operator.kind, left, right, expression.span);
                }
                if operator.kind == BinaryOperator::EngramAt {
                    unreachable!("Engram access is lowered before generic binary operators");
                }
                if matches!(
                    operator.kind,
                    BinaryOperator::Equal | BinaryOperator::NotEqual
                ) && left.ty == bool_type()
                {
                    return self.lower_bool_equality(operator.kind, left, right, expression.span);
                }
                ExpressionKind::PrimitiveBinary {
                    operator: lower_binary_primitive(operator.kind),
                    left: Box::new(self.lower_expression(left)),
                    right: Box::new(self.lower_expression(right)),
                }
            }
        };
        Expression {
            kind,
            ty: expression.ty.clone(),
            span: expression.span,
        }
    }

    fn lower_lambda(&mut self, lambda: &checked::Lambda) -> Lambda {
        let parameter_type = match lambda.parameters.as_slice() {
            [] => checked::Type::Unit,
            [parameter] => parameter.ty.clone(),
            parameters => checked::Type::Product(
                parameters
                    .iter()
                    .map(|parameter| parameter.ty.clone())
                    .collect(),
            ),
        };
        let parameter_binding = match lambda.parameters.as_slice() {
            [] => None,
            [parameter] => Some(ValueId::Source(parameter.binding.id)),
            _ => Some(self.temporary()),
        };
        let mut body = self.lower_body(&lambda.body.items, &lambda.body.result);
        if lambda.parameters.len() > 1 {
            let parameter_id = parameter_binding.expect("multiple parameters use a product value");
            let destructuring = Binding {
                pattern: Pattern::Product {
                    elements: lambda
                        .parameters
                        .iter()
                        .map(|parameter| Pattern::Binding {
                            id: ValueId::Source(parameter.binding.id),
                            ty: parameter.ty.clone(),
                        })
                        .collect(),
                    ty: parameter_type.clone(),
                    span: lambda.body.span,
                },
                value: self.reference(parameter_id, parameter_type.clone(), lambda.body.span),
                span: lambda.body.span,
            };
            let result_type = body.ty.clone();
            body = Expression {
                kind: ExpressionKind::Let {
                    binding: Box::new(destructuring),
                    body: Box::new(body),
                },
                ty: result_type,
                span: lambda.body.span,
            };
        }
        Lambda {
            id: lambda.id,
            self_binding: lambda.self_binding.map(ValueId::Source),
            captures: lambda
                .captures
                .iter()
                .map(|capture| Capture {
                    source: ValueId::Source(capture.source.id),
                    binding: ValueId::Source(capture.binding.id),
                    ty: capture.ty.clone(),
                })
                .collect(),
            parameter: Parameter {
                binding: parameter_binding,
                ty: parameter_type,
                span: lambda
                    .parameters
                    .first()
                    .map_or(lambda.body.span, |parameter| parameter.span),
            },
            body: Box::new(body),
        }
    }

    fn lower_body(
        &mut self,
        items: &[checked::BodyItem],
        result: &checked::Expression,
    ) -> Expression {
        let mut body = self.lower_expression(result);
        for item in items.iter().rev() {
            let binding = match item {
                checked::BodyItem::Binding(binding) => self.lower_binding(binding),
                checked::BodyItem::Expression(value) => Binding {
                    pattern: Pattern::Wildcard {
                        ty: value.ty.clone(),
                        span: value.span,
                    },
                    value: self.lower_expression(value),
                    span: value.span,
                },
            };
            let span = Span::new(
                binding.span.file(),
                binding.span.start().min(body.span.start()),
                binding.span.end().max(body.span.end()),
            );
            let ty = body.ty.clone();
            body = Expression {
                kind: ExpressionKind::Let {
                    binding: Box::new(binding),
                    body: Box::new(body),
                },
                ty,
                span,
            };
        }
        body
    }

    fn lower_case_arm(&mut self, arm: &checked::CaseArm) -> CaseArm {
        CaseArm {
            index: arm.index,
            pattern: self.lower_pattern(&arm.pattern),
            value: self.lower_body(&arm.body.items, &arm.body.result),
            span: arm.span,
        }
    }

    fn case(
        &self,
        scrutinee: Expression,
        arms: Vec<CaseArm>,
        ty: checked::Type,
        span: Span,
    ) -> Expression {
        Expression {
            kind: ExpressionKind::Case {
                scrutinee: Box::new(scrutinee),
                arms,
            },
            ty,
            span,
        }
    }

    fn wildcard_arm(&self, index: usize, value: Expression, span: Span) -> CaseArm {
        CaseArm {
            index,
            pattern: Pattern::Wildcard {
                ty: checked::Type::Unit,
                span,
            },
            value,
            span,
        }
    }

    fn reference(&self, id: ValueId, ty: checked::Type, span: Span) -> Expression {
        Expression {
            kind: ExpressionKind::Reference(id),
            ty,
            span,
        }
    }

    fn temporary(&mut self) -> ValueId {
        let id = ValueId::Temporary(self.next_temporary);
        self.next_temporary += 1;
        id
    }
}
