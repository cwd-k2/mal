use crate::resolve::ast as resolved;
use crate::resolve::{COPY_VALUE, FILL_VALUE, GET_VALUE, INTO_VALUE, NEW_VALUE, PUT_VALUE};
use mal_syntax::ast::Node;
use mal_syntax::diagnostic::Diagnostic;
use mal_syntax::source::Span;

use super::super::ast::{Expression, ExpressionKind, MemoryPrimitive, Type};
use super::super::{CheckFailure, CheckResult, Checker};

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
        // Every operation has at least two operands, so an abrupt receiver leaves the next one unreachable.
        let receiver = self.check_before(&arguments[0], None, arguments[1].span)?;
        let receiver_view = super::super::types::representation_view(&receiver.ty, span.file());
        let Type::Buffer(element) = receiver_view else {
            return Err(
                Diagnostic::error("memory operation is not defined for this receiver")
                    .with_primary(
                        receiver.span,
                        format!(
                            "this has type `{}`",
                            super::super::types::type_name(&receiver.ty)
                        ),
                    )
                    .into(),
            );
        };
        let element = element.as_ref().clone();
        let (primitive, operand_types, result) = match reference.id {
            NEW_VALUE => (MemoryPrimitive::BufferNew, vec![element], Type::USize),
            GET_VALUE => (MemoryPrimitive::BufferGet, vec![Type::USize], element),
            PUT_VALUE => (
                MemoryPrimitive::BufferPut,
                vec![Type::USize, element],
                Type::Unit,
            ),
            FILL_VALUE => (
                MemoryPrimitive::BufferFill,
                vec![Type::USize, Type::USize, element],
                Type::Unit,
            ),
            COPY_VALUE => (
                MemoryPrimitive::BufferCopy,
                vec![
                    Type::USize,
                    Type::Buffer(element.into()),
                    Type::USize,
                    Type::USize,
                ],
                Type::Unit,
            ),
            INTO_VALUE => {
                super::ensure_copyable_element(&element, receiver.span)?;
                (
                    MemoryPrimitive::BufferIntoAddress,
                    vec![Type::Address, Type::USize, Type::USize],
                    Type::Unit,
                )
            }
            _ => unreachable!("caller recognizes predefined memory operations"),
        };
        let operands = self.check_operands_after(receiver, &arguments[1..], &operand_types)?;
        Ok(memory(primitive, operands, result, span))
    }

    /// Checks operands in evaluation order after an already checked first operand. Like a product, an abrupt operand
    /// makes the next one unreachable, and a final abrupt operand keeps every value evaluated before it.
    fn check_operands_after(
        &mut self,
        first: Expression,
        operands: &[Node<resolved::Expression>],
        expected: &[Type],
    ) -> CheckResult<Vec<Expression>> {
        let mut checked = Vec::with_capacity(operands.len() + 1);
        checked.push(first);
        for (index, (operand, expected)) in operands.iter().zip(expected).enumerate() {
            let value = match operands.get(index + 1) {
                Some(next) => self.check_before(operand, Some(expected), next.span)?,
                None => match self.check_expression(operand, Some(expected)) {
                    Err(CheckFailure::Abrupt(abrupt)) => {
                        return Err(CheckFailure::Abrupt(Box::new(
                            (*abrupt).preceded_by(checked),
                        )));
                    }
                    result => result?,
                },
            };
            checked.push(value);
        }
        Ok(checked)
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
