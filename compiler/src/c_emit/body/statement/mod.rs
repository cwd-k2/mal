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
        self.emit_bindings(output, &block.bindings);
    }

    pub(in crate::c_emit::body) fn emit_bindings(
        &mut self,
        output: &mut Block,
        bindings: &[Binding],
    ) {
        let mut index = 0;
        while let Some(binding) = bindings.get(index) {
            if let Some(consumer) = bindings.get(index + 1)
                && self.emit_ephemeral_product_consumer(output, binding, consumer)
            {
                index += 2;
                continue;
            }
            self.emit_binding(output, binding);
            index += 1;
        }
    }

    fn emit_ephemeral_product_consumer(
        &mut self,
        output: &mut Block,
        product: &Binding,
        consumer: &Binding,
    ) -> bool {
        let (Pattern::Binding { id, ty }, Operation::Product(elements)) =
            (&product.pattern, &product.operation)
        else {
            return false;
        };
        let argument = match &consumer.operation {
            Operation::SymbolAt { argument }
            | Operation::Call { argument, .. }
            | Operation::Memory { argument, .. }
            | Operation::Atom(argument) => argument,
            _ => return false,
        };
        let crate::closure::ast::AtomKind::Reference(crate::closure::ast::Reference::Binding(
            argument_id,
        )) = argument.kind
        else {
            return false;
        };
        if argument_id != *id || !self.can_transfer(argument) {
            return false;
        }

        if let (
            Operation::Atom(_),
            Pattern::Product {
                elements: patterns, ..
            },
        ) = (&consumer.operation, &consumer.pattern)
        {
            if patterns.len() != elements.len() {
                return false;
            }
            self.ephemeral_bindings.insert(*id);
            for (pattern, atom) in patterns.iter().zip(elements) {
                let mut transfers = Vec::new();
                let value = self.materialize_atom(atom, &mut transfers);
                self.emit_simple_result_with_transfers(
                    output,
                    pattern,
                    pattern_type(pattern),
                    value,
                    ResultOwnership::Owned,
                    &transfers,
                );
            }
            return true;
        }

        let (value, ownership) = match &consumer.operation {
            Operation::SymbolAt { .. } if elements.len() == 2 => {
                self.needs.symbol_at = true;
                (
                    Expr::named_call(
                        "mal_symbol_at",
                        [
                            Expr::identifier("mal_context"),
                            self.emit_atom(&elements[0]),
                            self.emit_atom(&elements[1]),
                        ],
                    ),
                    ResultOwnership::Borrowed,
                )
            }
            Operation::Call { callee, .. }
                if self.direct_function(callee).is_some_and(|(function, _)| {
                    !self.types.contains_managed(ty) || !self.owned_calls.contains(function)
                }) && super::has_direct_product_entry(ty) =>
            {
                (
                    self.emit_direct_call_with_product_elements(callee, ty, elements),
                    ResultOwnership::Owned,
                )
            }
            Operation::Memory { primitive, .. } if elements.len() == 2 => (
                self.emit_memory_with_product_elements(*primitive, elements),
                if matches!(primitive, crate::check::ast::MemoryPrimitive::LoadSymbol) {
                    ResultOwnership::Owned
                } else {
                    ResultOwnership::Borrowed
                },
            ),
            _ => return false,
        };
        self.ephemeral_bindings.insert(*id);
        self.emit_simple_result(
            output,
            &consumer.pattern,
            pattern_type(&consumer.pattern),
            value,
            ownership,
        );
        true
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
                    && !self.can_transfer(atom) =>
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
            Operation::Call { callee, argument }
                if self.types.contains_managed(&argument.ty)
                    && self.can_transfer(argument)
                    && self
                        .direct_function(callee)
                        .is_some_and(|(function, _)| self.owned_calls.contains(function)) =>
            {
                let value = self.emit_call(callee, argument, true);
                self.emit_simple_result_with_transfers(
                    output,
                    &binding.pattern,
                    ty,
                    value,
                    ResultOwnership::Owned,
                    &[argument],
                );
            }
            Operation::PrimitiveBinary {
                operator: BinaryPrimitive::Add,
                left,
                right,
            } if left.ty == Type::Symbol && self.can_transfer(left) => {
                let value = self.emit_symbol_concatenate(left, right, true, false);
                self.emit_simple_result_with_transfers(
                    output,
                    &binding.pattern,
                    ty,
                    value,
                    ResultOwnership::Owned,
                    &[left],
                );
            }
            Operation::PrimitiveBinary {
                operator: BinaryPrimitive::Add,
                left,
                right,
            } if left.ty == Type::Symbol && self.can_transfer(right) => {
                let value = self.emit_symbol_concatenate(left, right, false, true);
                self.emit_simple_result_with_transfers(
                    output,
                    &binding.pattern,
                    ty,
                    value,
                    ResultOwnership::Owned,
                    &[right],
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
