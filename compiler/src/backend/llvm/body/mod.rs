use std::collections::HashMap;

use crate::anf::ast::ValueId;
use crate::check::ast::Type;
use crate::closure::ast::{Atom, FunctionId};
use crate::control::ast::{Operation, Program, StateId, Terminator};
use crate::execution::{ControlCallMode, ControlRegionId, ParameterDestination};

mod admission;
mod aggregate;
mod bridge;
mod call_emission;
mod frame;
mod memory;
mod operation;
pub(in crate::backend::llvm) mod ownership;
mod plan;
mod scalar;
mod setup;
mod source_layout;
mod symbol;
mod terminator;
pub(super) mod types;
mod value;

use plan::{
    TopLevelConstants, collect_pattern_ids, collect_pattern_slot, insert_slot, main_function,
    pattern_value_type, reachable_states,
};
use scalar::{comparison_predicate, scalar_type};
use types::{Types, is_bool};

pub(super) fn admit_target(
    execution: &crate::execution::Program,
    target: super::TargetLayout,
) -> Result<(), crate::diagnostic::Diagnostic> {
    admission::admit(execution, target)
}

pub(super) struct Output {
    pub(super) globals: String,
    pub(super) definitions: String,
    pub(super) main: FunctionId,
    pub(super) main_parameter: Type,
    pub(super) uses_control: bool,
    pub(super) uses_symbol_runtime: bool,
}

#[cfg(test)]
pub(super) fn supports(execution: &crate::execution::Program) -> bool {
    generate(
        execution,
        super::TargetLayout::natural(8, 8).expect("test target layout"),
        super::optimization::OptimizationSet::production(),
    )
    .is_some()
}

pub(super) fn generate(
    execution: &crate::execution::Program,
    target: super::TargetLayout,
    enabled: super::optimization::OptimizationSet,
) -> Option<Output> {
    let (main, main_parameter) = main_function(execution)?;
    let types = Types::for_target(target)?;
    let source_layouts = source_layout::SourceLayouts::new(target);
    let top_levels = TopLevelConstants::new(execution, types, source_layouts)?;
    let ownership = ownership::Plan::new(&execution.control);
    let index = ProgramIndex::new(execution)?;
    let optimizations =
        super::optimization::OptimizationPlan::new(&execution.control, &ownership, enabled);
    debug_assert!(optimizations.is_valid(&execution.control, &ownership, enabled));
    let mut globals = top_levels.globals().to_string();
    let mut definitions = String::new();
    let mut uses_control = false;
    for function in &execution.control.functions {
        let emitter = FunctionEmitter::new(
            execution,
            &index,
            function.id,
            target,
            &top_levels,
            &ownership,
            &optimizations,
        )?;
        uses_control |= !emitter.frame_sites.is_empty();
        let emitted = emitter.emit()?;
        globals.push_str(&emitted.globals);
        definitions.push_str(&emitted.definition);
        definitions.push('\n');
    }
    Some(Output {
        globals,
        definitions,
        main,
        main_parameter,
        uses_control,
        uses_symbol_runtime: symbol::program_uses_runtime(execution),
    })
}

struct FunctionEmitter<'a> {
    execution: &'a crate::execution::Program,
    index: &'a ProgramIndex<'a>,
    control: &'a Program,
    function: &'a crate::control::ast::Function,
    current_function: FunctionId,
    common_region: Option<ControlRegionId>,
    states: Vec<StateId>,
    state_functions: HashMap<StateId, FunctionId>,
    result_type: Type,
    slots: HashMap<ValueId, Slot>,
    function_slots: HashMap<FunctionId, Vec<ValueId>>,
    frame_sites: Vec<StateId>,
    frame_tags: HashMap<StateId, u32>,
    external_storage: Option<(usize, usize)>,
    types: Types,
    source_layouts: source_layout::SourceLayouts,
    top_levels: &'a TopLevelConstants,
    ownership: &'a ownership::Plan,
    optimizations: &'a super::optimization::OptimizationPlan,
    next_register: usize,
    globals: String,
    output: String,
}

struct ProgramIndex<'a> {
    control_functions: HashMap<FunctionId, &'a crate::control::ast::Function>,
    lowered_functions: HashMap<FunctionId, &'a crate::closure::ast::Function>,
    externals:
        HashMap<crate::resolve::ast::ExternalOperationId, &'a crate::core::ast::ExternalOperation>,
}

impl<'a> ProgramIndex<'a> {
    fn new(execution: &'a crate::execution::Program) -> Option<Self> {
        let control_functions = execution
            .control
            .functions
            .iter()
            .map(|function| (function.id, function))
            .collect::<HashMap<_, _>>();
        let lowered_functions = execution
            .lowered
            .functions
            .iter()
            .map(|function| (function.id, function))
            .collect::<HashMap<_, _>>();
        let externals = execution
            .lowered
            .interface
            .externals
            .iter()
            .map(|external| (external.id, external))
            .collect::<HashMap<_, _>>();
        (control_functions.len() == execution.control.functions.len()
            && lowered_functions.len() == execution.lowered.functions.len()
            && externals.len() == execution.lowered.interface.externals.len())
        .then_some(Self {
            control_functions,
            lowered_functions,
            externals,
        })
    }
}

#[derive(Clone)]
pub(super) struct Slot {
    index: usize,
    ty: Type,
}

#[derive(Clone)]
struct EmittedValue {
    ty: Type,
    representation: String,
    owned: bool,
}

struct EmittedFunction {
    globals: String,
    definition: String,
}

fn function_name(id: FunctionId) -> Option<String> {
    let FunctionId::Lambda(id) = id;
    Some(format!("mal_function_{}", id.0))
}

fn function_number(id: FunctionId) -> Option<u32> {
    let FunctionId::Lambda(id) = id;
    Some(id.0)
}
