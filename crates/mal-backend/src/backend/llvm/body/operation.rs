use crate::control::ast::Operation;
use mal_frontend::check::ast::Type;

use super::{EmittedValue, FunctionEmitter};
use crate::execution::ownership::BindingOperand;

impl FunctionEmitter<'_> {
    /// Emits one control binding operation and returns its materialized result.
    ///
    /// `None` rejects an inconsistent execution plan, unsupported target representation, ownership
    /// mismatch, or nested syntax emission failure. Even `Unit` has an `EmittedValue`, so successful
    /// operations never need a second sentinel for an absent result.
    pub(super) fn emit_operation(
        &mut self,
        site: crate::control::ast::StateId,
        binding: usize,
        operation: &Operation,
        result_type: Option<&Type>,
        symbol_concat: super::super::optimization::SymbolConcatMode,
    ) -> Option<EmittedValue> {
        match operation {
            Operation::Atom(atom) => self
                .prepare_atom_for_use(
                    atom,
                    self.ownership.binding_operand_use(
                        site,
                        binding,
                        BindingOperand::Atom,
                        atom,
                    )?,
                )
                .and_then(|prepared| {
                    self.commit_consumes(&prepared)?;
                    Some(prepared.value)
                }),
            Operation::MakeClosure { function, captures } => {
                self.emit_make_closure(site, binding, function, captures, result_type)
            }
            Operation::PrimitiveUnary { operator, operand } => {
                self.emit_primitive_unary(site, binding, operator, operand)
            }
            Operation::PrimitiveBinary {
                operator,
                left,
                right,
            } => self.emit_primitive_binary(site, binding, operator, left, right, result_type),
            Operation::NumericConversion { operand } => {
                self.emit_numeric_conversion(site, binding, operand, result_type)
            }
            Operation::Symbol {
                primitive,
                operands,
            } => {
                for (index, operand) in operands.iter().enumerate() {
                    self.require_binding_borrow(
                        site,
                        binding,
                        BindingOperand::SymbolOperand(index),
                        operand,
                    )?;
                }
                use mal_frontend::check::ast::SymbolPrimitive;
                match (primitive, operands.as_slice()) {
                    (SymbolPrimitive::Length, [value]) => self.emit_symbol_length(value),
                    (SymbolPrimitive::ByteAt, [symbol, index]) => {
                        self.emit_symbol_at(symbol, index)
                    }
                    (SymbolPrimitive::Concatenate, [left, right]) => {
                        self.emit_symbol_concatenate(left, right, symbol_concat)
                    }
                    (SymbolPrimitive::Prefix | SymbolPrimitive::Suffix, [symbol, index]) => {
                        self.emit_symbol_partition(symbol, index, *primitive)
                    }
                    (SymbolPrimitive::Equal | SymbolPrimitive::NotEqual, [left, right]) => {
                        self.emit_symbol_equality(left, right, *primitive)
                    }
                    _ => None,
                }
            }
            Operation::ExternalCall { id, argument } => {
                self.require_binding_borrow(
                    site,
                    binding,
                    BindingOperand::ExternalArgument,
                    argument,
                )?;
                self.emit_external_call(*id, argument, result_type?)
            }
            Operation::Memory {
                primitive,
                operands,
            } => {
                if let [operand] = operands.as_slice()
                    && self.optimizations.transfers_byte_conversion(site, binding)
                    && self.atom_has_slot(operand)
                {
                    return self.emit_byte_conversion_transfer(*primitive, operand, result_type?);
                }
                let effects = operands
                    .iter()
                    .enumerate()
                    .map(|(index, operand)| {
                        self.ownership.binding_operand_use(
                            site,
                            binding,
                            BindingOperand::MemoryOperand(index),
                            operand,
                        )
                    })
                    .collect::<Option<Vec<_>>>()?;
                let prepared = if let [operand] = operands.as_slice() {
                    self.prepare_atom_for_use(operand, effects[0])?
                } else {
                    let argument_type =
                        Type::Product(operands.iter().map(|operand| operand.ty.clone()).collect());
                    self.emit_product(operands, &argument_type, &effects)?
                };
                let result = self.emit_memory(*primitive, &prepared.value, result_type?)?;
                self.commit_consumes(&prepared)?;
                Some(result)
            }
            Operation::Buffer {
                operation,
                element,
                operands,
            } => {
                let prepared = operands
                    .iter()
                    .enumerate()
                    .map(|(index, operand)| {
                        let effect = self.ownership.binding_operand_use(
                            site,
                            binding,
                            BindingOperand::BufferOperand(index),
                            operand,
                        )?;
                        self.prepare_atom_for_use(operand, effect)
                    })
                    .collect::<Option<Vec<_>>>()?;
                let values = prepared
                    .iter()
                    .map(|prepared| prepared.value.clone())
                    .collect::<Vec<_>>();
                let result = self.emit_buffer(*operation, element, &values, result_type?)?;
                for prepared in &prepared {
                    self.commit_consumes(prepared)?;
                }
                Some(result)
            }
            Operation::Product(elements) => {
                let effects = elements
                    .iter()
                    .enumerate()
                    .map(|(index, atom)| {
                        self.ownership.binding_operand_use(
                            site,
                            binding,
                            BindingOperand::ProductElement(index),
                            atom,
                        )
                    })
                    .collect::<Option<Vec<_>>>()?;
                let prepared = self.emit_product(elements, result_type?, &effects)?;
                self.commit_consumes(&prepared)?;
                Some(prepared.value)
            }
            Operation::SumInjection { index, value } => {
                let effect = self.ownership.binding_operand_use(
                    site,
                    binding,
                    BindingOperand::SumValue,
                    value,
                )?;
                let prepared = self.emit_sum(*index, value, result_type?, effect)?;
                self.commit_consumes(&prepared)?;
                Some(prepared.value)
            }
        }
    }
}
