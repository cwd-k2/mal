use crate::c_emit::syntax::{Block, Expr, Statement, SwitchCase};
use crate::c_emit::types::is_bool;
use crate::check::ast::Type;
use crate::closure::ast::FunctionId;
use crate::closure::ast::{self as closure, Atom, Block as ClosureBlock, Pattern};

use super::super::ResultOwnership;
use super::super::{BodyEmitter, pattern_type};

impl BodyEmitter<'_> {
    pub(in crate::c_emit::body) fn emit_tail_block(
        &mut self,
        output: &mut Block,
        block: &ClosureBlock,
        function: FunctionId,
        parameter_name: &str,
        parameter_type: &Type,
    ) {
        self.emit_tail_block_with_cleanup(
            output,
            block,
            function,
            parameter_name,
            parameter_type,
            &[],
        );
    }

    fn emit_tail_block_with_cleanup<'a>(
        &mut self,
        output: &mut Block,
        block: &'a ClosureBlock,
        function: FunctionId,
        parameter_name: &str,
        parameter_type: &Type,
        outer_cleanup: &[TailCleanup<'a>],
    ) {
        let tail = block.bindings.last().filter(|binding| {
            matches!(
                (&block.result.kind, &binding.pattern),
                (
                    closure::AtomKind::Reference(closure::Reference::Binding(result)),
                    Pattern::Binding { id, .. }
                ) if result == id
            )
        });
        let ordinary_count = block.bindings.len() - usize::from(tail.is_some());
        self.emit_bindings(output, &block.bindings[..ordinary_count]);
        let mut cleanup = outer_cleanup.to_vec();
        cleanup.push(TailCleanup::Bindings(&block.bindings[..ordinary_count]));

        match tail.map(|binding| &binding.operation) {
            Some(closure::Operation::Call { callee, argument })
                if matches!(
                        callee.kind,
                        closure::AtomKind::Reference(closure::Reference::SelfClosure(id)) if id == function
                ) =>
            {
                let mut transfers = Vec::new();
                let next_parameter = self.materialize_atom(argument, &mut transfers);
                output.push(Statement::variable(
                    self.types.c_type(parameter_type),
                    "mal_tail_next_parameter",
                    Some(next_parameter),
                ));
                self.clear_transferred_atoms(output, &transfers);
                self.emit_tail_cleanup(output, &cleanup);
                self.types
                    .destroy_value(output, parameter_type, Expr::identifier(parameter_name));
                output.push(Statement::assignment(
                    Expr::identifier(parameter_name),
                    Expr::identifier("mal_tail_next_parameter"),
                ));
                output.push(Statement::goto("mal_tail_entry"));
            }
            Some(closure::Operation::Case { scrutinee, arms }) => self.emit_tail_case(
                output,
                scrutinee,
                arms,
                function,
                parameter_name,
                parameter_type,
                &cleanup,
            ),
            Some(closure::Operation::PrimitiveBranch {
                operator,
                left,
                right,
                otherwise,
                then,
            }) => self.emit_tail_primitive_branch(
                output,
                *operator,
                left,
                right,
                otherwise,
                then,
                function,
                parameter_name,
                parameter_type,
                &cleanup,
            ),
            Some(_) => {
                self.emit_binding(output, block.bindings.last().expect("tail binding"));
                cleanup.push(TailCleanup::Pattern(
                    &block.bindings.last().expect("tail binding").pattern,
                ));
                self.emit_tail_return(
                    output,
                    &block.result,
                    parameter_name,
                    parameter_type,
                    &cleanup,
                );
            }
            None => self.emit_tail_return(
                output,
                &block.result,
                parameter_name,
                parameter_type,
                &cleanup,
            ),
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn emit_tail_case<'a>(
        &mut self,
        output: &mut Block,
        scrutinee: &Atom,
        arms: &'a [closure::CaseArm],
        function: FunctionId,
        parameter_name: &str,
        parameter_type: &Type,
        outer_cleanup: &[TailCleanup<'a>],
    ) {
        let scrutinee_atom = scrutinee;
        let transfer_scrutinee = self.can_transfer(scrutinee);
        let bool_scrutinee = is_bool(&scrutinee.ty);
        let scrutinee = self.emit_atom(scrutinee);
        let tag = if bool_scrutinee {
            scrutinee.clone()
        } else {
            scrutinee.clone().field("tag")
        };
        let mut cases = Vec::new();
        for arm in arms {
            let payload = case_payload(&scrutinee, arm.index, bool_scrutinee);
            let mut body = Block::default();
            if transfer_scrutinee {
                self.emit_simple_result_with_transfers(
                    &mut body,
                    &arm.pattern,
                    pattern_type(&arm.pattern),
                    payload,
                    ResultOwnership::Owned,
                    &[scrutinee_atom],
                );
            } else {
                self.emit_simple_result(
                    &mut body,
                    &arm.pattern,
                    pattern_type(&arm.pattern),
                    payload,
                    ResultOwnership::Borrowed,
                );
            }
            let mut cleanup = outer_cleanup.to_vec();
            cleanup.push(TailCleanup::Pattern(&arm.pattern));
            self.emit_tail_block_with_cleanup(
                &mut body,
                &arm.value,
                function,
                parameter_name,
                parameter_type,
                &cleanup,
            );
            cases.push(SwitchCase::case(uint32(arm.index), body));
        }
        cases.push(invalid_sum_default());
        output.push(Statement::switch(tag, cases));
    }

    #[allow(clippy::too_many_arguments)]
    fn emit_tail_primitive_branch<'a>(
        &mut self,
        output: &mut Block,
        operator: crate::core::ast::BinaryPrimitive,
        left: &Atom,
        right: &Atom,
        otherwise: &'a ClosureBlock,
        then: &'a ClosureBlock,
        function: FunctionId,
        parameter_name: &str,
        parameter_type: &Type,
        cleanup: &[TailCleanup<'a>],
    ) {
        let condition = self.emit_primitive_condition(operator, left, right);
        let mut then_body = Block::default();
        self.emit_tail_block_with_cleanup(
            &mut then_body,
            then,
            function,
            parameter_name,
            parameter_type,
            cleanup,
        );
        let mut otherwise_body = Block::default();
        self.emit_tail_block_with_cleanup(
            &mut otherwise_body,
            otherwise,
            function,
            parameter_name,
            parameter_type,
            cleanup,
        );
        output.push(Statement::if_else(condition, then_body, otherwise_body));
    }

    fn emit_tail_return(
        &self,
        output: &mut Block,
        result: &Atom,
        parameter_name: &str,
        parameter_type: &Type,
        cleanup: &[TailCleanup<'_>],
    ) {
        let mut transfers = Vec::new();
        let result_value = self.materialize_atom(result, &mut transfers);
        output.push(Statement::variable(
            self.types.c_type(&result.ty),
            "mal_tail_result",
            Some(result_value),
        ));
        self.clear_transferred_atoms(output, &transfers);
        self.emit_tail_cleanup(output, cleanup);
        self.types
            .destroy_value(output, parameter_type, Expr::identifier(parameter_name));
        output.push(Statement::return_value(Expr::identifier("mal_tail_result")));
    }

    fn emit_tail_cleanup(&self, output: &mut Block, cleanup: &[TailCleanup<'_>]) {
        for cleanup in cleanup.iter().rev() {
            match cleanup {
                TailCleanup::Bindings(bindings) => {
                    for binding in bindings.iter().rev() {
                        self.destroy_pattern_bindings(output, &binding.pattern);
                    }
                }
                TailCleanup::Pattern(pattern) => {
                    self.destroy_pattern_bindings(output, pattern);
                }
            }
        }
    }

    pub(super) fn emit_case(
        &mut self,
        output: &mut Block,
        pattern: &Pattern,
        ty: &Type,
        scrutinee: &Atom,
        arms: &[closure::CaseArm],
    ) {
        let target = self.result_target(pattern);
        output.push(Statement::variable(
            self.types.c_type(ty),
            target.clone(),
            None,
        ));
        let scrutinee_atom = scrutinee;
        let transfer_scrutinee = self.can_transfer(scrutinee);
        let bool_scrutinee = is_bool(&scrutinee.ty);
        let scrutinee = self.emit_atom(scrutinee);
        let tag = if bool_scrutinee {
            scrutinee.clone()
        } else {
            scrutinee.clone().field("tag")
        };
        let mut cases = Vec::new();
        for arm in arms {
            let payload = case_payload(&scrutinee, arm.index, bool_scrutinee);
            let mut body = Block::default();
            if transfer_scrutinee {
                self.emit_simple_result_with_transfers(
                    &mut body,
                    &arm.pattern,
                    pattern_type(&arm.pattern),
                    payload,
                    ResultOwnership::Owned,
                    &[scrutinee_atom],
                );
            } else {
                self.emit_simple_result(
                    &mut body,
                    &arm.pattern,
                    pattern_type(&arm.pattern),
                    payload,
                    ResultOwnership::Borrowed,
                );
            }
            self.emit_block_bindings(&mut body, &arm.value);
            let mut transfers = Vec::new();
            let result = self.materialize_atom(&arm.value.result, &mut transfers);
            body.push(Statement::assignment(
                Expr::identifier(target.clone()),
                result,
            ));
            self.clear_transferred_atoms(&mut body, &transfers);
            self.emit_block_cleanup(&mut body, &arm.value);
            self.destroy_pattern_bindings(&mut body, &arm.pattern);
            body.push(Statement::Break);
            cases.push(SwitchCase::case(uint32(arm.index), body));
        }
        cases.push(invalid_sum_default());
        output.push(Statement::switch(tag, cases));
        output.push(discard(Expr::identifier(target.clone())));
        match pattern {
            Pattern::Binding { .. } => {}
            Pattern::Wildcard { .. } => {
                self.types
                    .destroy_value(output, ty, Expr::identifier(target));
            }
            Pattern::Product { .. } => {
                self.emit_owned_pattern_bindings(output, pattern, Expr::identifier(target));
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub(super) fn emit_primitive_branch(
        &mut self,
        output: &mut Block,
        pattern: &Pattern,
        ty: &Type,
        operator: crate::core::ast::BinaryPrimitive,
        left: &Atom,
        right: &Atom,
        otherwise: &ClosureBlock,
        then: &ClosureBlock,
    ) {
        let target = self.result_target(pattern);
        output.push(Statement::variable(
            self.types.c_type(ty),
            target.clone(),
            None,
        ));
        let condition = self.emit_primitive_condition(operator, left, right);
        let mut then_body = Block::default();
        self.emit_block_bindings(&mut then_body, then);
        let mut then_transfers = Vec::new();
        let then_result = self.materialize_atom(&then.result, &mut then_transfers);
        then_body.push(Statement::assignment(
            Expr::identifier(target.clone()),
            then_result,
        ));
        self.clear_transferred_atoms(&mut then_body, &then_transfers);
        self.emit_block_cleanup(&mut then_body, then);
        let mut otherwise_body = Block::default();
        self.emit_block_bindings(&mut otherwise_body, otherwise);
        let mut otherwise_transfers = Vec::new();
        let otherwise_result = self.materialize_atom(&otherwise.result, &mut otherwise_transfers);
        otherwise_body.push(Statement::assignment(
            Expr::identifier(target.clone()),
            otherwise_result,
        ));
        self.clear_transferred_atoms(&mut otherwise_body, &otherwise_transfers);
        self.emit_block_cleanup(&mut otherwise_body, otherwise);
        output.push(Statement::if_else(condition, then_body, otherwise_body));
        output.push(discard(Expr::identifier(target.clone())));
        match pattern {
            Pattern::Binding { .. } => {}
            Pattern::Wildcard { .. } => {
                self.types
                    .destroy_value(output, ty, Expr::identifier(target));
            }
            Pattern::Product { .. } => {
                self.emit_owned_pattern_bindings(output, pattern, Expr::identifier(target));
            }
        }
    }
}

#[derive(Clone, Copy)]
enum TailCleanup<'a> {
    Bindings(&'a [closure::Binding]),
    Pattern(&'a Pattern),
}

fn case_payload(scrutinee: &Expr, index: usize, bool_scrutinee: bool) -> Expr {
    if bool_scrutinee {
        Expr::compound_literal(
            "MalType_Unit",
            [crate::c_emit::syntax::Initializer::positional(
                Expr::named_call("UINT8_C", [Expr::number("0")]),
            )],
        )
    } else {
        scrutinee
            .clone()
            .field("payload")
            .field(format!("variant_{index}"))
    }
}

fn uint32(value: usize) -> Expr {
    Expr::named_call("UINT32_C", [Expr::number(value.to_string())])
}

fn discard(value: Expr) -> Statement {
    Statement::expression(Expr::cast("void", value))
}

fn invalid_sum_default() -> SwitchCase {
    SwitchCase::default(Block::new([Statement::call(
        "mal_trap",
        [
            Expr::identifier("mal_context"),
            Expr::string("invalid sum tag"),
        ],
    )]))
}
