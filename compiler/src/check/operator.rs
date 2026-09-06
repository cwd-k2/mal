use crate::ast::{BinaryOperator, Node, UnaryOperator};
use crate::diagnostic::Diagnostic;
use crate::resolve::ast as resolved;
use crate::source::Span;

use super::Checker;
use super::ast::{Expression, ExpressionKind, MemoryPrimitive, Type};
use super::float::{is_contextual_float, is_float};
use super::integer::{
    integer_is_signed, integer_negative_magnitude, is_contextual_integer, is_integer, literal_type,
    parse_magnitude, unparenthesized_integer,
};
use super::types::{bool_type, type_name};

impl Checker {
    pub(super) fn check_unary(
        &mut self,
        operator: &Node<UnaryOperator>,
        operand: &Node<resolved::Expression>,
        span: Span,
        expected: Option<&Type>,
    ) -> Result<Expression, Diagnostic> {
        if operator.kind == UnaryOperator::EngramLength {
            let value = self.check_expression(operand, Some(&Type::Engram))?;
            return Ok(Expression {
                kind: ExpressionKind::EngramLength {
                    value: Box::new(value),
                },
                ty: Type::UInt64,
                span,
            });
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
            let operand = self.check_expression(
                operand,
                expected.filter(|expected| is_integer(expected) || is_float(expected)),
            )?;
            if !is_integer(&operand.ty) && !is_float(&operand.ty) {
                return Err(
                    Diagnostic::error("numeric negation requires a numeric value").with_primary(
                        operand.span,
                        format!("this has type `{}`", type_name(&operand.ty)),
                    ),
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
        if operator.kind == UnaryOperator::BitwiseNot {
            let operand =
                self.check_expression(operand, expected.filter(|expected| is_integer(expected)))?;
            if !is_integer(&operand.ty) {
                return Err(
                    Diagnostic::error("integer unary operator requires an integer").with_primary(
                        operand.span,
                        format!("this has type `{}`", type_name(&operand.ty)),
                    ),
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
    ) -> Result<Expression, Diagnostic> {
        if operator.kind == BinaryOperator::EngramAt {
            let left = self.check_expression(left, Some(&Type::Engram))?;
            let right = self.check_expression(right, Some(&Type::UInt64))?;
            return Ok(Expression {
                kind: ExpressionKind::EngramAt {
                    argument: Box::new(Expression {
                        kind: ExpressionKind::Product(vec![left, right]),
                        ty: Type::Product(vec![Type::Engram, Type::UInt64]),
                        span,
                    }),
                },
                ty: Type::UInt8,
                span,
            });
        }
        if matches!(
            operator.kind,
            BinaryOperator::Add | BinaryOperator::Subtract
        ) {
            return self.check_pointer_or_numeric_arithmetic(operator, left, right, span, expected);
        }
        let expected_integer = expected.filter(|expected| is_integer(expected));
        let expected_numeric =
            expected.filter(|expected| is_integer(expected) || is_float(expected));
        let (left, right, result) = match operator.kind {
            BinaryOperator::Multiply | BinaryOperator::Divide => {
                let (left, right) = self.check_numeric_operands(left, right, expected_numeric)?;
                let result = left.ty.clone();
                (left, right, result)
            }
            BinaryOperator::Remainder => {
                let (left, right) = self.check_integer_operands(left, right, expected_integer)?;
                let result = left.ty.clone();
                (left, right, result)
            }
            BinaryOperator::Less
            | BinaryOperator::LessEqual
            | BinaryOperator::Greater
            | BinaryOperator::GreaterEqual => {
                let (left, right) = self.check_numeric_operands(left, right, None)?;
                (left, right, bool_type())
            }
            BinaryOperator::Equal | BinaryOperator::NotEqual => {
                let left_contextual = is_contextual_integer(left) || is_contextual_float(left);
                let right_contextual = is_contextual_integer(right) || is_contextual_float(right);
                let (left, right) = if left_contextual && !right_contextual {
                    let right = self.check_expression(right, None)?;
                    let left = self.check_expression(left, Some(&right.ty))?;
                    (left, right)
                } else {
                    let left = self.check_expression(left, None)?;
                    let right = self.check_expression(right, Some(&left.ty))?;
                    (left, right)
                };
                if !is_integer(&left.ty)
                    && !is_float(&left.ty)
                    && left.ty != bool_type()
                    && left.ty != Type::Engram
                {
                    return Err(Diagnostic::error("equality is not defined for this type")
                        .with_primary(
                            left.span,
                            format!("this has type `{}`", type_name(&left.ty)),
                        ));
                }
                (left, right, bool_type())
            }
            BinaryOperator::LogicalAnd | BinaryOperator::LogicalOr => (
                self.check_expression(left, Some(&bool_type()))?,
                self.check_expression(right, Some(&bool_type()))?,
                bool_type(),
            ),
            BinaryOperator::ShiftLeft
            | BinaryOperator::ShiftRight
            | BinaryOperator::BitwiseAnd
            | BinaryOperator::BitwiseXor
            | BinaryOperator::BitwiseOr => {
                let (left, right) = self.check_integer_operands(left, right, expected_integer)?;
                let result = left.ty.clone();
                (left, right, result)
            }
            BinaryOperator::EngramAt | BinaryOperator::Add | BinaryOperator::Subtract => {
                unreachable!("specialized operators are checked separately")
            }
        };
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

    fn check_pointer_or_numeric_arithmetic(
        &mut self,
        operator: &Node<BinaryOperator>,
        left: &Node<resolved::Expression>,
        right: &Node<resolved::Expression>,
        span: Span,
        expected: Option<&Type>,
    ) -> Result<Expression, Diagnostic> {
        let pointer_primitive = match operator.kind {
            BinaryOperator::Add => MemoryPrimitive::OffsetForward,
            BinaryOperator::Subtract => MemoryPrimitive::OffsetBackward,
            _ => unreachable!("caller selects addition or subtraction"),
        };
        if expected == Some(&Type::Ptr) {
            let left = self.check_expression(left, Some(&Type::Ptr))?;
            return self.check_pointer_offset(pointer_primitive, left, right, span);
        }

        let expected_numeric =
            expected.filter(|expected| is_integer(expected) || is_float(expected));
        let (left, right) = if expected_numeric.is_some()
            || is_contextual_integer(left)
            || is_contextual_float(left)
        {
            self.check_numeric_operands(left, right, expected_numeric)?
        } else {
            let left = self.check_expression(left, None)?;
            if left.ty == Type::Ptr {
                return self.check_pointer_offset(pointer_primitive, left, right, span);
            }
            let right = self.check_expression(right, Some(&left.ty))?;
            if !is_integer(&left.ty) && !is_float(&left.ty) {
                return Err(
                    Diagnostic::error("numeric operator requires numeric operands").with_primary(
                        left.span,
                        format!("this has type `{}`", type_name(&left.ty)),
                    ),
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

    fn check_pointer_offset(
        &mut self,
        primitive: MemoryPrimitive,
        left: Expression,
        right: &Node<resolved::Expression>,
        span: Span,
    ) -> Result<Expression, Diagnostic> {
        let right = self.check_expression(right, Some(&Type::UInt64))?;
        Ok(Expression {
            kind: ExpressionKind::Memory {
                primitive,
                argument: Box::new(Expression {
                    kind: ExpressionKind::Product(vec![left, right]),
                    ty: Type::Product(vec![Type::Ptr, Type::UInt64]),
                    span,
                }),
            },
            ty: Type::Ptr,
            span,
        })
    }

    fn check_numeric_operands(
        &mut self,
        left: &Node<resolved::Expression>,
        right: &Node<resolved::Expression>,
        expected: Option<&Type>,
    ) -> Result<(Expression, Expression), Diagnostic> {
        let left_contextual = is_contextual_integer(left) || is_contextual_float(left);
        let right_contextual = is_contextual_integer(right) || is_contextual_float(right);
        let (left, right) = if let Some(expected) = expected {
            (
                self.check_expression(left, Some(expected))?,
                self.check_expression(right, Some(expected))?,
            )
        } else if left_contextual && !right_contextual {
            let right = self.check_expression(right, None)?;
            let left = self.check_expression(left, Some(&right.ty))?;
            (left, right)
        } else {
            let left = self.check_expression(left, None)?;
            let right = self.check_expression(right, Some(&left.ty))?;
            (left, right)
        };
        if !is_integer(&left.ty) && !is_float(&left.ty) {
            return Err(
                Diagnostic::error("numeric operator requires numeric operands").with_primary(
                    left.span,
                    format!("this has type `{}`", type_name(&left.ty)),
                ),
            );
        }
        Ok((left, right))
    }
}
