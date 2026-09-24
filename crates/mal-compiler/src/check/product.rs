use crate::resolve::ast as resolved;
use mal_syntax::ast::Node;
use mal_syntax::source::Span;

use super::ast::{Expression, ExpressionKind, Type};
use super::{CheckFailure, CheckResult, Checker};

impl Checker {
    pub(super) fn check_product(
        &mut self,
        elements: &[Node<resolved::Expression>],
        span: Span,
        expected: Option<&Type>,
    ) -> CheckResult<Expression> {
        let expected_elements = match expected {
            Some(Type::Product(expected_elements)) if elements.len() == expected_elements.len() => {
                Some(expected_elements)
            }
            _ => None,
        };
        let mut checked = Vec::with_capacity(elements.len());
        for (index, element) in elements.iter().enumerate() {
            match self.check_expression(
                element,
                expected_elements.and_then(|elements| elements.get(index)),
            ) {
                Ok(value) => checked.push(value),
                Err(CheckFailure::Abrupt(_)) if index + 1 < elements.len() => {
                    return Err(mal_syntax::diagnostic::Diagnostic::error(
                        "unreachable expression after abrupt completion",
                    )
                    .with_primary(
                        elements[index + 1].span,
                        "this expression cannot be reached",
                    )
                    .into());
                }
                Err(CheckFailure::Abrupt(abrupt)) => {
                    return Err(CheckFailure::Abrupt(Box::new(
                        (*abrupt).preceded_by(checked),
                    )));
                }
                Err(error) => return Err(error),
            }
        }
        let ty = Type::Product(checked.iter().map(|element| element.ty.clone()).collect());
        super::types::ensure_representable(&ty, span)?;
        Ok(Expression {
            ty,
            kind: ExpressionKind::Product(checked),
            span,
        })
    }
}
