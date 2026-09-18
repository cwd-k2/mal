use crate::ast::{BinaryOperator, UnaryOperator};
use crate::diagnostic::Diagnostic;
use crate::resolve::ast::{FALSE_VALUE, TRUE_VALUE};

use super::Checker;
use super::ast::{Expression, ExpressionKind, Type};
use super::float::is_float;
use super::integer::is_integer;

impl Checker {
    pub(super) fn check_top_level_initializer(
        &self,
        expression: &Expression,
    ) -> Result<(), Diagnostic> {
        if is_top_level_initializer(expression, &self.external_values) {
            Ok(())
        } else {
            Err(
                Diagnostic::error("invalid top-level initializer").with_primary(
                    expression.span,
                    "top-level values must be closed, constant expressions or functions",
                ),
            )
        }
    }
}

fn is_top_level_initializer(
    expression: &Expression,
    external_values: &std::collections::HashSet<crate::resolve::ast::ValueId>,
) -> bool {
    match &expression.kind {
        ExpressionKind::Integer(_)
        | ExpressionKind::Float(_)
        | ExpressionKind::Symbol(_)
        | ExpressionKind::StorageSize(_)
        | ExpressionKind::Unit => true,
        ExpressionKind::Reference(reference) => {
            matches!(reference.id, FALSE_VALUE | TRUE_VALUE)
                || external_values.contains(&reference.id)
        }
        ExpressionKind::Parenthesized(inner) => is_top_level_initializer(inner, external_values),
        ExpressionKind::Product(elements) => elements
            .iter()
            .all(|element| is_top_level_initializer(element, external_values)),
        ExpressionKind::NumericConversion { value } => {
            is_top_level_initializer(value, external_values)
        }
        ExpressionKind::SumInjection { value, .. } => {
            is_top_level_initializer(value, external_values)
        }
        ExpressionKind::Lambda(_) => true,
        ExpressionKind::Unary {
            operator, operand, ..
        } => {
            operator.kind == UnaryOperator::Negate
                && is_constant_numeric(&operand.ty)
                && is_top_level_initializer(operand, external_values)
        }
        ExpressionKind::Binary {
            operator,
            left,
            right,
        } => {
            is_constant_binary(operator.kind, &left.ty, &right.ty)
                && is_top_level_initializer(left, external_values)
                && is_top_level_initializer(right, external_values)
        }
        _ => false,
    }
}

fn is_constant_numeric(ty: &Type) -> bool {
    is_integer(ty) || is_float(ty)
}

fn is_constant_binary(operator: BinaryOperator, left: &Type, right: &Type) -> bool {
    if left != right || !is_integer(left) {
        return false;
    }
    matches!(
        operator,
        BinaryOperator::Multiply | BinaryOperator::Add | BinaryOperator::Subtract
    )
}
