use crate::c_emit::syntax::{
    Block, Expr, FunctionDefinition, Initializer, Statement, SwitchCase, TranslationUnit, TypeName,
};
use crate::c_emit::types::is_bool;
use crate::check::ast::Type;
use crate::closure::ast::{self as closure, Atom, AtomKind, Reference};
use crate::control::ast::{StateId, Terminator};

use super::super::analysis::ControlCallMode;
use super::super::{
    BodyEmitter, ResultOwnership, function_name, has_direct_product_entry, pattern_type, value_name,
};
use super::ownership::{supports_local_control_type, zero_value};
use super::support::{local_slots, reachable_states, state_label, uint8, uint32};
use super::{frame_field_name, frame_name};

impl BodyEmitter<'_> {
    pub(in crate::c_emit::body) fn can_emit_local_control(
        &self,
        function: &closure::Function,
    ) -> bool {
        if self.common_control.contains(function.id) {
            return false;
        }
        if !supports_local_control_type(&function.parameter.ty)
            || !supports_local_control_type(&function.body.result.ty)
        {
            return false;
        }
        let Some(control_function) = self
            .control
            .functions
            .iter()
            .find(|candidate| candidate.id == function.id)
        else {
            return false;
        };
        let mut has_control = false;
        for site in reachable_states(&self.control, control_function.entry) {
            let state = &self.control.states[site.0];
            if state
                .input
                .as_ref()
                .is_some_and(|pattern| !supports_local_control_type(pattern_type(pattern)))
                || state
                    .bindings
                    .iter()
                    .any(|binding| !supports_local_control_type(pattern_type(&binding.pattern)))
            {
                return false;
            }
            if self.control_calls.mode(site) == Some(ControlCallMode::Dispatch) {
                has_control = true;
                if !matches!(
                    state.terminator,
                    Terminator::Call {
                        callee: Atom {
                            kind: AtomKind::Reference(Reference::SelfClosure(target)),
                            ..
                        },
                        ..
                    } if target == function.id
                ) {
                    return false;
                }
            }
            if self.control_calls.forwarded_self_argument(site).is_some() {
                has_control = true;
            }
        }
        has_control
    }

    pub(in crate::c_emit::body) fn emit_local_control_function(
        &mut self,
        function: &closure::Function,
    ) -> TranslationUnit {
        let control_function = self
            .control
            .functions
            .iter()
            .find(|candidate| candidate.id == function.id)
            .expect("control lowering preserves function identities")
            .clone();
        let sites = reachable_states(&self.control, control_function.entry);
        let local_slots = local_slots(&self.control, &sites, function.parameter.binding);
        let mut body = Block::default();
        self.emit_function_preamble(&mut body, function);
        if let Some(parameter) = function.parameter.binding
            && self.types.contains_managed(&function.parameter.ty)
        {
            let target = Expr::identifier(value_name(parameter));
            body.push(Statement::assignment(
                target.clone(),
                self.types.copy_value(&function.parameter.ty, target),
            ));
        }
        body.push(Statement::variable(
            "size_t",
            "mal_control_base_top",
            Some(Expr::identifier("mal_context").pointer_field("control_top")),
        ));
        body.push(Statement::variable(
            "size_t",
            "mal_control_base_frame",
            Some(Expr::identifier("mal_context").pointer_field("control_frame")),
        ));
        for (id, ty) in &local_slots {
            body.push(Statement::variable(
                self.types.c_type(ty),
                value_name(*id),
                Some(zero_value(self, ty)),
            ));
        }
        body.push(Statement::goto(state_label(control_function.entry)));
        for site in &sites {
            let state = self.control.states[site.0].clone();
            let mut state_body = Block::default();
            for binding in &state.bindings {
                self.emit_control_binding(&mut state_body, binding);
            }
            self.emit_control_terminator(
                &mut state_body,
                function,
                *site,
                &state.terminator,
                &local_slots,
            );
            body.push(Statement::label(state_label(*site), state_body));
        }
        let mut output = TranslationUnit::default();
        output.push(FunctionDefinition::from_signature(
            self.function_signature(function),
            body,
        ));
        output.blank_line();

        if has_direct_product_entry(&function.parameter.ty) {
            let mut next_parameter = 0;
            let parameter =
                self.direct_parameter_value(&function.parameter.ty, &mut next_parameter);
            let direct_body = Block::new([Statement::return_value(Expr::named_call(
                function_name(function.id),
                [
                    Expr::identifier("mal_context"),
                    Expr::identifier("mal_environment"),
                    parameter,
                ],
            ))]);
            let mut signature = self.direct_function_signature(function);
            if !self.control_calls.has_direct_target(function.id)
                || (self.closure_uses.has_direct_top_level_function(function.id)
                    && self.owned_calls.contains(function.id))
            {
                signature = signature.maybe_unused();
            }
            output.push(FunctionDefinition::from_signature(signature, direct_body));
            output.blank_line();
        }
        if self.owned_calls.contains(function.id) {
            output.push(self.emit_owned_control_wrapper(function));
            output.blank_line();
        }
        output
    }

    fn emit_control_terminator(
        &mut self,
        output: &mut Block,
        function: &closure::Function,
        site: StateId,
        terminator: &Terminator,
        local_slots: &[(crate::anf::ast::ValueId, Type)],
    ) {
        match terminator {
            Terminator::Return(value) => {
                let result = self.emit_atom(value);
                self.emit_local_control_return(
                    output,
                    function,
                    site,
                    result,
                    ResultOwnership::Borrowed,
                );
            }
            Terminator::Goto(target) => output.push(Statement::goto(state_label(*target))),
            Terminator::Jump { target, value } => {
                let input = self.control.states[target.0]
                    .input
                    .as_ref()
                    .expect("jump target accepts a value")
                    .clone();
                self.emit_control_borrowed_pattern_assignment(
                    output,
                    &input,
                    self.emit_atom(value),
                );
                output.push(Statement::goto(state_label(*target)));
            }
            Terminator::Call {
                callee,
                argument,
                resume,
            } => match self.control_calls.mode(site) {
                Some(ControlCallMode::Dispatch) => {
                    let previous = format!("mal_previous_frame_{}", site.0);
                    let frame_variable = format!("mal_frame_{}", site.0);
                    let next_parameter = format!("mal_next_parameter_{}", site.0);
                    output.push(Statement::variable(
                        self.types.c_type(&argument.ty),
                        &next_parameter,
                        Some(
                            self.types
                                .copy_value(&argument.ty, self.emit_atom(argument)),
                        ),
                    ));
                    output.push(Statement::variable(
                        "size_t",
                        &previous,
                        Some(Expr::identifier("mal_context").pointer_field("control_frame")),
                    ));
                    output.push(Statement::variable(
                        TypeName::named(frame_name(site)).pointer(),
                        &frame_variable,
                        Some(Expr::cast(
                            TypeName::named(frame_name(site)).pointer(),
                            Expr::named_call(
                                "mal_control_push",
                                [
                                    Expr::identifier("mal_context"),
                                    Expr::sizeof_type(frame_name(site)),
                                ],
                            ),
                        )),
                    ));
                    output.push(Statement::assignment(
                        Expr::identifier(&frame_variable)
                            .pointer_field("header")
                            .field("previous_frame"),
                        Expr::identifier(previous),
                    ));
                    output.push(Statement::assignment(
                        Expr::identifier(&frame_variable)
                            .pointer_field("header")
                            .field("resume"),
                        uint32(resume.0),
                    ));
                    self.emit_control_frame_field_moves(output, site, &frame_variable);
                    self.emit_control_activation_cleanup(output, function, local_slots);
                    if let Some(parameter) = function.parameter.binding {
                        output.push(Statement::assignment(
                            Expr::identifier(value_name(parameter)),
                            Expr::identifier(next_parameter),
                        ));
                    }
                    output.push(Statement::goto(state_label(
                        self.control_function(function.id).entry,
                    )));
                    debug_assert!(matches!(
                        callee.kind,
                        AtomKind::Reference(Reference::SelfClosure(target)) if target == function.id
                    ));
                }
                Some(ControlCallMode::Direct(_)) => {
                    let input = self.control.states[resume.0]
                        .input
                        .as_ref()
                        .expect("call resume accepts its result")
                        .clone();
                    self.emit_control_owned_pattern_assignment(
                        output,
                        &input,
                        self.emit_local_control_call(callee, argument),
                    );
                    output.push(Statement::goto(state_label(*resume)));
                }
                Some(ControlCallMode::DirectSelfTail) | None => {
                    unreachable!("a non-tail call has a complete call mode")
                }
            },
            Terminator::TailCall { callee, argument } => match self.control_calls.mode(site) {
                Some(ControlCallMode::DirectSelfTail) => {
                    let argument = self
                        .control_calls
                        .forwarded_self_argument(site)
                        .unwrap_or(argument);
                    let next = format!("mal_next_parameter_{}", site.0);
                    output.push(Statement::variable(
                        self.types.c_type(&argument.ty),
                        &next,
                        Some(
                            self.types
                                .copy_value(&argument.ty, self.emit_atom(argument)),
                        ),
                    ));
                    self.emit_control_activation_cleanup(output, function, local_slots);
                    if let Some(parameter) = function.parameter.binding {
                        output.push(Statement::assignment(
                            Expr::identifier(value_name(parameter)),
                            Expr::identifier(next),
                        ));
                    }
                    output.push(Statement::goto(state_label(
                        self.control_function(function.id).entry,
                    )));
                }
                Some(ControlCallMode::Direct(_)) => {
                    let result = self.emit_local_control_call(callee, argument);
                    self.emit_local_control_return(
                        output,
                        function,
                        site,
                        result,
                        ResultOwnership::Owned,
                    );
                }
                Some(ControlCallMode::Dispatch) | None => {
                    unreachable!("local control only dispatches non-tail self calls")
                }
            },
            Terminator::Case { scrutinee, arms } => {
                let value = self.emit_atom(scrutinee);
                let tag = if is_bool(&scrutinee.ty) {
                    value.clone()
                } else {
                    value.clone().field("tag")
                };
                let mut cases = Vec::new();
                for arm in arms {
                    let mut arm_body = Block::default();
                    let input = self.control.states[arm.target.0]
                        .input
                        .as_ref()
                        .expect("case target accepts its payload")
                        .clone();
                    let payload = if is_bool(&scrutinee.ty) {
                        Expr::compound_literal("MalType_Unit", [Initializer::positional(uint8(0))])
                    } else {
                        value
                            .clone()
                            .field("payload")
                            .field(format!("variant_{}", arm.index))
                    };
                    self.emit_control_borrowed_pattern_assignment(&mut arm_body, &input, payload);
                    arm_body.push(Statement::goto(state_label(arm.target)));
                    cases.push(SwitchCase::case(uint32(arm.index), arm_body));
                }
                cases.push(SwitchCase::default(Block::new([Statement::call(
                    "mal_trap",
                    [
                        Expr::identifier("mal_context"),
                        Expr::string("invalid sum tag"),
                    ],
                )])));
                output.push(Statement::switch(tag, cases));
            }
            Terminator::PrimitiveBranch {
                operator,
                left,
                right,
                otherwise,
                then,
            } => output.push(Statement::if_else(
                self.emit_primitive_condition(*operator, left, right),
                Block::new([Statement::goto(state_label(*then))]),
                Block::new([Statement::goto(state_label(*otherwise))]),
            )),
        }
    }

    fn emit_local_control_return(
        &mut self,
        output: &mut Block,
        function: &closure::Function,
        state: StateId,
        result: Expr,
        ownership: ResultOwnership,
    ) {
        let entry = self.control_function(function.id).entry;
        let sites = reachable_states(&self.control, entry);
        let local_slots = local_slots(&self.control, &sites, function.parameter.binding);
        let result_name = format!("mal_control_result_{}", state.0);
        let result = if ownership == ResultOwnership::Owned {
            result
        } else {
            self.types.copy_value(&function.body.result.ty, result)
        };
        output.push(Statement::variable(
            self.types.c_type(&function.body.result.ty),
            &result_name,
            Some(result),
        ));
        self.emit_control_activation_cleanup(output, function, &local_slots);
        let mut resume_cases = Vec::new();
        for site in &sites {
            let Some(frame) = self.control_frames.frame(*site).cloned() else {
                continue;
            };
            let frame_variable = format!("mal_resume_frame_{}_{}", state.0, site.0);
            let mut resume = Block::new([Statement::variable(
                TypeName::named(frame_name(*site)).pointer(),
                &frame_variable,
                Some(Expr::cast(
                    TypeName::named(frame_name(*site)).pointer(),
                    Expr::add(
                        Expr::identifier("mal_context").pointer_field("control_storage"),
                        Expr::identifier("mal_context").pointer_field("control_frame"),
                    ),
                )),
            )]);
            for (index, field) in frame.fields.iter().enumerate() {
                let frame_field =
                    Expr::identifier(&frame_variable).pointer_field(frame_field_name(index));
                resume.push(Statement::assignment(
                    Expr::identifier(value_name(field.value.id)),
                    frame_field.clone(),
                ));
                resume.push(Statement::assignment(
                    frame_field,
                    zero_value(self, &field.value.ty),
                ));
            }
            let input = self.control.states[frame.resume.0]
                .input
                .as_ref()
                .expect("resume accepts a call result")
                .clone();
            self.emit_control_owned_pattern_assignment(
                &mut resume,
                &input,
                Expr::identifier(&result_name),
            );
            resume.push(Statement::assignment(
                Expr::identifier("mal_context").pointer_field("control_top"),
                Expr::identifier("mal_context").pointer_field("control_frame"),
            ));
            resume.push(Statement::assignment(
                Expr::identifier("mal_context").pointer_field("control_frame"),
                Expr::identifier(&frame_variable)
                    .pointer_field("header")
                    .field("previous_frame"),
            ));
            resume.push(Statement::goto(state_label(frame.resume)));
            resume_cases.push(SwitchCase::case(uint32(frame.resume.0), resume));
        }
        resume_cases.push(SwitchCase::default(Block::new([Statement::call(
            "mal_trap",
            [
                Expr::identifier("mal_context"),
                Expr::string("invalid control resume state"),
            ],
        )])));
        let header = format!("mal_frame_header_{}", state.0);
        let unwind = Block::new([
            Statement::variable(
                TypeName::named("MalControlFrameHeader").pointer(),
                &header,
                Some(Expr::cast(
                    TypeName::named("MalControlFrameHeader").pointer(),
                    Expr::add(
                        Expr::identifier("mal_context").pointer_field("control_storage"),
                        Expr::identifier("mal_context").pointer_field("control_frame"),
                    ),
                )),
            ),
            Statement::switch(
                Expr::identifier(header).pointer_field("resume"),
                resume_cases,
            ),
        ]);
        output.push(Statement::if_else(
            Expr::equal(
                Expr::identifier("mal_context").pointer_field("control_top"),
                Expr::identifier("mal_control_base_top"),
            ),
            Block::new([
                Statement::assignment(
                    Expr::identifier("mal_context").pointer_field("control_frame"),
                    Expr::identifier("mal_control_base_frame"),
                ),
                Statement::return_value(Expr::identifier(result_name)),
            ]),
            unwind,
        ));
    }
}
