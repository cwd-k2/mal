mod terminator;

use std::collections::HashSet;

use crate::c_emit::syntax::{
    Block, Expr, FunctionDefinition, FunctionSignature, Parameter, Statement, SwitchCase,
    TranslationUnit, TypeName, VariableDeclaration,
};
use crate::closure::ast as closure;

use super::super::{
    BodyEmitter, environment_name, function_name, has_direct_product_entry, value_name,
};
use super::ownership::zero_value;
use super::support::{local_slots, reachable_states, uint32};

const COMMON_CONTROL_NAME: &str = "mal_run_control";
const CONTROL_ENTRY: &str = "mal_control_entry";
const CONTROL_ENVIRONMENT: &str = "mal_environment";
const CONTROL_DESTROY_ENVIRONMENT: &str = "mal_control_destroy_environment";
const CONTROL_ARGUMENT: &str = "mal_control_argument";
const CONTROL_RESULT: &str = "mal_control_result";

impl BodyEmitter<'_> {
    pub(in crate::c_emit::body) fn emit_common_control_machine(&mut self) -> TranslationUnit {
        if self.common_control.is_empty() {
            return TranslationUnit::default();
        }
        let functions = self
            .program
            .functions
            .iter()
            .filter(|function| self.common_control.contains(function.id))
            .cloned()
            .collect::<Vec<_>>();
        let mut body = Block::default();
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
            let sites = reachable_states(&self.control, self.control_function(function.id).entry);
            for (id, ty) in local_slots(&self.control, &sites, function.parameter.binding) {
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
            let sites = reachable_states(&self.control, control_function.entry);
            let slots = local_slots(&self.control, &sites, function.parameter.binding);
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
        body.push(Statement::label("mal_control_done", Block::default()));

        let definition = FunctionDefinition::from_signature(
            FunctionSignature::static_function(
                "void",
                COMMON_CONTROL_NAME,
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
        let mut body = Block::new([
            Statement::variable(self.types.c_type(&function.body.result.ty), result, None),
            Statement::call(
                COMMON_CONTROL_NAME,
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
