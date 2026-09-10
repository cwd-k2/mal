mod terminator;

use std::collections::HashSet;

use crate::c_emit::syntax::{
    Block, Expr, FunctionDefinition, FunctionSignature, Parameter, Statement, SwitchCase,
    TranslationUnit, TypeName, VariableDeclaration,
};
use crate::closure::ast as closure;

use super::super::analysis::ControlRegionId;
use super::super::{
    BodyEmitter, environment_name, function_name, has_direct_product_entry, value_name,
};
use super::ownership::zero_value;
use super::support::{
    emit_control_stack_cache, emit_control_stack_preamble, local_slots, reachable_states, uint32,
};

const CONTROL_ENTRY: &str = "mal_control_entry";
const CONTROL_ENVIRONMENT: &str = "mal_environment";
const CONTROL_DESTROY_ENVIRONMENT: &str = "mal_control_destroy_environment";
const CONTROL_ARGUMENT: &str = "mal_control_argument";
const CONTROL_RESULT: &str = "mal_control_result";

impl BodyEmitter<'_> {
    pub(in crate::c_emit::body) fn emit_common_control_machines(&mut self) -> TranslationUnit {
        let regions = self
            .control_regions
            .ids()
            .filter(|region| {
                self.control_regions
                    .functions(*region)
                    .iter()
                    .any(|function| self.uses_common_control(*function))
            })
            .collect::<Vec<_>>();
        let mut output = TranslationUnit::default();
        for region in regions {
            output.extend(self.emit_common_control_machine(region));
        }
        output
    }

    fn emit_common_control_machine(&mut self, region: ControlRegionId) -> TranslationUnit {
        let region_functions = self.control_regions.functions(region).to_vec();
        let functions = self
            .program
            .functions
            .iter()
            .filter(|function| region_functions.contains(&function.id))
            .cloned()
            .collect::<Vec<_>>();
        let mut body = Block::default();
        let arena = self.control_frames.arena(region);
        if let Some(arena) = arena {
            emit_control_stack_preamble(
                &mut body,
                arena,
                self.control_frames.homogeneous_frame(region).is_some(),
            );
        }
        body.push(Statement::variable(
            TypeName::const_named("void").pointer(),
            CONTROL_ENVIRONMENT,
            Some(Expr::identifier("NULL")),
        ));
        body.push(Statement::variable_declaration(
            VariableDeclaration::function_pointer(
                "void",
                CONTROL_DESTROY_ENVIRONMENT,
                [
                    Parameter::unnamed(TypeName::named("MalContext").pointer()),
                    Parameter::unnamed(TypeName::const_named("void").pointer()),
                ],
            ),
            Some(Expr::identifier("NULL")),
        ));
        body.push(Statement::expression(Expr::cast(
            "void",
            Expr::identifier(CONTROL_ARGUMENT),
        )));

        let mut declared = HashSet::new();
        for function in &functions {
            if let Some(parameter) = function.parameter.binding
                && declared.insert(parameter)
            {
                body.push(Statement::variable(
                    self.types.c_type(&function.parameter.ty),
                    value_name(parameter),
                    Some(zero_value(self, &function.parameter.ty)),
                ));
            }
            let sites = reachable_states(self.control, self.control_function(function.id).entry);
            for (id, ty) in local_slots(self.control, &sites, function.parameter.binding) {
                if declared.insert(id) {
                    body.push(Statement::variable(
                        self.types.c_type(&ty),
                        value_name(id),
                        Some(zero_value(self, &ty)),
                    ));
                }
            }
        }

        let entries = functions
            .iter()
            .map(|function| {
                let mut entry = Block::new([Statement::assignment(
                    Expr::identifier(CONTROL_ENVIRONMENT),
                    Expr::identifier("mal_initial_environment"),
                )]);
                if let Some(parameter) = function.parameter.binding {
                    let argument = Expr::dereference(Expr::cast(
                        self.types.c_type(&function.parameter.ty).pointer(),
                        Expr::identifier(CONTROL_ARGUMENT),
                    ));
                    entry.push(Statement::assignment(
                        Expr::identifier(value_name(parameter)),
                        self.types.copy_value(&function.parameter.ty, argument),
                    ));
                }
                entry.push(Statement::goto(super::support::state_label(
                    self.control_function(function.id).entry,
                )));
                SwitchCase::case(uint32(self.control_function(function.id).entry.0), entry)
            })
            .chain([SwitchCase::default(Block::new([Statement::call(
                "mal_trap",
                [
                    Expr::identifier("mal_context"),
                    Expr::string("invalid control entry state"),
                ],
            )]))])
            .collect();
        body.push(Statement::switch(Expr::identifier(CONTROL_ENTRY), entries));

        for function in &functions {
            let control_function = self.control_function(function.id).clone();
            let sites = reachable_states(self.control, control_function.entry);
            let slots = local_slots(self.control, &sites, function.parameter.binding);
            for site in sites {
                let state = self.control.states[site.0].clone();
                let mut state_body = Block::default();
                if state.needs_environment && !function.environment.is_empty() {
                    state_body.push(Statement::variable(
                        TypeName::const_named(environment_name(function.id)).pointer(),
                        "mal_environment_fields",
                        Some(Expr::cast(
                            TypeName::const_named(environment_name(function.id)).pointer(),
                            Expr::identifier(CONTROL_ENVIRONMENT),
                        )),
                    ));
                    state_body.push(Statement::expression(Expr::cast(
                        "void",
                        Expr::identifier("mal_environment_fields"),
                    )));
                }
                for binding in &state.bindings {
                    self.emit_control_binding(&mut state_body, binding);
                }
                self.emit_common_control_terminator(
                    &mut state_body,
                    function,
                    site,
                    &state.terminator,
                    &slots,
                );
                body.push(Statement::label(
                    super::support::state_label(site),
                    state_body,
                ));
            }
        }
        let done = format!("mal_control_done_{}", region.0);
        let mut done_body = Block::default();
        if let Some(arena) = arena {
            emit_control_stack_cache(&mut done_body, arena);
        }
        body.push(Statement::label(&done, done_body));

        let definition = FunctionDefinition::from_signature(
            FunctionSignature::static_function(
                "void",
                common_control_name(region),
                [
                    Parameter::named(TypeName::named("MalContext").pointer(), "mal_context"),
                    Parameter::named("uint32_t", CONTROL_ENTRY),
                    Parameter::named(
                        TypeName::const_named("void").pointer(),
                        "mal_initial_environment",
                    ),
                    Parameter::named(TypeName::const_named("void").pointer(), CONTROL_ARGUMENT),
                    Parameter::named(TypeName::named("void").pointer(), CONTROL_RESULT),
                ],
            ),
            body,
        );
        let mut output = TranslationUnit::new([definition.into()]);
        output.blank_line();
        output
    }

    pub(in crate::c_emit::body) fn emit_common_control_wrappers(
        &self,
        function: &closure::Function,
    ) -> TranslationUnit {
        let mut output = TranslationUnit::default();
        let parameter_name = function
            .parameter
            .binding
            .map_or_else(|| "mal_parameter".into(), value_name);
        let result = "mal_common_control_result";
        let region = self
            .control_regions
            .function_region(function.id)
            .expect("common control function belongs to a region");
        let mut body = Block::new([
            Statement::variable(self.types.c_type(&function.body.result.ty), result, None),
            Statement::call(
                common_control_name(region),
                [
                    Expr::identifier("mal_context"),
                    uint32(self.control_function(function.id).entry.0),
                    Expr::identifier("mal_environment"),
                    Expr::address_of(Expr::identifier(&parameter_name)),
                    Expr::address_of(Expr::identifier(result)),
                ],
            ),
            Statement::return_value(Expr::identifier(result)),
        ]);
        output.push(FunctionDefinition::from_signature(
            self.function_signature(function),
            body.clone(),
        ));
        output.blank_line();

        if has_direct_product_entry(&function.parameter.ty) {
            let mut next_parameter = 0;
            let parameter =
                self.direct_parameter_value(&function.parameter.ty, &mut next_parameter);
            body = Block::new([Statement::return_value(Expr::named_call(
                function_name(function.id),
                [
                    Expr::identifier("mal_context"),
                    Expr::identifier("mal_environment"),
                    parameter,
                ],
            ))]);
            output.push(FunctionDefinition::from_signature(
                self.direct_function_signature(function).maybe_unused(),
                body,
            ));
            output.blank_line();
        }
        if self.owned_calls.contains(function.id) {
            output.push(self.emit_owned_control_wrapper(function));
            output.blank_line();
        }
        output
    }
}

pub(super) fn common_control_done(region: ControlRegionId) -> String {
    format!("mal_control_done_{}", region.0)
}

fn common_control_name(region: ControlRegionId) -> String {
    format!("mal_run_control_{}", region.0)
}
