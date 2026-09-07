use crate::c_emit::syntax::{Block, Statement};
use crate::check::ast::Type;
use crate::closure::ast::{Binding, Block as ClosureBlock, Operation, Pattern};

use super::{BodyEmitter, pattern_type};

mod control;
mod result;

impl BodyEmitter<'_> {
    pub(super) fn emit_block_bindings(&mut self, output: &mut Block, block: &ClosureBlock) {
        for binding in &block.bindings {
            self.emit_binding(output, binding);
        }
    }

    fn emit_binding(&mut self, output: &mut Block, binding: &Binding) {
        let ty = pattern_type(&binding.pattern);
        match &binding.operation {
            Operation::Atom(atom) if matches!(binding.pattern, Pattern::Product { .. }) => {
                let value = self.emit_atom(atom);
                self.emit_pattern_bindings(output, &binding.pattern, value);
            }
            Operation::Case { scrutinee, arms } => {
                self.emit_case(output, &binding.pattern, ty, scrutinee, arms);
            }
            Operation::PrimitiveBranch {
                operator,
                left,
                right,
                otherwise,
                then,
            } => self.emit_primitive_branch(
                output,
                &binding.pattern,
                ty,
                *operator,
                left,
                right,
                otherwise,
                then,
            ),
            Operation::MakeClosure { function, captures } => {
                self.emit_make_closure(output, &binding.pattern, ty, *function, captures);
            }
            Operation::ExternalCall { id, argument } if *ty == Type::Unit => {
                let call = self.emit_external_call(*id, argument);
                output.push(Statement::expression(call));
                self.emit_unit_result(output, &binding.pattern);
            }
            operation => {
                let emitted = self.emit_operation_expression(operation, ty);
                self.emit_simple_result(
                    output,
                    &binding.pattern,
                    ty,
                    emitted.expression,
                    emitted.ownership,
                );
            }
        }
    }

    pub(super) fn emit_block_cleanup(&self, output: &mut Block, block: &ClosureBlock) {
        for binding in block.bindings.iter().rev() {
            self.destroy_pattern_bindings(output, &binding.pattern);
        }
    }
}
