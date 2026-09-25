use crate::resolve::ast as resolved;
use crate::resolve::{COPY_VALUE, FILL_VALUE, GET_VALUE, INTO_VALUE, NEW_VALUE, PUT_VALUE};
use mal_syntax::ast::Node;
use mal_syntax::diagnostic::Diagnostic;
use mal_syntax::source::Span;

use super::super::ast::{Expression, ExpressionKind, MemoryPrimitive, Type};
use super::super::{CheckResult, Checker};

impl Checker {
    pub(crate) fn check_memory_operation(
        &mut self,
        reference: &resolved::ValueReference,
        arguments: &[Node<resolved::Expression>],
        span: Span,
    ) -> CheckResult<Expression> {
        let expected = match reference.id {
            NEW_VALUE | GET_VALUE => 2,
            PUT_VALUE => 3,
            INTO_VALUE | FILL_VALUE => 4,
            COPY_VALUE => 5,
            _ => unreachable!("caller recognizes predefined memory operations"),
        };
        if arguments.len() != expected {
            return Err(
                Diagnostic::error("memory operation argument arity mismatch")
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
        let receiver = self.check_expression(&arguments[0], None)?;
        match (&receiver.ty, reference.id) {
            (Type::Buffer(element), NEW_VALUE) => {
                let element = element.clone();
                let (receiver, value) =
                    self.check_after(receiver, &arguments[1], Some(element.as_ref()))?;
                Ok(memory(
                    MemoryPrimitive::BufferNew,
                    vec![receiver, value],
                    Type::USize,
                    span,
                ))
            }
            (Type::Buffer(element), GET_VALUE) => {
                let element = element.clone();
                let (receiver, index) =
                    self.check_after(receiver, &arguments[1], Some(&Type::USize))?;
                Ok(memory(
                    MemoryPrimitive::BufferGet,
                    vec![receiver, index],
                    (*element).clone(),
                    span,
                ))
            }
            (Type::Buffer(element), PUT_VALUE) => {
                let element = element.clone();
                let (receiver, index) =
                    self.check_after(receiver, &arguments[1], Some(&Type::USize))?;
                let (index, value) =
                    self.check_after(index, &arguments[2], Some(element.as_ref()))?;
                Ok(memory(
                    MemoryPrimitive::BufferPut,
                    vec![receiver, index, value],
                    Type::Unit,
                    span,
                ))
            }
            (Type::Buffer(element), FILL_VALUE) => {
                let element = element.clone();
                let (receiver, offset) =
                    self.check_after(receiver, &arguments[1], Some(&Type::USize))?;
                let (offset, length) =
                    self.check_after(offset, &arguments[2], Some(&Type::USize))?;
                let (length, value) =
                    self.check_after(length, &arguments[3], Some(element.as_ref()))?;
                Ok(memory(
                    MemoryPrimitive::BufferFill,
                    vec![receiver, offset, length, value],
                    Type::Unit,
                    span,
                ))
            }
            (Type::Buffer(element), COPY_VALUE) => {
                let element = element.clone();
                let source_type = Type::Buffer(element.clone());
                let (receiver, destination_offset) =
                    self.check_after(receiver, &arguments[1], Some(&Type::USize))?;
                let (destination_offset, source) =
                    self.check_after(destination_offset, &arguments[2], Some(&source_type))?;
                let (source, source_offset) =
                    self.check_after(source, &arguments[3], Some(&Type::USize))?;
                let (source_offset, length) =
                    self.check_after(source_offset, &arguments[4], Some(&Type::USize))?;
                Ok(memory(
                    MemoryPrimitive::BufferCopy,
                    vec![receiver, destination_offset, source, source_offset, length],
                    Type::Unit,
                    span,
                ))
            }
            (Type::Buffer(element), INTO_VALUE) => {
                super::ensure_copyable_element(element, receiver.span)?;
                let (receiver, address) =
                    self.check_after(receiver, &arguments[1], Some(&Type::Address))?;
                let (address, offset) =
                    self.check_after(address, &arguments[2], Some(&Type::USize))?;
                let (offset, length) =
                    self.check_after(offset, &arguments[3], Some(&Type::USize))?;
                Ok(memory(
                    MemoryPrimitive::BufferIntoAddress,
                    vec![receiver, address, offset, length],
                    Type::Unit,
                    span,
                ))
            }
            _ => Err(
                Diagnostic::error("memory operation is not defined for this receiver")
                    .with_primary(
                        receiver.span,
                        format!(
                            "this has type `{}`",
                            super::super::types::type_name(&receiver.ty)
                        ),
                    )
                    .into(),
            ),
        }
    }
}

fn memory(
    primitive: MemoryPrimitive,
    operands: Vec<Expression>,
    ty: Type,
    span: Span,
) -> Expression {
    Expression {
        kind: ExpressionKind::Memory {
            primitive,
            operands,
        },
        ty,
        span,
    }
}
