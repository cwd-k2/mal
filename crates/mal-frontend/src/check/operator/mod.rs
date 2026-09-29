//! Operator-family admission and result typing over already resolved operands.

use crate::resolve::ast as resolved;
use mal_syntax::ast::{BinaryOperator, Node, UnaryOperator};
use mal_syntax::diagnostic::Diagnostic;
use mal_syntax::source::Span;

use super::ast::{Expression, ExpressionKind, MemoryPrimitive, SymbolPrimitive, Type};
use super::float::{is_contextual_float, is_float};
use super::integer::{
    integer_is_signed, integer_negative_magnitude, is_contextual_integer, is_integer, literal_type,
    parse_magnitude, unparenthesized_integer,
};
use super::types::{bool_type, type_name};
use super::{CheckFailure, CheckResult, Checker};

mod arithmetic;
mod logical;

use arithmetic::{arithmetic_result, is_target_quantity, unsupported_binary};

/// Operators see an operand of a file-local opaque type as its representation in the declaring file, like every
/// other comparison there. The operand keeps its checked value; only the type the operator rule reads changes.
pub(super) fn view_operand(mut operand: Expression) -> Expression {
    let viewed = super::types::representation_view(&operand.ty, operand.span.file());
    if !std::ptr::eq(viewed, &operand.ty) {
        operand.ty = viewed.clone();
    }
    operand
}

/// Builds a checked binary operation. Symbol and Bool operands select the operation the operator denotes, so later
/// stages never re-derive it from operand types.
pub(super) fn binary_expression(
    operator: &Node<BinaryOperator>,
    left: Expression,
    right: Expression,
    ty: Type,
    span: Span,
) -> Expression {
    if left.ty == bool_type()
        && matches!(
            operator.kind,
            BinaryOperator::Equal | BinaryOperator::NotEqual
        )
    {
        return Expression {
            kind: ExpressionKind::BoolEquality {
                equal: operator.kind == BinaryOperator::Equal,
                left: Box::new(left),
                right: Box::new(right),
            },
            ty,
            span,
        };
    }
    if left.ty == Type::Symbol {
        let primitive = match operator.kind {
            BinaryOperator::Add => Some(SymbolPrimitive::Concatenate),
            BinaryOperator::Divide => Some(SymbolPrimitive::Prefix),
            BinaryOperator::Remainder => Some(SymbolPrimitive::Suffix),
            BinaryOperator::Equal => Some(SymbolPrimitive::Equal),
            BinaryOperator::NotEqual => Some(SymbolPrimitive::NotEqual),
            _ => None,
        };
        if let Some(primitive) = primitive {
            return symbol(primitive, vec![left, right], ty, span);
        }
    }
    Expression {
        kind: ExpressionKind::Binary {
            operator: operator.clone(),
            left: Box::new(left),
            right: Box::new(right),
        },
        ty,
        span,
    }
}

pub(super) fn symbol(
    primitive: SymbolPrimitive,
    operands: Vec<Expression>,
    ty: Type,
    span: Span,
) -> Expression {
    Expression {
        kind: ExpressionKind::SymbolOperation {
            primitive,
            operands,
        },
        ty,
        span,
    }
}

/// The expected type of an operator result, seen through the declaring file's representation view.
pub(super) fn view_expected(expected: Option<&Type>, span: Span) -> Option<&Type> {
    expected.map(|ty| super::types::representation_view(ty, span.file()))
}

impl Checker {
    pub(super) fn binary_left_expected(
        &self,
        operator: &Node<BinaryOperator>,
        expected: Option<&Type>,
    ) -> Option<Type> {
        match operator.kind {
            BinaryOperator::SymbolAt => Some(Type::Symbol),
            BinaryOperator::Add | BinaryOperator::Subtract => expected
                .filter(|ty| {
                    (operator.kind == BinaryOperator::Add && **ty == Type::Symbol)
                        || is_integer(ty)
                        || is_float(ty)
                })
                .cloned(),
            BinaryOperator::Multiply => expected
                .filter(|ty| (is_integer(ty) || is_float(ty)) && **ty != Type::ByteSize)
                .cloned(),
            BinaryOperator::Divide => expected
                .filter(|ty| **ty == Type::Symbol || is_integer(ty) || is_float(ty))
                .cloned(),
            BinaryOperator::Remainder => expected
                .filter(|ty| **ty == Type::Symbol || is_integer(ty))
                .cloned(),
            BinaryOperator::ShiftLeft
            | BinaryOperator::ShiftRight
            | BinaryOperator::BitwiseAnd
            | BinaryOperator::BitwiseXor
            | BinaryOperator::BitwiseOr => expected.filter(|ty| is_integer(ty)).cloned(),
            BinaryOperator::LogicalAnd | BinaryOperator::LogicalOr => Some(bool_type()),
            BinaryOperator::Less
            | BinaryOperator::LessEqual
            | BinaryOperator::Greater
            | BinaryOperator::GreaterEqual
            | BinaryOperator::Equal
            | BinaryOperator::NotEqual => None,
        }
    }

    pub(super) fn check_unary(
        &mut self,
        operator: &Node<UnaryOperator>,
        operand: &Node<resolved::Expression>,
        span: Span,
        expected: Option<&Type>,
    ) -> CheckResult<Expression> {
        if operator.kind == UnaryOperator::Length {
            let value = self.check_expression(operand, None)?;
            let viewed = super::types::representation_view(&value.ty, value.span.file());
            if matches!(viewed, Type::Buffer(_)) {
                return Ok(Expression {
                    kind: ExpressionKind::Memory {
                        primitive: MemoryPrimitive::ViewLength,
                        operands: vec![value],
                    },
                    ty: Type::USize,
                    span,
                });
            }
            self.require_type(&value.ty, &Type::Symbol, value.span)?;
            return Ok(symbol(
                SymbolPrimitive::Length,
                vec![value],
                Type::USize,
                span,
            ));
        }
        if operator.kind == UnaryOperator::Negate
            && let Some(literal) = unparenthesized_integer(operand)
        {
            let ty = literal_type(literal.suffix).unwrap_or_else(|| {
                expected
                    .filter(|ty| is_integer(ty))
                    .cloned()
                    .unwrap_or(Type::Int64)
            });
            let magnitude = parse_magnitude(literal, operand.span)?;
            if integer_is_signed(&ty) && magnitude == integer_negative_magnitude(&ty) {
                return Ok(Expression {
                    kind: ExpressionKind::Integer(
                        -i128::try_from(magnitude).expect("signed literal magnitudes fit in i128"),
                    ),
                    ty,
                    span,
                });
            }
        }
        if operator.kind == UnaryOperator::Negate {
            let operand = view_operand(self.check_expression(
                operand,
                expected.filter(|expected| is_integer(expected) || is_float(expected)),
            )?);
            if (!is_integer(&operand.ty) && !is_float(&operand.ty))
                || is_target_quantity(&operand.ty)
            {
                return Err(Diagnostic::error("negation is not defined for this type")
                    .with_primary(
                        operand.span,
                        format!("this has type `{}`", type_name(&operand.ty)),
                    )
                    .into());
            }
            let operand_type = operand.ty.clone();
            return Ok(Expression {
                kind: ExpressionKind::Unary {
                    operator: operator.clone(),
                    operand: Box::new(operand),
                },
                ty: operand_type,
                span,
            });
        }
        if operator.kind == UnaryOperator::BitwiseNot {
            let operand = view_operand(
                self.check_expression(operand, expected.filter(|expected| is_integer(expected)))?,
            );
            if !is_integer(&operand.ty) {
                return Err(
                    Diagnostic::error("integer unary operator requires an integer")
                        .with_primary(
                            operand.span,
                            format!("this has type `{}`", type_name(&operand.ty)),
                        )
                        .into(),
                );
            }
            if is_target_quantity(&operand.ty) {
                return Err(
                    Diagnostic::error("bitwise not is not defined for this type")
                        .with_primary(
                            operand.span,
                            format!("this has type `{}`", type_name(&operand.ty)),
                        )
                        .into(),
                );
            }
            let operand_type = operand.ty.clone();
            return Ok(Expression {
                kind: ExpressionKind::Unary {
                    operator: operator.clone(),
                    operand: Box::new(operand),
                },
                ty: operand_type,
                span,
            });
        }
        let operand_type = bool_type();
        let operand = self.check_expression(operand, Some(&operand_type))?;
        Ok(Expression {
            kind: ExpressionKind::Unary {
                operator: operator.clone(),
                operand: Box::new(operand),
            },
            ty: operand_type,
            span,
        })
    }

    pub(super) fn check_binary(
        &mut self,
        operator: &Node<BinaryOperator>,
        left: &Node<resolved::Expression>,
        right: &Node<resolved::Expression>,
        span: Span,
        expected: Option<&Type>,
    ) -> CheckResult<Expression> {
        if operator.kind == BinaryOperator::SymbolAt {
            let left = view_operand(self.check_before(left, None, right.span)?);
            return self.check_binary_after_left(operator, left, right, span);
        }
        if matches!(
            operator.kind,
            BinaryOperator::Add | BinaryOperator::Subtract
        ) {
            return self.check_additive(operator, left, right, span, expected);
        }
        if matches!(
            operator.kind,
            BinaryOperator::Divide | BinaryOperator::Remainder
        ) && !is_contextual_integer(left)
            && !is_contextual_float(left)
        {
            let left_expected = self.binary_left_expected(operator, expected);
            let left = view_operand(self.check_before(left, left_expected.as_ref(), right.span)?);
            return self.check_binary_after_left(operator, left, right, span);
        }
        let expected_integer = expected.filter(|expected| is_integer(expected));
        let expected_numeric =
            expected.filter(|expected| is_integer(expected) || is_float(expected));
        let (left, right, result) = match operator.kind {
            BinaryOperator::Multiply => {
                let (left, right) =
                    self.check_multiplication_operands(left, right, expected_numeric)?;
                let result = arithmetic_result(operator.kind, &left.ty, &right.ty)
                    .ok_or_else(|| unsupported_binary(operator.kind, &left))?;
                (left, right, result)
            }
            BinaryOperator::Divide => {
                let (left, right) = self.check_numeric_operands(left, right, expected_numeric)?;
                let result = arithmetic_result(operator.kind, &left.ty, &right.ty)
                    .ok_or_else(|| unsupported_binary(operator.kind, &left))?;
                (left, right, result)
            }
            BinaryOperator::Remainder => {
                let (left, right) = self.check_integer_operands(left, right, expected_integer)?;
                let result = arithmetic_result(operator.kind, &left.ty, &right.ty)
                    .ok_or_else(|| unsupported_binary(operator.kind, &left))?;
                (left, right, result)
            }
            BinaryOperator::Less
            | BinaryOperator::LessEqual
            | BinaryOperator::Greater
            | BinaryOperator::GreaterEqual => {
                let (left, right) = self.check_numeric_operands(left, right, None)?;
                if arithmetic_result(operator.kind, &left.ty, &right.ty).is_none() {
                    return Err(unsupported_binary(operator.kind, &left));
                }
                (left, right, bool_type())
            }
            BinaryOperator::Equal | BinaryOperator::NotEqual => {
                let left_contextual = is_contextual_integer(left) || is_contextual_float(left);
                let right_contextual = is_contextual_integer(right) || is_contextual_float(right);
                let (left, right) = if left_contextual && !right_contextual {
                    let right = match self.check_expression(right, None) {
                        Err(CheckFailure::Abrupt(abrupt)) => {
                            let left = self.check_expression(left, None)?;
                            return Err(CheckFailure::Abrupt(Box::new(
                                (*abrupt).preceded_by(vec![left]),
                            )));
                        }
                        result => view_operand(result?),
                    };
                    let left = self.check_before(left, Some(&right.ty), right.span)?;
                    (left, right)
                } else {
                    let left = view_operand(self.check_before(left, None, right.span)?);
                    let expected = left.ty.clone();
                    self.check_after(left, right, Some(&expected))?
                };
                if !is_integer(&left.ty)
                    && !is_float(&left.ty)
                    && left.ty != bool_type()
                    && left.ty != Type::Symbol
                {
                    return Err(Diagnostic::error("equality is not defined for this type")
                        .with_primary(
                            left.span,
                            format!("this has type `{}`", type_name(&left.ty)),
                        )
                        .into());
                }
                (left, right, bool_type())
            }
            BinaryOperator::LogicalAnd | BinaryOperator::LogicalOr => {
                return self.check_logical(operator, left, right, span);
            }
            BinaryOperator::ShiftLeft
            | BinaryOperator::ShiftRight
            | BinaryOperator::BitwiseAnd
            | BinaryOperator::BitwiseXor
            | BinaryOperator::BitwiseOr => {
                let (left, right) = self.check_integer_operands(left, right, expected_integer)?;
                let result = arithmetic_result(operator.kind, &left.ty, &right.ty)
                    .ok_or_else(|| unsupported_binary(operator.kind, &left))?;
                (left, right, result)
            }
            BinaryOperator::SymbolAt | BinaryOperator::Add | BinaryOperator::Subtract => {
                unreachable!("specialized operators are checked separately")
            }
        };
        Ok(binary_expression(operator, left, right, result, span))
    }

    pub(super) fn check_binary_after_left(
        &mut self,
        operator: &Node<BinaryOperator>,
        left: Expression,
        right: &Node<resolved::Expression>,
        span: Span,
    ) -> CheckResult<Expression> {
        match operator.kind {
            BinaryOperator::SymbolAt => {
                self.require_type(&left.ty, &Type::Symbol, left.span)?;
                let (left, right) = self.check_after(left, right, Some(&Type::USize))?;
                return Ok(symbol(
                    SymbolPrimitive::ByteAt,
                    vec![left, right],
                    Type::UInt8,
                    span,
                ));
            }
            BinaryOperator::LogicalAnd | BinaryOperator::LogicalOr => {
                self.require_type(&left.ty, &bool_type(), left.span)?;
                return self.check_logical_after_left(operator, left, right, span);
            }
            BinaryOperator::Add if left.ty == Type::Symbol => {
                let (left, right) = self.check_after(left, right, Some(&Type::Symbol))?;
                return Ok(binary_expression(operator, left, right, Type::Symbol, span));
            }
            BinaryOperator::Divide | BinaryOperator::Remainder if left.ty == Type::Symbol => {
                let (left, right) = self.check_after(left, right, Some(&Type::USize))?;
                return Ok(binary_expression(operator, left, right, Type::Symbol, span));
            }
            _ => {}
        }

        let numeric = is_integer(&left.ty) || is_float(&left.ty);
        let integer = is_integer(&left.ty);
        let expected = left.ty.clone();
        let right_expected =
            if operator.kind == BinaryOperator::Multiply && is_target_quantity(&left.ty) {
                None
            } else {
                Some(&expected)
            };
        let (left, right) = self.check_after(left, right, right_expected)?;
        let valid = match operator.kind {
            BinaryOperator::Add
            | BinaryOperator::Subtract
            | BinaryOperator::Multiply
            | BinaryOperator::Divide
            | BinaryOperator::Less
            | BinaryOperator::LessEqual
            | BinaryOperator::Greater
            | BinaryOperator::GreaterEqual => {
                numeric && arithmetic_result(operator.kind, &left.ty, &right.ty).is_some()
            }
            BinaryOperator::Remainder
            | BinaryOperator::ShiftLeft
            | BinaryOperator::ShiftRight
            | BinaryOperator::BitwiseAnd
            | BinaryOperator::BitwiseXor
            | BinaryOperator::BitwiseOr => {
                integer && arithmetic_result(operator.kind, &left.ty, &right.ty).is_some()
            }
            BinaryOperator::Equal | BinaryOperator::NotEqual => {
                numeric || left.ty == bool_type() || left.ty == Type::Symbol
            }
            BinaryOperator::SymbolAt | BinaryOperator::LogicalAnd | BinaryOperator::LogicalOr => {
                unreachable!("specialized operators return above")
            }
        };
        if !valid {
            let integer_operator = matches!(
                operator.kind,
                BinaryOperator::Remainder
                    | BinaryOperator::ShiftLeft
                    | BinaryOperator::ShiftRight
                    | BinaryOperator::BitwiseAnd
                    | BinaryOperator::BitwiseXor
                    | BinaryOperator::BitwiseOr
            );
            let message = if is_target_quantity(&left.ty) {
                "binary operator is not defined for this type"
            } else if integer_operator && !integer {
                "integer operator requires integer operands"
            } else if integer_operator {
                "operator is not defined for this integer type"
            } else if matches!(
                operator.kind,
                BinaryOperator::Equal | BinaryOperator::NotEqual
            ) {
                "equality is not defined for this type"
            } else {
                "numeric operator requires numeric operands"
            };
            return Err(Diagnostic::error(message)
                .with_primary(
                    left.span,
                    format!("this has type `{}`", type_name(&left.ty)),
                )
                .into());
        }
        let result = if matches!(
            operator.kind,
            BinaryOperator::Less
                | BinaryOperator::LessEqual
                | BinaryOperator::Greater
                | BinaryOperator::GreaterEqual
                | BinaryOperator::Equal
                | BinaryOperator::NotEqual
        ) {
            bool_type()
        } else {
            arithmetic_result(operator.kind, &left.ty, &right.ty).unwrap_or_else(|| left.ty.clone())
        };
        Ok(binary_expression(operator, left, right, result, span))
    }
}
