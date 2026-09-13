use crate::ast::{BinaryOperator, UnaryOperator};
use crate::check::ast as checked;
use crate::resolve::ast::{FALSE_VALUE, TRUE_VALUE};
use crate::source::Span;

pub mod ast;
mod bool;
mod completion;
mod interface;
mod pattern;
mod primitive;

use self::bool::bool_type;
pub use self::interface::lower_interface;
use self::primitive::lower_binary_primitive;

use self::ast::{
    Binding, Capture, CaseArm, Expression, ExpressionKind, Lambda, Parameter, Pattern, Program,
    TopLevelBinding, UnaryPrimitive, ValueId,
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
        let mut bindings = program
            .items
            .iter()
            .filter_map(|item| match &item.kind {
                checked::TopItem::ExternalOperation {
                    id,
                    binding,
                    lambda_id,
                    parameter,
                    result,
                    ..
                } => Some(self.lower_external_operation(
                    *id, binding, *lambda_id, parameter, result, item.span,
                )),
                _ => None,
            })
            .collect::<Vec<_>>();
        bindings.extend(program.items.iter().filter_map(|item| match &item.kind {
            checked::TopItem::Binding(binding) => Some(self.lower_top_level_binding(binding)),
            _ => None,
        }));
        Program {
            interface: lower_interface(program),
            bindings,
            span: program.span,
        }
    }

    fn lower_external_operation(
        &mut self,
        id: crate::resolve::ast::ExternalOperationId,
        binding: &crate::resolve::ast::ValueBinding,
        lambda_id: crate::resolve::ast::LambdaId,
        parameter: &checked::Type,
        result: &checked::Type,
        span: Span,
    ) -> TopLevelBinding {
        let parameter_binding = (parameter != &checked::Type::Unit).then(|| self.temporary());
        let argument = parameter_binding.map_or(
            Expression {
                kind: ExpressionKind::Unit,
                ty: checked::Type::Unit,
                span,
            },
            |parameter_binding| self.reference(parameter_binding, parameter.clone(), span),
        );
        let function_type = checked::Type::Function {
            parameter: parameter.clone().into(),
            result: result.clone().into(),
        };
        TopLevelBinding {
            pattern: self::ast::TopLevelPattern::Binding {
                id: ValueId::Source(binding.id),
                name: binding.name.text.clone(),
                ty: function_type.clone(),
            },
            value: Expression {
                kind: ExpressionKind::Lambda(Lambda {
                    id: lambda_id,
                    self_binding: None,
                    captures: Vec::new(),
                    parameter: Parameter {
                        binding: parameter_binding,
                        ty: parameter.clone(),
                        span,
                    },
                    body: Box::new(Expression {
                        kind: ExpressionKind::ExternalCall {
                            id,
                            argument: Box::new(argument),
                        },
                        ty: result.clone(),
                        span,
                    }),
                }),
                ty: function_type,
                span,
            },
            span,
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
            checked::ExpressionKind::Symbol(value) => ExpressionKind::Symbol(value.clone()),
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
            checked::ExpressionKind::SymbolLength { value } => ExpressionKind::SymbolLength {
                value: Box::new(self.lower_expression(value)),
            },
            checked::ExpressionKind::SymbolAt { argument } => ExpressionKind::SymbolAt {
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
            checked::ExpressionKind::NumericConversion { value } => {
                ExpressionKind::NumericConversion {
                    value: Box::new(self.lower_expression(value)),
                }
            }
            checked::ExpressionKind::InjectionConstructor { lambda_id, index } => {
                let checked::Type::Function { parameter, result } = &expression.ty else {
                    unreachable!("an injection constructor has a function type");
                };
                let parameter_binding =
                    (parameter.as_ref() != &checked::Type::Unit).then(|| self.temporary());
                let value = parameter_binding.map_or(
                    Expression {
                        kind: ExpressionKind::Unit,
                        ty: checked::Type::Unit,
                        span: expression.span,
                    },
                    |parameter_id| {
                        self.reference(parameter_id, parameter.as_ref().clone(), expression.span)
                    },
                );
                ExpressionKind::Lambda(Lambda {
                    id: *lambda_id,
                    self_binding: None,
                    captures: Vec::new(),
                    parameter: Parameter {
                        binding: parameter_binding,
                        ty: parameter.as_ref().clone(),
                        span: expression.span,
                    },
                    body: Box::new(Expression {
                        kind: ExpressionKind::SumInjection {
                            index: *index,
                            value: Box::new(value),
                        },
                        ty: result.as_ref().clone(),
                        span: expression.span,
                    }),
                })
            }
            checked::ExpressionKind::SumElimination {
                scrutinee,
                continuations,
            } => {
                let checked::Type::Sum(members) = &scrutinee.ty else {
                    unreachable!("sum elimination has a sum scrutinee");
                };
                let arms = continuations
                    .iter()
                    .zip(members.iter())
                    .enumerate()
                    .map(|(index, (continuation, member))| {
                        let payload_id = self.temporary();
                        let payload = self.reference(payload_id, member.clone(), continuation.span);
                        CaseArm {
                            index,
                            pattern: Pattern::Binding {
                                id: payload_id,
                                ty: member.clone(),
                            },
                            value: Expression {
                                kind: ExpressionKind::Call {
                                    callee: Box::new(self.lower_expression(continuation)),
                                    argument: Box::new(payload),
                                },
                                ty: expression.ty.clone(),
                                span: continuation.span,
                            },
                            span: continuation.span,
                        }
                    })
                    .collect();
                ExpressionKind::Case {
                    scrutinee: Box::new(self.lower_expression(scrutinee)),
                    arms,
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
            checked::ExpressionKind::Unary { operator, operand } => {
                if operator.kind == UnaryOperator::LogicalNot {
                    return self.lower_logical_not(operand, expression.span);
                }
                ExpressionKind::PrimitiveUnary {
                    operator: match operator.kind {
                        UnaryOperator::Negate => UnaryPrimitive::Negate,
                        UnaryOperator::BitwiseNot => UnaryPrimitive::BitwiseNot,
                        UnaryOperator::LogicalNot | UnaryOperator::SymbolLength => {
                            unreachable!("type checking rejects non-numeric core primitives")
                        }
                    },
                    operand: Box::new(self.lower_expression(operand)),
                }
            }
            checked::ExpressionKind::Binary { .. } => return self.lower_binary_chain(expression),
        };
        Expression {
            kind,
            ty: expression.ty.clone(),
            span: expression.span,
        }
    }

    fn lower_binary_chain(&mut self, expression: &checked::Expression) -> Expression {
        let mut outer = Vec::new();
        let mut current = expression;
        while let checked::ExpressionKind::Binary {
            operator,
            left,
            right,
        } = &current.kind
            && matches!(left.kind, checked::ExpressionKind::Binary { .. })
        {
            outer.push((
                operator.kind,
                right.as_ref(),
                current.ty.clone(),
                current.span,
            ));
            current = left;
        }

        let checked::ExpressionKind::Binary {
            operator,
            left,
            right,
        } = &current.kind
        else {
            unreachable!("caller selects a binary expression");
        };
        let left = self.lower_expression(left);
        let mut lowered = self.lower_binary_after_left(
            operator.kind,
            left,
            right,
            current.ty.clone(),
            current.span,
        );
        let mut preceding = Vec::with_capacity(outer.len());
        while let Some((operator, right, ty, span)) = outer.pop() {
            let id = self.temporary();
            let left = self.reference(id, lowered.ty.clone(), lowered.span);
            preceding.push((id, lowered));
            lowered = self.lower_binary_after_left(operator, left, right, ty, span);
        }
        while let Some((id, value)) = preceding.pop() {
            let span = value.span;
            lowered = self.temporary_let(id, value, lowered, span);
        }
        lowered
    }

    fn lower_binary_after_left(
        &mut self,
        operator: BinaryOperator,
        left: Expression,
        right: &checked::Expression,
        result_type: checked::Type,
        span: Span,
    ) -> Expression {
        if matches!(
            operator,
            BinaryOperator::LogicalAnd | BinaryOperator::LogicalOr
        ) {
            return self.lower_short_circuit_after_left(operator, left, right, span);
        }
        if operator == BinaryOperator::SymbolAt {
            unreachable!("Symbol access is lowered before generic binary operators");
        }
        if matches!(operator, BinaryOperator::Equal | BinaryOperator::NotEqual)
            && left.ty == bool_type()
        {
            return self.lower_bool_equality_after_left(operator, left, right, span);
        }
        Expression {
            kind: ExpressionKind::PrimitiveBinary {
                operator: lower_binary_primitive(operator),
                left: Box::new(left),
                right: Box::new(self.lower_expression(right)),
            },
            ty: result_type,
            span,
        }
    }

    fn lower_lambda(&mut self, lambda: &checked::Lambda) -> Lambda {
        let parameter_type = lambda.parameter_type.clone();
        let parameter_binding = match lambda.parameter.as_deref() {
            None | Some(checked::Pattern::Wildcard { .. }) => None,
            Some(checked::Pattern::Binding { binding, .. }) => Some(ValueId::Source(binding.id)),
            Some(checked::Pattern::Product { .. }) => Some(self.temporary()),
        };
        let mut body =
            self.lower_lambda_body(&lambda.body.items, &lambda.body.result, &lambda.result_type);
        if let Some(pattern @ checked::Pattern::Product { .. }) = lambda.parameter.as_deref() {
            let parameter_id = parameter_binding.expect("a product pattern uses a product value");
            let destructuring = Binding {
                pattern: self.lower_pattern(pattern),
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
                span: lambda.parameter.as_deref().map_or(
                    lambda.body.span,
                    |pattern| match pattern {
                        checked::Pattern::Binding { binding, .. } => binding.name.span,
                        checked::Pattern::Wildcard { span, .. }
                        | checked::Pattern::Product { span, .. } => *span,
                    },
                ),
            },
            body: Box::new(body),
        }
    }

    fn lower_body(
        &mut self,
        items: &[checked::BodyItem],
        result: &checked::Completion,
    ) -> Expression {
        let checked::Completion::Value(result) = result else {
            unreachable!("direct lowering only receives value-completing blocks");
        };
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
