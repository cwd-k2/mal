use crate::ast::Node;
use crate::diagnostic::Diagnostic;
use crate::resolve::ast as resolved;
use crate::resolve::{EDIT_VALUE, PACK_VALUE};
use crate::source::Span;

use super::ast::{Expression, ExpressionKind, Type};
use super::{CheckResult, Checker};

impl Checker {
    pub(super) fn check_packed_build(
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
        if !super::types::satisfies_representable_requirement(&element, &self.active_requirements) {
            return Err(Diagnostic::error(
                "Packed intrinsic requires a Representable element type",
            )
            .with_primary(
                type_argument.span,
                format!(
                    "`{}` has no available canonical memory representation",
                    super::types::type_name(&element)
                ),
            )
            .into());
        }

        let packed = Type::Packed(element.clone().into());
        let callback_type = callback_type(&element);
        let (source, callback) = match (reference.id, arguments) {
            (PACK_VALUE, [callback]) => {
                (None, self.check_expression(callback, Some(&callback_type))?)
            }
            (EDIT_VALUE, [_, _]) => {
                let parameter = Type::Product(vec![packed.clone(), callback_type.clone()].into());
                let checked = self.check_product(arguments, span, Some(&parameter))?;
                let ExpressionKind::Product(mut elements) = checked.kind else {
                    unreachable!("two checked arguments form a product")
                };
                let callback = elements.pop().expect("edit has a callback argument");
                let source = elements.pop().expect("edit has a source argument");
                (Some(Box::new(source)), callback)
            }
            (PACK_VALUE, _) => {
                return Err(Diagnostic::error("pack argument arity mismatch")
                    .with_primary(
                        span,
                        format!("expected 1 argument but found {}", arguments.len()),
                    )
                    .into());
            }
            (EDIT_VALUE, _) => {
                return Err(Diagnostic::error("edit argument arity mismatch")
                    .with_primary(
                        span,
                        format!("expected 2 arguments but found {}", arguments.len()),
                    )
                    .into());
            }
            _ => unreachable!("caller recognizes Packed intrinsic identities"),
        };
        Ok(Expression {
            kind: ExpressionKind::PackedBuild {
                source,
                callback: Box::new(callback),
                element,
            },
            ty: packed,
            span,
        })
    }
}

fn callback_type(element: &Type) -> Type {
    let new = Type::Function {
        parameter: element.clone().into(),
        result: Type::USize.into(),
    };
    let get = Type::Function {
        parameter: Type::USize.into(),
        result: element.clone().into(),
    };
    let put = Type::Function {
        parameter: Type::Product(vec![Type::USize, element.clone()].into()).into(),
        result: Type::Unit.into(),
    };
    Type::Function {
        parameter: Type::Product(vec![new, get, put].into()).into(),
        result: Type::Unit.into(),
    }
}
