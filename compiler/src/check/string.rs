use crate::ast::Node;
use crate::diagnostic::Diagnostic;
use crate::resolve::ast::{self as resolved, BYTE_AT_VALUE, BYTE_LENGTH_VALUE, ValueId};
use crate::source::Span;

use super::Checker;
use super::ast::{Expression, ExpressionKind, Type};

impl Checker {
    pub(super) fn check_string_call(
        &mut self,
        primitive: ValueId,
        arguments: &[Node<resolved::Expression>],
        span: Span,
    ) -> Option<Result<Expression, Diagnostic>> {
        match primitive {
            BYTE_LENGTH_VALUE => Some(self.check_argument(arguments, &Type::String, span).map(
                |value| Expression {
                    kind: ExpressionKind::StringLength {
                        value: Box::new(value),
                    },
                    ty: Type::UInt64,
                    span,
                },
            )),
            BYTE_AT_VALUE => {
                let parameter = Type::Product(vec![Type::String, Type::UInt64]);
                Some(
                    self.check_argument(arguments, &parameter, span)
                        .map(|argument| Expression {
                            kind: ExpressionKind::StringAt {
                                argument: Box::new(argument),
                            },
                            ty: Type::UInt8,
                            span,
                        }),
                )
            }
            _ => None,
        }
    }
}
