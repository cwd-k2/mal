use crate::ast::Node;
use crate::diagnostic::Diagnostic;
use crate::resolve::ast as resolved;
use crate::resolve::{EDIT_VALUE, GET_VALUE, NEW_VALUE, PACK_VALUE, PUT_VALUE};
use crate::source::Span;

use super::ast::{Expression, ExpressionKind, MemoryPrimitive, Type};
use super::{CheckResult, Checker};

impl Checker {
    pub(super) fn is_buffer_operation_call(
        &self,
        reference: &resolved::ValueReference,
        arguments: &[Node<resolved::Expression>],
    ) -> bool {
        let named_operation = matches!(reference.name.text.as_str(), "new" | "get" | "put");
        if !named_operation {
            return false;
        }
        if matches!(reference.id, NEW_VALUE | GET_VALUE | PUT_VALUE) {
            return true;
        }
        let Some(Node {
            kind: resolved::Expression::Reference(receiver),
            ..
        }) = arguments.first()
        else {
            return false;
        };
        self.value_type(receiver)
            .is_ok_and(|ty| matches!(ty, Type::Buffer(_)))
    }

    pub(super) fn check_buffer_operation(
        &mut self,
        reference: &resolved::ValueReference,
        arguments: &[Node<resolved::Expression>],
        span: Span,
    ) -> CheckResult<Expression> {
        let expected = match reference.name.text.as_str() {
            "new" | "get" => 2,
            "put" => 3,
            _ => unreachable!("caller recognizes Buffer operation identities"),
        };
        if arguments.len() != expected {
            return Err(
                Diagnostic::error("Buffer operation argument arity mismatch")
                    .with_primary(
                        span,
                        format!(
                            "expected {expected} arguments but found {}",
                            arguments.len()
                        ),
                    )
                    .into(),
            );
        }
        let buffer = self.check_expression(&arguments[0], None)?;
        let Type::Buffer(element) = &buffer.ty else {
            return Err(
                Diagnostic::error("Buffer operation requires a Buffer receiver")
                    .with_primary(
                        buffer.span,
                        format!("this has type `{}`", super::types::type_name(&buffer.ty)),
                    )
                    .into(),
            );
        };
        let element = element.clone();
        let (primitive, ty, mut operands) = match reference.name.text.as_str() {
            "new" => (
                MemoryPrimitive::BufferNew,
                Type::USize,
                vec![
                    buffer,
                    self.check_expression(&arguments[1], Some(element.as_ref()))?,
                ],
            ),
            "get" => (
                MemoryPrimitive::BufferGet,
                element.as_ref().clone(),
                vec![
                    buffer,
                    self.check_expression(&arguments[1], Some(&Type::USize))?,
                ],
            ),
            "put" => (
                MemoryPrimitive::BufferPut,
                Type::Unit,
                vec![
                    buffer,
                    self.check_expression(&arguments[1], Some(&Type::USize))?,
                ],
            ),
            _ => unreachable!("caller recognizes Buffer operation identities"),
        };
        if reference.name.text == "put" {
            operands.push(self.check_expression(&arguments[2], Some(element.as_ref()))?);
        }
        Ok(Expression {
            kind: ExpressionKind::Memory {
                primitive,
                operands,
            },
            ty,
            span,
        })
    }

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
    Type::Function {
        parameter: Type::Buffer(element.clone().into()).into(),
        result: Type::Unit.into(),
    }
}
