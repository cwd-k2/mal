use crate::check::ast::Type;
use crate::control::ast::Operation;
use crate::core::ast::UnaryPrimitive;

use super::scalar::{arithmetic_instruction, scalar_type};
use super::{EmittedValue, FunctionEmitter};

impl FunctionEmitter<'_> {
    pub(super) fn emit_operation(
        &mut self,
        operation: &Operation,
        result_type: Option<&Type>,
    ) -> Option<Option<EmittedValue>> {
        match operation {
            Operation::Atom(atom) => self.atom(atom).map(Some),
            Operation::PrimitiveUnary { operator, operand } => {
                let operand = self.atom(operand)?;
                let scalar = scalar_type(&operand.ty)?;
                let register = self.register();
                let instruction = match operator {
                    UnaryPrimitive::Negate if scalar.floating => {
                        format!("fneg {} {}", scalar.llvm, operand.representation)
                    }
                    UnaryPrimitive::Negate => {
                        format!("sub {} 0, {}", scalar.llvm, operand.representation)
                    }
                    UnaryPrimitive::BitwiseNot if !scalar.floating => {
                        format!("xor {} {}, -1", scalar.llvm, operand.representation)
                    }
                    UnaryPrimitive::BitwiseNot => return None,
                };
                self.line(format!("  {register} = {instruction}"));
                Some(Some(EmittedValue {
                    ty: operand.ty,
                    representation: register,
                    owned: false,
                }))
            }
            Operation::PrimitiveBinary {
                operator,
                left,
                right,
            } => {
                if *operator == crate::core::ast::BinaryPrimitive::Add
                    && left.ty == Type::Symbol
                    && right.ty == Type::Symbol
                {
                    return self.emit_symbol_concatenate(left, right).map(Some);
                }
                let left = self.atom(left)?;
                let right = self.atom(right)?;
                if left.ty != right.ty {
                    return None;
                }
                let scalar = scalar_type(&left.ty)?;
                let instruction = arithmetic_instruction(*operator, scalar)?;
                let register = self.register();
                self.line(format!(
                    "  {register} = {instruction} {} {}, {}",
                    scalar.llvm, left.representation, right.representation
                ));
                Some(Some(EmittedValue {
                    ty: left.ty,
                    representation: register,
                    owned: false,
                }))
            }
            Operation::NumericConversion { operand } => {
                let operand = self.atom(operand)?;
                let source = scalar_type(&operand.ty)?;
                let result_type = result_type?.clone();
                let target = scalar_type(&result_type)?;
                if source.floating == target.floating && source.bits == target.bits {
                    return Some(Some(EmittedValue {
                        ty: result_type,
                        representation: operand.representation,
                        owned: false,
                    }));
                }
                let instruction = if source.floating && target.floating {
                    if source.bits > target.bits {
                        "fptrunc"
                    } else {
                        "fpext"
                    }
                } else if source.floating {
                    if target.signed { "fptosi" } else { "fptoui" }
                } else if target.floating {
                    if source.signed { "sitofp" } else { "uitofp" }
                } else if source.bits > target.bits {
                    "trunc"
                } else if source.signed {
                    "sext"
                } else {
                    "zext"
                };
                let register = self.register();
                self.line(format!(
                    "  {register} = {instruction} {} {} to {}",
                    source.llvm, operand.representation, target.llvm
                ));
                Some(Some(EmittedValue {
                    ty: result_type,
                    representation: register,
                    owned: false,
                }))
            }
            Operation::SymbolLength { value } => self.emit_symbol_length(value).map(Some),
            Operation::SymbolAt { argument } => self.emit_symbol_at(argument).map(Some),
            Operation::ExternalCall { id, argument } => self
                .emit_external_call(*id, argument, result_type?)
                .map(Some),
            Operation::Memory {
                primitive,
                argument,
            } => self
                .emit_memory(*primitive, argument, result_type?)
                .map(Some),
            Operation::Product(elements) => self.emit_product(elements, result_type?).map(Some),
            Operation::SumInjection { index, value } => {
                self.emit_sum(*index, value, result_type?).map(Some)
            }
            _ => None,
        }
    }
}
