use std::collections::HashMap;

use crate::anf::ast::ValueId;
use crate::closure::ast::{Atom, FunctionId};
use crate::control::ast::{Operation, Program, StateId, Terminator};
use crate::execution::{ControlCallMode, ControlRegionId, ParameterDestination};
use mal_frontend::check::ast::Type;

mod admission;
mod aggregate;
mod bridge;
mod call_emission;
mod control_storage;
mod control_top;
mod frame;
mod memory;
mod operation;
mod plan;
mod scalar;
mod setup;
mod symbol;
mod terminator;
pub(super) mod types;
mod value;

use memory::ManagedBufferElements;
use plan::{TopLevelConstants, collect_pattern_slot, insert_slot, main_function};
use scalar::{comparison_predicate, scalar_type};
use types::{Types, is_bool};

pub(super) fn admit_target(
    execution: &crate::execution::Program,
    target: super::TargetLayout,
) -> Result<(), mal_syntax::diagnostic::Diagnostic> {
    admission::admit(execution, target)
}

pub(super) struct Output {
    pub(super) globals: String,
    pub(super) definitions: String,
    pub(super) main: FunctionId,
    pub(super) main_parameter: Type,
    pub(super) uses_control: bool,
    pub(super) uses_byte_runtime: bool,
}

#[cfg(test)]
pub(super) fn supports(execution: &crate::execution::Program) -> bool {
    generate(
        execution,
        super::TargetLayout::natural(8, 8).expect("test target layout"),
        super::optimization::OptimizationSet::production(),
    )
    .is_ok()
}

pub(super) fn generate(
    execution: &crate::execution::Program,
    target: super::TargetLayout,
    enabled: super::optimization::OptimizationSet,
) -> Result<Output, String> {
    let (main, main_parameter) = main_function(execution).ok_or("entry function selection")?;
    let types = Types::for_program(target, &execution.lowered.functions)
        .ok_or("value type construction")?;
    let top_levels = TopLevelConstants::new(execution, types.clone())
        .ok_or("top-level constant construction")?;
    let index = ProgramIndex::new(execution).ok_or("program index construction")?;
    let optimizations = super::optimization::OptimizationPlan::new(execution, enabled);
    debug_assert!(optimizations.is_valid(execution, enabled));
    let mut globals = top_levels.globals().to_string();
    let mut definitions = FunctionEmitter::new(
        execution,
        &index,
        main,
        target,
        &top_levels,
        &execution.ownership,
        &optimizations,
    )
    .and_then(|mut emitter| emitter.emit_managed_buffer_element_callbacks())
    .ok_or("Buffer element callback emission")?;
    let mut uses_control = false;
    for function in &execution.control.functions {
        let failed =
            |version: &str| format!("emission of the {version} of {}", describe(function.id));
        let emitter = FunctionEmitter::new(
            execution,
            &index,
            function.id,
            target,
            &top_levels,
            &execution.ownership,
            &optimizations,
        )
        .ok_or_else(|| failed("setup"))?;
        uses_control |= !emitter.frame_sites.is_empty();
        if emitter.has_native_version() {
            let native = FunctionEmitter::new(
                execution,
                &index,
                function.id,
                target,
                &top_levels,
                &execution.ownership,
                &optimizations,
            )
            .ok_or_else(|| failed("setup"))?
            .into_native_version();
            // Both versions declare the same globals and environment destructor; the native version owns them.
            let emitted = native.emit().ok_or_else(|| failed("native version"))?;
            globals.push_str(&emitted.globals);
            definitions.push_str(&emitted.definition);
            definitions.push('\n');
            let frames = emitter
                .into_frames_version()
                .emit()
                .ok_or_else(|| failed("frames version"))?;
            definitions.push_str(&frames.definition);
            definitions.push('\n');
            continue;
        }
        let emitted = emitter.emit().ok_or_else(|| failed("body"))?;
        globals.push_str(&emitted.globals);
        definitions.push_str(&emitted.definition);
        definitions.push('\n');
    }
    Ok(Output {
        globals,
        definitions,
        main,
        main_parameter,
        uses_control,
        uses_byte_runtime: symbol::program_uses_byte_runtime(execution),
    })
}

/// Which of a function's emitted bodies an emitter produces.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum EmissionMode {
    Standard,
    /// The version that keeps suspended callers in the control arena; emitted as `<name>_frames`.
    Frames,
    /// The version that nests native calls while the native stack has room.
    Native,
}

struct FunctionEmitter<'a> {
    mode: EmissionMode,
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
    frame_sites: Vec<StateId>,
    frame_tags: HashMap<StateId, u32>,
    local_control_storage: bool,
    local_control_top: bool,
    external_storage: Option<(usize, usize)>,
    buffer_value_storage: Option<(usize, usize)>,
    needs_symbol_result_slot: bool,
    types: Types,
    source_layouts: crate::backend::source_layout::SourceLayouts,
    top_levels: &'a TopLevelConstants,
    ownership: &'a crate::execution::OwnershipPlan,
    optimizations: &'a super::optimization::OptimizationPlan,
    next_register: usize,
    next_entry_alloca: usize,
    entry_allocas: String,
    globals: String,
    output: String,
}

struct ProgramIndex<'a> {
    control_functions: HashMap<FunctionId, &'a crate::control::ast::Function>,
    lowered_functions: HashMap<FunctionId, &'a crate::closure::ast::Function>,
    externals: HashMap<
        mal_frontend::resolve::ast::ExternalOperationId,
        &'a crate::core::ast::ExternalOperation,
    >,
    managed_buffer_elements: ManagedBufferElements,
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
            managed_buffer_elements: ManagedBufferElements::collect(execution),
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

struct PreparedValue {
    value: EmittedValue,
    consumed_slots: Vec<Slot>,
}

struct EmittedFunction {
    globals: String,
    definition: String,
}

/// Names a function in an internal error by its LLVM symbol.
fn describe(id: FunctionId) -> String {
    function_name(id).unwrap_or_else(|| format!("{id:?}"))
}

fn function_name(id: FunctionId) -> Option<String> {
    let FunctionId::Lambda(id) = id;
    Some(format!("mal_function_{}", id.0))
}

fn function_number(id: FunctionId) -> Option<u32> {
    let FunctionId::Lambda(id) = id;
    Some(id.0)
}

fn self_tail_entry_label(id: FunctionId) -> Option<String> {
    Some(format!("mal_self_tail_entry_{}", function_number(id)?))
}
