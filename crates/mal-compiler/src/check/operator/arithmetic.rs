use crate::resolve::ast as resolved;
use mal_syntax::ast::{BinaryOperator, Node};
use mal_syntax::diagnostic::Diagnostic;
use mal_syntax::source::Span;

use super::super::ast::{Expression, ExpressionKind, Type};
use super::super::float::{is_contextual_float, is_float};
use super::super::integer::{is_contextual_integer, is_integer};
use super::super::types::{bool_type, type_name};
use super::super::{CheckFailure, CheckResult, Checker};

impl Checker {
    pub(super) fn check_pointer_or_numeric_arithmetic(
        &mut self,
        operator: &Node<BinaryOperator>,
        left: &Node<resolved::Expression>,
        right: &Node<resolved::Expression>,
        span: Span,
        expected: Option<&Type>,
    ) -> CheckResult<Expression> {
        if operator.kind == BinaryOperator::Add && expected == Some(&Type::Symbol) {
            return self.check_symbol_concatenation(operator, left, right, span);
        }

        let expected_numeric =
            expected.filter(|expected| is_integer(expected) || is_float(expected));
        let (left, right) = if expected_numeric.is_some()
            || is_contextual_integer(left)
            || is_contextual_float(left)
        {
            self.check_numeric_operands(left, right, expected_numeric)?
        } else {
            let left = self.check_before(left, None, right.span)?;
            if operator.kind == BinaryOperator::Add && left.ty == Type::Symbol {
                let (left, right) = self.check_after(left, right, Some(&Type::Symbol))?;
                return Ok(Expression {
                    kind: ExpressionKind::Binary {
                        operator: operator.clone(),
                        left: Box::new(left),
                        right: Box::new(right),
                    },
                    ty: Type::Symbol,
                    span,
                });
            }
            let expected = left.ty.clone();
            let (left, right) = self.check_after(left, right, Some(&expected))?;
            if !is_integer(&left.ty) && !is_float(&left.ty) {
                return Err(
                    Diagnostic::error("numeric operator requires numeric operands")
                        .with_primary(
                            left.span,
                            format!("this has type `{}`", type_name(&left.ty)),
                        )
                        .into(),
                );
            }
            (left, right)
        };
        let result = left.ty.clone();
        Ok(Expression {
            kind: ExpressionKind::Binary {
                operator: operator.clone(),
                left: Box::new(left),
                right: Box::new(right),
            },
            ty: result,
            span,
        })
    }

    fn check_symbol_concatenation(
        &mut self,
        operator: &Node<BinaryOperator>,
        left: &Node<resolved::Expression>,
        right: &Node<resolved::Expression>,
        span: Span,
    ) -> CheckResult<Expression> {
        let left = self.check_before(left, Some(&Type::Symbol), right.span)?;
        let (left, right) = self.check_after(left, right, Some(&Type::Symbol))?;
        Ok(Expression {
            kind: ExpressionKind::Binary {
                operator: operator.clone(),
                left: Box::new(left),
                right: Box::new(right),
            },
            ty: Type::Symbol,
            span,
        })
    }

    pub(super) fn check_numeric_operands(
        &mut self,
        left: &Node<resolved::Expression>,
        right: &Node<resolved::Expression>,
        expected: Option<&Type>,
    ) -> CheckResult<(Expression, Expression)> {
        let left_contextual = is_contextual_integer(left) || is_contextual_float(left);
        let right_contextual = is_contextual_integer(right) || is_contextual_float(right);
        let (left, right) = if let Some(expected) = expected {
            let left = self.check_before(left, Some(expected), right.span)?;
            self.check_after(left, right, Some(expected))?
        } else if left_contextual && !right_contextual {
            let right = match self.check_expression(right, None) {
                Err(CheckFailure::Abrupt(abrupt)) => {
                    let left = self.check_expression(left, None)?;
                    return Err(CheckFailure::Abrupt(Box::new(
                        (*abrupt).preceded_by(vec![left]),
                    )));
                }
                result => result?,
            };
            let left = self.check_before(left, Some(&right.ty), right.span)?;
            (left, right)
        } else {
            let left = self.check_before(left, None, right.span)?;
            let expected = left.ty.clone();
            self.check_after(left, right, Some(&expected))?
        };
        if !is_integer(&left.ty) && !is_float(&left.ty) {
            return Err(
                Diagnostic::error("numeric operator requires numeric operands")
                    .with_primary(
                        left.span,
                        format!("this has type `{}`", type_name(&left.ty)),
                    )
                    .into(),
            );
        }
        Ok((left, right))
    }

    pub(super) fn check_multiplication_operands(
        &mut self,
        left: &Node<resolved::Expression>,
        right: &Node<resolved::Expression>,
        expected: Option<&Type>,
    ) -> CheckResult<(Expression, Expression)> {
        if let Some(expected) = expected.filter(|ty| **ty != Type::ByteSize) {
            return self.check_numeric_operands(left, right, Some(expected));
        }

        let left = self.check_before(left, None, right.span)?;
        let (left, right) = self.check_after(left, right, None)?;
        if (!is_integer(&left.ty) && !is_float(&left.ty))
            || (!is_integer(&right.ty) && !is_float(&right.ty))
        {
            return Err(unsupported_binary(BinaryOperator::Multiply, &left));
        }
        Ok((left, right))
    }
}

pub(super) fn is_target_quantity(ty: &Type) -> bool {
    matches!(ty, Type::ByteSize | Type::USize)
}

fn is_fixed_width_integer(ty: &Type) -> bool {
    is_integer(ty) && !is_target_quantity(ty)
}

pub(super) fn arithmetic_result(
    operator: BinaryOperator,
    left: &Type,
    right: &Type,
) -> Option<Type> {
    use BinaryOperator::*;

    if operator == Multiply
        && matches!(
            (left, right),
            (Type::ByteSize, Type::USize) | (Type::USize, Type::ByteSize)
        )
    {
        return Some(Type::ByteSize);
    }
    if left != right {
        return None;
    }
    let ordinary_numeric = is_fixed_width_integer(left) || is_float(left);
    let result = match operator {
        Add | Subtract if ordinary_numeric || is_target_quantity(left) => left.clone(),
        Multiply if ordinary_numeric || *left == Type::USize => left.clone(),
        Divide if ordinary_numeric || *left == Type::USize => left.clone(),
        Remainder if is_fixed_width_integer(left) || *left == Type::USize => left.clone(),
        Less | LessEqual | Greater | GreaterEqual
            if ordinary_numeric || is_target_quantity(left) =>
        {
            bool_type()
        }
        ShiftLeft | ShiftRight | BitwiseAnd | BitwiseXor | BitwiseOr
            if is_fixed_width_integer(left) =>
        {
            left.clone()
        }
        _ => return None,
    };
    Some(result)
}

pub(super) fn unsupported_binary(operator: BinaryOperator, operand: &Expression) -> CheckFailure {
    let _ = operator;
    Diagnostic::error("binary operator is not defined for this type")
        .with_primary(
            operand.span,
            format!("this has type `{}`", type_name(&operand.ty)),
        )
        .into()
}
