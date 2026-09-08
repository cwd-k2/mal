use crate::c_emit::syntax::{Block, Expr, Statement, SwitchCase, TypeName};
use crate::check::ast::Type;
use crate::closure::ast as closure;
use crate::control::ast::StateId;

use super::super::super::super::{BodyEmitter, ResultOwnership, pattern_type, value_name};
use super::super::super::ownership::zero_value;
use super::super::super::support::control_stack_field;
use super::super::super::support::{state_label, uint8, uint32};
use super::super::super::{frame_field_name, frame_name};
use super::super::{
    CONTROL_DESTROY_ENVIRONMENT, CONTROL_ENVIRONMENT, CONTROL_RESULT, common_control_done,
};

impl BodyEmitter<'_> {
    pub(super) fn emit_common_control_return(
        &mut self,
        output: &mut Block,
        function: &closure::Function,
        state: StateId,
        result: Expr,
        ownership: ResultOwnership,
        local_slots: &[(crate::anf::ast::ValueId, Type)],
    ) {
        let result_type = &function.body.result.ty;
        let result_name = format!("mal_control_result_{}", state.0);
        let result = if ownership == ResultOwnership::Owned {
            result
        } else {
            self.types.copy_value(result_type, result)
        };
        output.push(Statement::variable(
            self.types.c_type(result_type),
            &result_name,
            Some(result),
        ));
        self.emit_control_activation_cleanup(output, function, local_slots);
        self.emit_release_control_environment(output);

        let region = self
            .control_regions
            .function_region(function.id)
            .expect("common control return belongs to a region");

        let root = Block::new([
            Statement::assignment(
                Expr::dereference(Expr::cast(
                    self.types.c_type(result_type).pointer(),
                    Expr::identifier(CONTROL_RESULT),
                )),
                Expr::identifier(&result_name),
            ),
            Statement::goto(common_control_done(region)),
        ]);
        if self.control_regions.arena(region).is_none() {
            output.append(root);
            return;
        }
        let mut resume_cases = Vec::new();
        for index in 0..self.control.states.len() {
            let site = StateId(index);
            let Some(frame) = self.control_frames.frame(site).cloned() else {
                continue;
            };
            if self.control_regions.site_region(site) != Some(region) {
                continue;
            }
            let resume_type = self.control.states[frame.resume.0]
                .input
                .as_ref()
                .map(pattern_type);
            if resume_type != Some(result_type) {
                continue;
            }
            let frame_variable = format!("mal_resume_frame_{}_{}", state.0, site.0);
            let mut resume = Block::new([Statement::variable(
                TypeName::named(frame_name(site)).pointer(),
                &frame_variable,
                Some(Expr::cast(
                    TypeName::named(frame_name(site)).pointer(),
                    Expr::add(control_stack_field("storage"), control_stack_field("frame")),
                )),
            )]);
            for (field_index, field) in frame.fields.iter().enumerate() {
                let source =
                    Expr::identifier(&frame_variable).pointer_field(frame_field_name(field_index));
                resume.push(Statement::assignment(
                    Expr::identifier(value_name(field.value.id)),
                    source.clone(),
                ));
                resume.push(Statement::assignment(
                    source,
                    zero_value(self, &field.value.ty),
                ));
            }
            if frame.needs_environment {
                resume.push(Statement::assignment(
                    Expr::identifier(CONTROL_ENVIRONMENT),
                    Expr::identifier(&frame_variable).pointer_field("environment"),
                ));
                resume.push(Statement::assignment(
                    Expr::identifier(CONTROL_DESTROY_ENVIRONMENT),
                    Expr::identifier(&frame_variable).pointer_field("destroy_environment"),
                ));
                resume.push(Statement::assignment(
                    Expr::identifier(&frame_variable).pointer_field("environment"),
                    Expr::identifier("NULL"),
                ));
                resume.push(Statement::assignment(
                    Expr::identifier(&frame_variable).pointer_field("destroy_environment"),
                    Expr::identifier("NULL"),
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
                control_stack_field("top"),
                control_stack_field("frame"),
            ));
            resume.push(Statement::assignment(
                control_stack_field("frame"),
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
                    Expr::add(control_stack_field("storage"), control_stack_field("frame")),
                )),
            ),
            Statement::switch(
                Expr::identifier(header).pointer_field("resume"),
                resume_cases,
            ),
        ]);
        output.push(Statement::if_else(
            Expr::equal(control_stack_field("top"), Expr::number("0")),
            root,
            unwind,
        ));
    }

    pub(super) fn emit_release_control_environment(&self, output: &mut Block) {
        let environment = Expr::identifier(CONTROL_ENVIRONMENT);
        let destroy = Expr::identifier(CONTROL_DESTROY_ENVIRONMENT);
        output.push(Statement::if_then(
            Expr::not_equal(destroy.clone(), Expr::identifier("NULL")),
            Block::new([Statement::if_then(
                Expr::not_equal(
                    Expr::named_call("mal_release", [environment.clone()]),
                    uint8(0),
                ),
                Block::new([Statement::expression(Expr::call(
                    destroy,
                    [Expr::identifier("mal_context"), environment],
                ))]),
            )]),
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
}
