use crate::c_emit::syntax::{Block, Expr, Statement, SwitchCase};
use crate::c_emit::types::is_bool;
use crate::check::ast::Type;
use crate::closure::ast::FunctionId;
use crate::closure::ast::{self as closure, Atom, Block as ClosureBlock, Pattern};

use super::super::{BodyEmitter, pattern_type, value_name};

impl BodyEmitter<'_> {
    pub(in crate::c_emit::body) fn emit_tail_block(
        &mut self,
        output: &mut Block,
        block: &ClosureBlock,
        function: FunctionId,
        parameter_name: &str,
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
        for binding in &block.bindings[..ordinary_count] {
            self.emit_binding(output, binding);
        }

        match tail.map(|binding| &binding.operation) {
            Some(closure::Operation::Call { callee, argument })
                if matches!(
                    callee.kind,
                    closure::AtomKind::Reference(closure::Reference::SelfClosure(id)) if id == function
                ) =>
            {
                output.push(Statement::assignment(
                    Expr::identifier(parameter_name),
                    self.emit_atom(argument),
                ));
                output.push(Statement::goto("mal_tail_entry"));
            }
            Some(closure::Operation::Case { scrutinee, arms }) => {
                self.emit_tail_case(output, scrutinee, arms, function, parameter_name)
            }
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
            ),
            Some(_) => {
                self.emit_binding(output, block.bindings.last().expect("tail binding"));
                output.push(Statement::return_value(self.emit_atom(&block.result)));
            }
            None => output.push(Statement::return_value(self.emit_atom(&block.result))),
        }
    }

    fn emit_tail_case(
        &mut self,
        output: &mut Block,
        scrutinee: &Atom,
        arms: &[closure::CaseArm],
        function: FunctionId,
        parameter_name: &str,
    ) {
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
            self.emit_simple_result(&mut body, &arm.pattern, pattern_type(&arm.pattern), payload);
            self.emit_tail_block(&mut body, &arm.value, function, parameter_name);
            cases.push(SwitchCase::case(uint32(arm.index), body));
        }
        cases.push(invalid_sum_default());
        output.push(Statement::switch(tag, cases));
    }

    #[allow(clippy::too_many_arguments)]
    fn emit_tail_primitive_branch(
        &mut self,
        output: &mut Block,
        operator: crate::core::ast::BinaryPrimitive,
        left: &Atom,
        right: &Atom,
        otherwise: &ClosureBlock,
        then: &ClosureBlock,
        function: FunctionId,
        parameter_name: &str,
    ) {
        let condition = self.emit_primitive_condition(operator, left, right);
        let mut then_body = Block::default();
        self.emit_tail_block(&mut then_body, then, function, parameter_name);
        let mut otherwise_body = Block::default();
        self.emit_tail_block(&mut otherwise_body, otherwise, function, parameter_name);
        output.push(Statement::if_else(condition, then_body, otherwise_body));
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
            if let Pattern::Binding { id, ty } = &arm.pattern {
                let name = value_name(*id);
                body.push(Statement::variable(
                    self.types.c_type(ty),
                    &name,
                    Some(payload),
                ));
                body.push(discard(Expr::identifier(name)));
            }
            self.emit_block_bindings(&mut body, &arm.value);
            body.push(Statement::assignment(
                Expr::identifier(target.clone()),
                self.emit_atom(&arm.value.result),
            ));
            body.push(Statement::Break);
            cases.push(SwitchCase::case(uint32(arm.index), body));
        }
        cases.push(invalid_sum_default());
        output.push(Statement::switch(tag, cases));
        output.push(discard(Expr::identifier(target.clone())));
        if matches!(pattern, Pattern::Product { .. }) {
            self.emit_pattern_bindings(output, pattern, Expr::identifier(target));
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
        then_body.push(Statement::assignment(
            Expr::identifier(target.clone()),
            self.emit_atom(&then.result),
        ));
        let mut otherwise_body = Block::default();
        self.emit_block_bindings(&mut otherwise_body, otherwise);
        otherwise_body.push(Statement::assignment(
            Expr::identifier(target.clone()),
            self.emit_atom(&otherwise.result),
        ));
        output.push(Statement::if_else(condition, then_body, otherwise_body));
        output.push(discard(Expr::identifier(target.clone())));
        if matches!(pattern, Pattern::Product { .. }) {
            self.emit_pattern_bindings(output, pattern, Expr::identifier(target));
        }
    }
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
    SwitchCase::default(Block::new([Statement::expression(Expr::named_call(
        "mal_trap",
        [
            Expr::identifier("mal_context"),
            Expr::string("invalid sum tag"),
        ],
    ))]))
}
