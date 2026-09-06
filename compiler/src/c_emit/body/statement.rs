use crate::check::ast::Type;
use crate::closure::ast::{Binding, Block, Operation, Pattern};

use super::{BodyEmitter, pattern_type};

mod control;
mod result;

impl BodyEmitter<'_> {
    pub(super) fn emit_block_bindings(
        &mut self,
        output: &mut String,
        block: &Block,
        indent: usize,
    ) {
        for binding in &block.bindings {
            self.emit_binding(output, binding, indent);
        }
    }

    fn emit_binding(&mut self, output: &mut String, binding: &Binding, indent: usize) {
        let ty = pattern_type(&binding.pattern);
        match &binding.operation {
            Operation::Atom(atom) if matches!(binding.pattern, Pattern::Product { .. }) => {
                let value = self.emit_atom(atom);
                self.emit_pattern_bindings(output, &binding.pattern, &value, indent);
            }
            Operation::Case { scrutinee, arms } => {
                self.emit_case(output, &binding.pattern, ty, scrutinee, arms, indent);
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
                indent,
            ),
            Operation::MakeClosure { function, captures } => {
                self.emit_make_closure(output, &binding.pattern, ty, *function, captures, indent);
            }
            Operation::ExternalCall { id, argument } if *ty == Type::Unit => {
                let call = self.emit_external_call(*id, argument);
                c_line!(output, indent, "{call};");
                self.emit_unit_result(output, &binding.pattern, indent);
            }
            operation => {
                let expression = self.emit_operation_expression(operation, ty);
                self.emit_simple_result(output, &binding.pattern, ty, &expression, indent);
            }
        }
    }
}
