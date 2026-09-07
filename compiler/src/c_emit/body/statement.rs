use crate::c_emit::syntax::{Block, Expr, Initializer, Statement};
use crate::c_emit::types::is_bool;
use crate::check::ast::Type;
use crate::closure::ast::{Binding, Block as ClosureBlock, Operation, Pattern};
use crate::core::ast::BinaryPrimitive;

use super::{BodyEmitter, ResultOwnership, pattern_type};

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
        if let (
            Pattern::Binding { id, .. },
            Operation::Atom(crate::closure::ast::Atom {
                kind:
                    crate::closure::ast::AtomKind::Reference(crate::closure::ast::Reference::Binding(
                        source,
                    )),
                ..
            }),
        ) = (&binding.pattern, &binding.operation)
            && self.closure_uses.is_direct_alias(*id, *source)
        {
            return;
        }
        match &binding.operation {
            Operation::Atom(atom)
                if matches!(binding.pattern, Pattern::Product { .. })
                    && !self.ownership.can_transfer(atom) =>
            {
                let value = self.emit_atom(atom);
                self.emit_pattern_bindings(output, &binding.pattern, value);
            }
            Operation::Atom(atom) => {
                let mut transfers = Vec::new();
                let value = self.materialize_atom(atom, &mut transfers);
                self.emit_simple_result_with_transfers(
                    output,
                    &binding.pattern,
                    ty,
                    value,
                    ResultOwnership::Owned,
                    &transfers,
                );
            }
            Operation::Product(elements) => {
                let mut transfers = Vec::new();
                let fields = elements.iter().enumerate().map(|(index, element)| {
                    Initializer::designated(
                        format!("field_{index}"),
                        self.materialize_atom(element, &mut transfers),
                    )
                });
                let value = Expr::compound_literal(self.types.c_type(ty), fields);
                self.emit_simple_result_with_transfers(
                    output,
                    &binding.pattern,
                    ty,
                    value,
                    ResultOwnership::Owned,
                    &transfers,
                );
            }
            Operation::SumInjection { index, value } if !is_bool(ty) => {
                let mut transfers = Vec::new();
                let value = Expr::compound_literal(
                    self.types.c_type(ty),
                    [
                        Initializer::designated(
                            "tag",
                            Expr::named_call("UINT32_C", [Expr::number(index.to_string())]),
                        ),
                        Initializer::designated_path(
                            ["payload".into(), format!("variant_{index}")],
                            self.materialize_atom(value, &mut transfers),
                        ),
                    ],
                );
                self.emit_simple_result_with_transfers(
                    output,
                    &binding.pattern,
                    ty,
                    value,
                    ResultOwnership::Owned,
                    &transfers,
                );
            }
            Operation::PrimitiveBinary {
                operator: BinaryPrimitive::Add,
                left,
                right,
            } if left.ty == Type::Symbol && self.ownership.can_transfer(left) => {
                let value = self.emit_symbol_concatenate(left, right, true);
                self.emit_simple_result_with_transfers(
                    output,
                    &binding.pattern,
                    ty,
                    value,
                    ResultOwnership::Owned,
                    &[left],
                );
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
