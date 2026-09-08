use crate::c_emit::syntax::{Block, Expr, Initializer, Statement, SwitchCase, TypeName};
use crate::c_emit::types::is_bool;
use crate::check::ast::Type;
use crate::closure::ast as closure;
use crate::control::ast::{StateId, Terminator};

use super::super::super::analysis::ControlCallMode;
use super::super::super::{BodyEmitter, ResultOwnership, value_name};
use super::super::support::{state_label, uint8, uint32};
use super::super::{frame_field_name, frame_name};
use super::{CONTROL_DESTROY_ENVIRONMENT, CONTROL_ENVIRONMENT};

impl BodyEmitter<'_> {
    pub(super) fn emit_common_control_terminator(
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
                self.emit_common_control_return(
                    output,
                    function,
                    site,
                    result,
                    ResultOwnership::Borrowed,
                    local_slots,
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
                Some(ControlCallMode::Dispatch) => self.emit_common_dispatch(
                    output,
                    function,
                    site,
                    callee,
                    argument,
                    Some(*resume),
                    local_slots,
                ),
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
                    self.emit_common_control_return(
                        output,
                        function,
                        site,
                        result,
                        ResultOwnership::Owned,
                        local_slots,
                    );
                }
                Some(ControlCallMode::Dispatch) => self.emit_common_dispatch(
                    output,
                    function,
                    site,
                    callee,
                    argument,
                    None,
                    local_slots,
                ),
                None => unreachable!("a tail call has a complete call mode"),
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

    #[allow(clippy::too_many_arguments)]
    fn emit_common_dispatch(
        &mut self,
        output: &mut Block,
        function: &closure::Function,
        site: StateId,
        callee: &closure::Atom,
        argument: &closure::Atom,
        resume: Option<StateId>,
        local_slots: &[(crate::anf::ast::ValueId, Type)],
    ) {
        let next_callee = format!("mal_next_callee_{}", site.0);
        let next_argument = format!("mal_next_argument_{}", site.0);
        output.push(Statement::variable(
            self.types.c_type(&callee.ty),
            &next_callee,
            Some(self.types.copy_value(&callee.ty, self.emit_atom(callee))),
        ));
        output.push(Statement::variable(
            self.types.c_type(&argument.ty),
            &next_argument,
            Some(
                self.types
                    .copy_value(&argument.ty, self.emit_atom(argument)),
            ),
        ));

        for target in self
            .control_calls
            .recursive_dispatch_targets(site)
            .unwrap_or_default()
            .to_vec()
        {
            debug_assert!(self.common_control.contains(target));
            let target_function = self.function(target).clone();
            let mut branch = Block::default();
            if let Some(resume) = resume {
                self.emit_common_frame_push(&mut branch, function, site, resume, local_slots);
            } else {
                self.emit_control_activation_cleanup(&mut branch, function, local_slots);
                self.emit_release_control_environment(&mut branch);
            }
            branch.push(Statement::assignment(
                Expr::identifier(CONTROL_ENVIRONMENT),
                Expr::identifier(&next_callee).field("environment"),
            ));
            branch.push(Statement::assignment(
                Expr::identifier(CONTROL_DESTROY_ENVIRONMENT),
                Expr::identifier(&next_callee).field("destroy_environment"),
            ));
            if let Some(parameter) = target_function.parameter.binding {
                branch.push(Statement::assignment(
                    Expr::identifier(value_name(parameter)),
                    Expr::identifier(&next_argument),
                ));
            } else {
                self.types.destroy_value(
                    &mut branch,
                    &target_function.parameter.ty,
                    Expr::identifier(&next_argument),
                );
            }
            branch.push(Statement::goto(state_label(
                self.control_function(target).entry,
            )));
            output.push(Statement::if_then(
                Expr::equal(
                    Expr::identifier(&next_callee).field("call"),
                    Expr::identifier(super::super::super::function_name(target)),
                ),
                branch,
            ));
        }

        let fallback_result = format!("mal_fallback_result_{}", site.0);
        output.push(Statement::variable(
            self.types.c_type(&self.function_result_type(callee)),
            &fallback_result,
            Some(Expr::call(
                Expr::identifier(&next_callee).field("call"),
                [
                    Expr::identifier("mal_context"),
                    Expr::identifier(&next_callee).field("environment"),
                    Expr::identifier(&next_argument),
                ],
            )),
        ));
        self.types
            .destroy_value(output, &argument.ty, Expr::identifier(&next_argument));
        self.types
            .destroy_value(output, &callee.ty, Expr::identifier(&next_callee));
        if let Some(resume) = resume {
            let input = self.control.states[resume.0]
                .input
                .as_ref()
                .expect("call resume accepts its result")
                .clone();
            self.emit_control_owned_pattern_assignment(
                output,
                &input,
                Expr::identifier(fallback_result),
            );
            output.push(Statement::goto(state_label(resume)));
        } else {
            self.emit_common_control_return(
                output,
                function,
                site,
                Expr::identifier(fallback_result),
                ResultOwnership::Owned,
                local_slots,
            );
        }
    }

    fn emit_common_frame_push(
        &self,
        output: &mut Block,
        function: &closure::Function,
        site: StateId,
        resume: StateId,
        local_slots: &[(crate::anf::ast::ValueId, Type)],
    ) {
        let frame = self
            .control_frames
            .frame(site)
            .expect("dispatching non-tail calls have frames");
        let previous = format!("mal_previous_frame_{}", site.0);
        let frame_variable = format!("mal_frame_{}", site.0);
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
        for (index, field) in frame.fields.iter().enumerate() {
            output.push(Statement::assignment(
                Expr::identifier(&frame_variable).pointer_field(frame_field_name(index)),
                self.types.copy_value(
                    &field.value.ty,
                    Expr::identifier(value_name(field.value.id)),
                ),
            ));
        }
        if frame.needs_environment {
            output.push(Statement::assignment(
                Expr::identifier(&frame_variable).pointer_field("environment"),
                Expr::identifier(CONTROL_ENVIRONMENT),
            ));
            output.push(Statement::assignment(
                Expr::identifier(&frame_variable).pointer_field("destroy_environment"),
                Expr::identifier(CONTROL_DESTROY_ENVIRONMENT),
            ));
            output.push(Statement::assignment(
                Expr::identifier(CONTROL_ENVIRONMENT),
                Expr::identifier("NULL"),
            ));
            output.push(Statement::assignment(
                Expr::identifier(CONTROL_DESTROY_ENVIRONMENT),
                Expr::identifier("NULL"),
            ));
        }
        self.emit_control_activation_cleanup(output, function, local_slots);
        if !frame.needs_environment {
            self.emit_release_control_environment(output);
        }
    }

    fn function_result_type(&self, callee: &closure::Atom) -> Type {
        let Type::Function { result, .. } = &callee.ty else {
            unreachable!("dispatch callees have function type")
        };
        (**result).clone()
    }
}
mod returning;
