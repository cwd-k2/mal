use crate::ast::Node;
use crate::diagnostic::Diagnostic;
use crate::resolve::ast as resolved;
use crate::resolve::{GET_VALUE, NEW_VALUE, PUT_VALUE};
use crate::source::Span;

use super::super::ast::{Expression, ExpressionKind, MemoryPrimitive, Type};
use super::super::{CheckResult, Checker};

impl Checker {
    pub(in crate::check) fn is_buffer_operation_call(
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

    pub(in crate::check) fn check_buffer_operation(
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
                        format!(
                            "this has type `{}`",
                            super::super::types::type_name(&buffer.ty)
                        ),
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
}
