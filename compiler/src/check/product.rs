use crate::ast::Node;
use crate::diagnostic::Diagnostic;
use crate::resolve::ast as resolved;
use crate::source::Span;

use super::Checker;
use super::ast::{Expression, ExpressionKind, Type};

impl Checker {
    pub(super) fn check_product(
        &mut self,
        elements: &[Node<resolved::Expression>],
        span: Span,
        expected: Option<&Type>,
    ) -> Result<Expression, Diagnostic> {
        let expected_elements = match expected {
            Some(Type::Product(expected_elements)) if elements.len() == expected_elements.len() => {
                Some(expected_elements)
            }
            _ => None,
        };
        let elements = elements
            .iter()
            .enumerate()
            .map(|(index, element)| {
                self.check_expression(
                    element,
                    expected_elements.and_then(|elements| elements.get(index)),
                )
            })
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Expression {
            ty: Type::Product(elements.iter().map(|element| element.ty.clone()).collect()),
            kind: ExpressionKind::Product(elements),
            span,
        })
    }
}
