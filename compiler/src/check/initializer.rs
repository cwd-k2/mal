use crate::ast::{Node, UnaryOperator};
use crate::diagnostic::Diagnostic;
use crate::resolve::ast::{self as resolved, FALSE_VALUE, TRUE_VALUE};

use super::Checker;

impl Checker {
    pub(super) fn check_top_level_initializer(
        &self,
        expression: &Node<resolved::Expression>,
    ) -> Result<(), Diagnostic> {
        if is_top_level_initializer(expression) {
            Ok(())
        } else {
            Err(
                Diagnostic::error("unsupported top-level initializer").with_primary(
                    expression.span,
                    "top-level values must be closed literals, storage sizes, sums, or lambdas",
                ),
            )
        }
    }
}

fn is_top_level_initializer(expression: &Node<resolved::Expression>) -> bool {
    match &expression.kind {
        resolved::Expression::Integer(_)
        | resolved::Expression::Float(_)
        | resolved::Expression::Byte(_)
        | resolved::Expression::Symbol(_)
        | resolved::Expression::StorageSize(_)
        | resolved::Expression::Unit => true,
        resolved::Expression::Reference(reference) => {
            matches!(reference.id, FALSE_VALUE | TRUE_VALUE)
                || super::memory::memory_primitive(reference.id).is_some()
        }
        resolved::Expression::Parenthesized(inner) => is_top_level_initializer(inner),
        resolved::Expression::Product(elements) => elements.iter().all(is_top_level_initializer),
        resolved::Expression::SumInjection { value, .. }
        | resolved::Expression::Conversion { value, .. } => is_top_level_initializer(value),
        resolved::Expression::Lambda(_) => true,
        resolved::Expression::Unary {
            operator, operand, ..
        } => {
            operator.kind == UnaryOperator::Negate
                && matches!(
                    operand.kind,
                    resolved::Expression::Integer(_) | resolved::Expression::Float(_)
                )
        }
        _ => false,
    }
}
