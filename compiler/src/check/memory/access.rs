use crate::ast::Node;
use crate::diagnostic::Diagnostic;
use crate::resolve::ast as resolved;
use crate::source::Span;

use super::super::ast::{Expression, ExpressionKind, MemoryPrimitive, Type};
use super::super::{CheckResult, Checker};

impl Checker {
    pub(crate) fn check_memory_operation(
        &mut self,
        reference: &resolved::ValueReference,
        arguments: &[Node<resolved::Expression>],
        span: Span,
    ) -> CheckResult<Expression> {
        let expected = match reference.name.text.as_str() {
            "get" | "new" => 2,
            "put" => 3,
            "into" | "fill" => 4,
            "copy" => 5,
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
        match (&receiver.ty, reference.name.text.as_str()) {
            (Type::Buffer(element), "new") => {
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
            (Type::Buffer(element), "get") => {
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
            (Type::Buffer(element), "put") => {
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
            (Type::Buffer(element), "fill") => {
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
            (Type::Buffer(element), "copy") => {
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
            (Type::Buffer(_), "into") => {
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
