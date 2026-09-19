use crate::ast::Node;
use crate::diagnostic::Diagnostic;
use crate::resolve::ast as resolved;
use crate::resolve::{BULK_VALUE, EDIT_VALUE, PACK_VALUE};
use crate::source::Span;

use super::super::ast::{Expression, ExpressionKind, PackedBuild, Type};
use super::super::{CheckResult, Checker};
use super::callback_type;

impl Checker {
    pub(in crate::check) fn check_packed_build(
        &mut self,
        reference: &resolved::ValueReference,
        type_arguments: &[Node<resolved::TypeExpression>],
        arguments: &[Node<resolved::Expression>],
        span: Span,
    ) -> CheckResult<Expression> {
        let [type_argument] = type_arguments else {
            return Err(
                Diagnostic::error("Packed intrinsic type argument arity mismatch")
                    .with_primary(
                        reference.name.span,
                        format!("expected 1 argument but found {}", type_arguments.len()),
                    )
                    .into(),
            );
        };
        let element = self.expand_type(type_argument)?;
        if !super::super::types::satisfies_representable_requirement(
            &element,
            &self.active_requirements,
        ) {
            return Err(Diagnostic::error(
                "Packed intrinsic requires a Representable element type",
            )
            .with_primary(
                type_argument.span,
                format!(
                    "`{}` has no available canonical memory representation",
                    super::super::types::type_name(&element)
                ),
            )
            .into());
        }

        let packed = Type::Packed(element.clone().into());
        let callback_type = callback_type(&element);
        let (build, callback) = match (reference.id, arguments) {
            (PACK_VALUE, [callback]) => (
                PackedBuild::Pack,
                self.check_expression(callback, Some(&callback_type))?,
            ),
            (BULK_VALUE, [_, _]) => {
                let parameter = Type::Product(vec![Type::USize, callback_type.clone()].into());
                let checked = self.check_product(arguments, span, Some(&parameter))?;
                let ExpressionKind::Product(mut elements) = checked.kind else {
                    unreachable!("two checked arguments form a product")
                };
                let callback = elements.pop().expect("bulk has a callback argument");
                let capacity = elements.pop().expect("bulk has a capacity argument");
                (
                    PackedBuild::Bulk {
                        capacity: Box::new(capacity),
                    },
                    callback,
                )
            }
            (EDIT_VALUE, [_, _]) => {
                let parameter = Type::Product(vec![packed.clone(), callback_type.clone()].into());
                let checked = self.check_product(arguments, span, Some(&parameter))?;
                let ExpressionKind::Product(mut elements) = checked.kind else {
                    unreachable!("two checked arguments form a product")
                };
                let callback = elements.pop().expect("edit has a callback argument");
                let source = elements.pop().expect("edit has a source argument");
                (
                    PackedBuild::Edit {
                        source: Box::new(source),
                    },
                    callback,
                )
            }
            (PACK_VALUE, _) => {
                return Err(argument_arity_error("pack", 1, arguments.len(), span).into());
            }
            (BULK_VALUE, _) => {
                return Err(argument_arity_error("bulk", 2, arguments.len(), span).into());
            }
            (EDIT_VALUE, _) => {
                return Err(argument_arity_error("edit", 2, arguments.len(), span).into());
            }
            _ => unreachable!("caller recognizes Packed intrinsic identities"),
        };
        Ok(Expression {
            kind: ExpressionKind::PackedBuild {
                build,
                callback: Box::new(callback),
                element,
            },
            ty: packed,
            span,
        })
    }
}

fn argument_arity_error(name: &str, expected: usize, found: usize, span: Span) -> Diagnostic {
    Diagnostic::error(format!("{name} argument arity mismatch")).with_primary(
        span,
        format!("expected {expected} arguments but found {found}"),
    )
}
