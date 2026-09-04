use crate::ast::{BinaryOperator, UnaryOperator};
use crate::check::ast as checked;
use crate::resolve::ast::{FALSE_VALUE, TRUE_VALUE};
use crate::source::Span;

pub mod ast;

use self::ast::{
    BinaryPrimitive, Binding, Capture, CaseArm, Expression, ExpressionKind, ExternalOperation,
    Lambda, Parameter, Pattern, Program, TopLevelBinding, TopLevelPattern, UnaryPrimitive, ValueId,
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
        let mut bindings = Vec::new();
        for item in &program.items {
            match &item.kind {
                checked::TopItem::TypeAlias { .. } => {}
                checked::TopItem::ExternalOperation {
                    id,
                    name,
                    parameter,
                    result,
                    ..
                } => externals.push(ExternalOperation {
                    id: *id,
                    name: name.text.clone(),
                    parameter: parameter.clone(),
                    result: result.clone(),
                    span: item.span,
                }),
                checked::TopItem::Binding(binding) => {
                    bindings.push(self.lower_top_level_binding(binding));
                }
            }
        }
        Program {
            externals,
            bindings,
            span: program.span,
        }
    }

    fn lower_top_level_binding(&mut self, binding: &checked::Binding) -> TopLevelBinding {
        let pattern = match &binding.pattern {
            checked::Pattern::Binding { binding, ty } => TopLevelPattern::Binding {
                id: ValueId::Source(binding.id),
                name: binding.name.text.clone(),
                ty: ty.clone(),
            },
            checked::Pattern::Wildcard { ty, span } => TopLevelPattern::Wildcard {
                ty: ty.clone(),
                span: *span,
            },
        };
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

    fn lower_pattern(&self, pattern: &checked::Pattern) -> Pattern {
        match pattern {
            checked::Pattern::Binding { binding, ty } => Pattern::Binding {
                id: ValueId::Source(binding.id),
                ty: ty.clone(),
            },
            checked::Pattern::Wildcard { ty, span } => Pattern::Wildcard {
                ty: ty.clone(),
                span: *span,
            },
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
            checked::ExpressionKind::Unit => ExpressionKind::Unit,
            checked::ExpressionKind::Parenthesized(inner) => return self.lower_expression(inner),
            checked::ExpressionKind::Lambda(lambda) => {
                ExpressionKind::Lambda(self.lower_lambda(lambda))
            }
            checked::ExpressionKind::Call { callee, argument } => ExpressionKind::Call {
                callee: Box::new(self.lower_expression(callee)),
                argument: Box::new(self.lower_expression(argument)),
            },
            checked::ExpressionKind::ExternalCall { id, argument, .. } => {
                ExpressionKind::ExternalCall {
                    id: *id,
                    argument: Box::new(self.lower_expression(argument)),
                }
            }
            checked::ExpressionKind::IntegerConversion { value } => {
                ExpressionKind::IntegerConversion {
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
                        UnaryOperator::LogicalNot => {
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
        let parameter = lambda.parameters.first();
        Lambda {
            id: lambda.id,
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
                binding: parameter.map(|parameter| ValueId::Source(parameter.binding.id)),
                ty: parameter.map_or(checked::Type::Unit, |parameter| parameter.ty.clone()),
                span: parameter.map_or(lambda.body.span, |parameter| parameter.span),
            },
            body: Box::new(self.lower_body(&lambda.body.items, &lambda.body.result)),
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

    fn lower_if(
        &mut self,
        condition: &checked::Expression,
        then_branch: &checked::ExpressionBlock,
        else_branch: &checked::ExpressionBlock,
        result_type: &checked::Type,
        span: Span,
    ) -> Expression {
        let otherwise = self.lower_body(&else_branch.items, &else_branch.result);
        let then = self.lower_body(&then_branch.items, &then_branch.result);
        let condition = self.lower_expression(condition);
        self.case(
            condition,
            vec![
                self.wildcard_arm(0, otherwise, else_branch.span),
                self.wildcard_arm(1, then, then_branch.span),
            ],
            result_type.clone(),
            span,
        )
    }

    fn lower_logical_not(&mut self, operand: &checked::Expression, span: Span) -> Expression {
        let scrutinee = self.lower_expression(operand);
        let false_value = self.bool_value(false, span);
        let true_value = self.bool_value(true, span);
        self.case(
            scrutinee,
            vec![
                self.wildcard_arm(0, true_value, span),
                self.wildcard_arm(1, false_value, span),
            ],
            bool_type(),
            span,
        )
    }

    fn lower_short_circuit(
        &mut self,
        operator: BinaryOperator,
        left: &checked::Expression,
        right: &checked::Expression,
        span: Span,
    ) -> Expression {
        let left = self.lower_expression(left);
        let right = self.lower_expression(right);
        let false_value = self.bool_value(false, span);
        let true_value = self.bool_value(true, span);
        let arms = match operator {
            BinaryOperator::LogicalAnd => vec![
                self.wildcard_arm(0, false_value, span),
                self.wildcard_arm(1, right, span),
            ],
            BinaryOperator::LogicalOr => vec![
                self.wildcard_arm(0, right, span),
                self.wildcard_arm(1, true_value, span),
            ],
            _ => unreachable!("caller restricts short-circuit operators"),
        };
        self.case(left, arms, bool_type(), span)
    }

    fn lower_bool_equality(
        &mut self,
        operator: BinaryOperator,
        left: &checked::Expression,
        right: &checked::Expression,
        span: Span,
    ) -> Expression {
        let left_id = self.temporary();
        let right_id = self.temporary();
        let equal = operator == BinaryOperator::Equal;
        let right_when_false = self.bool_case_reference(right_id, equal, !equal, span);
        let right_when_true = self.bool_case_reference(right_id, !equal, equal, span);
        let comparison = self.case(
            self.reference(left_id, bool_type(), span),
            vec![
                self.wildcard_arm(0, right_when_false, span),
                self.wildcard_arm(1, right_when_true, span),
            ],
            bool_type(),
            span,
        );
        let right = self.lower_expression(right);
        let right_let = self.temporary_let(right_id, right, comparison, span);
        let left = self.lower_expression(left);
        self.temporary_let(left_id, left, right_let, span)
    }

    fn bool_case_reference(&self, id: ValueId, zero: bool, one: bool, span: Span) -> Expression {
        self.case(
            self.reference(id, bool_type(), span),
            vec![
                self.wildcard_arm(0, self.bool_value(zero, span), span),
                self.wildcard_arm(1, self.bool_value(one, span), span),
            ],
            bool_type(),
            span,
        )
    }

    fn temporary_let(
        &self,
        id: ValueId,
        value: Expression,
        body: Expression,
        span: Span,
    ) -> Expression {
        Expression {
            kind: ExpressionKind::Let {
                binding: Box::new(Binding {
                    pattern: Pattern::Binding {
                        id,
                        ty: value.ty.clone(),
                    },
                    value,
                    span,
                }),
                body: Box::new(body),
            },
            ty: bool_type(),
            span,
        }
    }

    fn lower_case_arm(&mut self, arm: &checked::CaseArm) -> CaseArm {
        CaseArm {
            index: arm.index,
            pattern: self.lower_pattern(&arm.pattern),
            value: self.lower_expression(&arm.value),
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

    fn bool_value(&self, value: bool, span: Span) -> Expression {
        Expression {
            kind: ExpressionKind::SumInjection {
                index: usize::from(value),
                value: Box::new(Expression {
                    kind: ExpressionKind::Unit,
                    ty: checked::Type::Unit,
                    span,
                }),
            },
            ty: bool_type(),
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

fn bool_type() -> checked::Type {
    checked::Type::Sum(vec![checked::Type::Unit, checked::Type::Unit])
}

fn lower_binary_primitive(operator: BinaryOperator) -> BinaryPrimitive {
    match operator {
        BinaryOperator::Multiply => BinaryPrimitive::Multiply,
        BinaryOperator::Divide => BinaryPrimitive::Divide,
        BinaryOperator::Remainder => BinaryPrimitive::Remainder,
        BinaryOperator::Add => BinaryPrimitive::Add,
        BinaryOperator::Subtract => BinaryPrimitive::Subtract,
        BinaryOperator::ShiftLeft => BinaryPrimitive::ShiftLeft,
        BinaryOperator::ShiftRight => BinaryPrimitive::ShiftRight,
        BinaryOperator::Less => BinaryPrimitive::Less,
        BinaryOperator::LessEqual => BinaryPrimitive::LessEqual,
        BinaryOperator::Greater => BinaryPrimitive::Greater,
        BinaryOperator::GreaterEqual => BinaryPrimitive::GreaterEqual,
        BinaryOperator::Equal => BinaryPrimitive::Equal,
        BinaryOperator::NotEqual => BinaryPrimitive::NotEqual,
        BinaryOperator::BitwiseAnd => BinaryPrimitive::BitwiseAnd,
        BinaryOperator::BitwiseXor => BinaryPrimitive::BitwiseXor,
        BinaryOperator::BitwiseOr => BinaryPrimitive::BitwiseOr,
        BinaryOperator::LogicalAnd | BinaryOperator::LogicalOr => {
            unreachable!("logical operators are lowered separately")
        }
    }
}
