//! How each operation and terminator moves function values between bindings, captures, parameters, results, and
//! buffers.

use crate::closure::ast::Pattern;
use crate::control::ast::{Operation, StateId, Terminator};
use crate::core::ast::BufferOperation;

use super::analysis::{Analysis, Functions, merge};
use super::owner::Owner;

impl Analysis<'_> {
    pub(super) fn operation(&mut self, owner: Owner, pattern: &Pattern, operation: &Operation) {
        match operation {
            Operation::Atom(atom) | Operation::SumInjection { value: atom, .. } => {
                let value = self.atom(owner, atom);
                self.assign(pattern, &value);
            }
            Operation::Product(atoms) => {
                let value = atoms
                    .iter()
                    .flat_map(|atom| self.atom(owner, atom))
                    .collect();
                self.assign(pattern, &value);
            }
            Operation::MakeClosure { function, captures } => {
                for (index, capture) in captures.iter().enumerate() {
                    let value = self.atom(owner, capture);
                    let slot = self.captures.entry((*function, index)).or_default();
                    self.changed |= merge(slot, &value);
                }
                self.assign(pattern, &Functions::from([*function]));
            }
            Operation::Buffer {
                operation: BufferOperation::Get,
                ..
            } => {
                let elements = self.buffer_elements.clone();
                self.assign(pattern, &elements);
            }
            Operation::Buffer { operands, .. } => {
                for operand in operands {
                    let value = self.atom(owner, operand);
                    self.changed |= merge(&mut self.buffer_elements, &value);
                }
            }
            Operation::SymbolLength { .. }
            | Operation::SymbolAt { .. }
            | Operation::Memory { .. }
            | Operation::ExternalCall { .. }
            | Operation::NumericConversion { .. }
            | Operation::PrimitiveUnary { .. }
            | Operation::PrimitiveBinary { .. } => {}
        }
    }

    pub(super) fn terminator(&mut self, site: StateId, owner: Owner, terminator: &Terminator) {
        match terminator {
            Terminator::Return(atom) => {
                let value = self.atom(owner, atom);
                self.add_return(owner, &value);
            }
            Terminator::Jump { target, value } => {
                let value = self.atom(owner, value);
                self.assign_input(*target, &value);
            }
            Terminator::Case { scrutinee, arms } => {
                let value = self.atom(owner, scrutinee);
                for arm in arms {
                    self.assign_input(arm.target, &value);
                }
            }
            Terminator::Call {
                argument, resume, ..
            } => {
                let argument = self.atom(owner, argument);
                let mut result = Functions::new();
                for target in self.reached_targets(site).unwrap_or_default() {
                    self.assign_parameter(target, &argument);
                    result.extend(
                        self.returns
                            .get(&Owner::Function(target))
                            .into_iter()
                            .flatten(),
                    );
                }
                self.assign_input(*resume, &result);
            }
            Terminator::TailCall { argument, .. } => {
                let argument = self.atom(owner, argument);
                for target in self.reached_targets(site).unwrap_or_default() {
                    self.assign_parameter(target, &argument);
                    let result = self.returns.get(&Owner::Function(target)).cloned();
                    if let Some(result) = result {
                        self.add_return(owner, &result);
                    }
                }
            }
            Terminator::Goto(_) | Terminator::PrimitiveBranch { .. } => {}
        }
    }
}
