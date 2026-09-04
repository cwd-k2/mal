use crate::ast::Node;
use crate::diagnostic::Diagnostic;
use crate::lexer::{IntegerLiteral, IntegerSuffix};
use crate::resolve::ast as resolved;
use crate::source::Span;

use super::Checker;
use super::ast::{Expression, ExpressionKind, Type};
use super::expression::type_name;

impl Checker {
    pub(super) fn check_integer(
        &self,
        literal: &IntegerLiteral,
        span: Span,
        expected: Option<&Type>,
    ) -> Result<Expression, Diagnostic> {
        let ty = literal_type(literal.suffix).unwrap_or_else(|| {
            expected
                .filter(|ty| is_integer(ty))
                .cloned()
                .unwrap_or(Type::Int64)
        });
        let magnitude = parse_magnitude(literal, span)?;
        let maximum = integer_positive_maximum(&ty);
        if magnitude > maximum {
            return Err(
                Diagnostic::error(format!("{} literal is out of range", type_name(&ty)))
                    .with_primary(span, format!("expected a value from 0 through {maximum}")),
            );
        }
        Ok(Expression {
            kind: ExpressionKind::Integer(
                i128::try_from(magnitude).expect("valid integer literals fit in i128"),
            ),
            ty,
            span,
        })
    }

    pub(super) fn check_integer_operands(
        &mut self,
        left: &Node<resolved::Expression>,
        right: &Node<resolved::Expression>,
        expected: Option<&Type>,
    ) -> Result<(Expression, Expression), Diagnostic> {
        let (left, right) = if let Some(expected) = expected {
            (
                self.check_expression(left, Some(expected))?,
                self.check_expression(right, Some(expected))?,
            )
        } else if is_contextual_integer(left) && !is_contextual_integer(right) {
            let right = self.check_expression(right, None)?;
            let left = self.check_expression(left, Some(&right.ty))?;
            (left, right)
        } else {
            let left = self.check_expression(left, None)?;
            let right = self.check_expression(right, Some(&left.ty))?;
            (left, right)
        };
        if !is_integer(&left.ty) {
            return Err(
                Diagnostic::error("integer operator requires integer operands").with_primary(
                    left.span,
                    format!("this has type `{}`", type_name(&left.ty)),
                ),
            );
        }
        Ok((left, right))
    }
}

pub(super) fn parse_magnitude(literal: &IntegerLiteral, span: Span) -> Result<u128, Diagnostic> {
    u128::from_str_radix(&literal.digits, literal.radix.value()).map_err(|_| {
        Diagnostic::error("integer literal is too large").with_primary(
            span,
            "the value is too large for a fixed-width integer literal",
        )
    })
}

pub(super) fn parse_index(literal: &IntegerLiteral, span: Span) -> Result<usize, Diagnostic> {
    let value = parse_magnitude(literal, span)?;
    usize::try_from(value).map_err(|_| {
        Diagnostic::error("variant index is too large").with_primary(
            span,
            "the index cannot be represented on this compiler host",
        )
    })
}

pub(super) fn unparenthesized_integer(
    expression: &Node<resolved::Expression>,
) -> Option<&IntegerLiteral> {
    match &expression.kind {
        resolved::Expression::Integer(literal) => Some(literal),
        resolved::Expression::Parenthesized(inner) => unparenthesized_integer(inner),
        _ => None,
    }
}

pub(super) fn is_contextual_integer(expression: &Node<resolved::Expression>) -> bool {
    unparenthesized_integer(expression).is_some_and(|literal| literal.suffix.is_none())
}

pub(super) fn literal_type(suffix: Option<IntegerSuffix>) -> Option<Type> {
    suffix.map(|suffix| match suffix {
        IntegerSuffix::Int8 => Type::Int8,
        IntegerSuffix::Int16 => Type::Int16,
        IntegerSuffix::Int32 => Type::Int32,
        IntegerSuffix::Int64 => Type::Int64,
        IntegerSuffix::UInt8 => Type::UInt8,
        IntegerSuffix::UInt16 => Type::UInt16,
        IntegerSuffix::UInt32 => Type::UInt32,
        IntegerSuffix::UInt64 => Type::UInt64,
    })
}

pub(super) fn is_integer(ty: &Type) -> bool {
    matches!(
        ty,
        Type::Int8
            | Type::Int16
            | Type::Int32
            | Type::Int64
            | Type::UInt8
            | Type::UInt16
            | Type::UInt32
            | Type::UInt64
    )
}

pub(super) fn integer_is_signed(ty: &Type) -> bool {
    matches!(ty, Type::Int8 | Type::Int16 | Type::Int32 | Type::Int64)
}

fn integer_bits(ty: &Type) -> u32 {
    match ty {
        Type::Int8 | Type::UInt8 => 8,
        Type::Int16 | Type::UInt16 => 16,
        Type::Int32 | Type::UInt32 => 32,
        Type::Int64 | Type::UInt64 => 64,
        _ => unreachable!("called only for integer types"),
    }
}

pub(super) fn integer_negative_magnitude(ty: &Type) -> u128 {
    1_u128 << (integer_bits(ty) - 1)
}

fn integer_positive_maximum(ty: &Type) -> u128 {
    if integer_is_signed(ty) {
        integer_negative_magnitude(ty) - 1
    } else {
        (1_u128 << integer_bits(ty)) - 1
    }
}
