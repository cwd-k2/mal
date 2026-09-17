use std::collections::HashMap;

use crate::anf::ast::ValueId;
use crate::check::ast::Type;
use crate::closure::ast::{Atom, FunctionId};
use crate::control::ast::{Operation, Program, StateId, Terminator};
use crate::execution::{ControlCallMode, ControlRegionId, ParameterDestination};

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
        8,
        super::optimization::OptimizationSet::production(),
    )
    .is_some()
}

pub(super) fn generate(
    execution: &crate::execution::Program,
    pointer_size: usize,
    enabled: super::optimization::OptimizationSet,
) -> Option<Output> {
    let (main, main_parameter) = main_function(execution)?;
    let types = Types::new(pointer_size)?;
    let top_levels = TopLevelConstants::new(execution, types)?;
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
            types,
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
    let name = match id {
        FunctionId::Lambda(id) => format!("mal_function_{}", id.0),
        FunctionId::Memory(primitive) => {
            format!("mal_memory_function_{}", memory_primitive_name(primitive)?)
        }
    };
    Some(name)
}

fn function_number(id: FunctionId) -> Option<u32> {
    match id {
        FunctionId::Lambda(id) => Some(id.0),
        FunctionId::Memory(_) => None,
    }
}

fn memory_primitive_name(primitive: crate::check::ast::MemoryPrimitive) -> Option<&'static str> {
    use crate::check::ast::{MemoryPrimitive, MemoryScalar};

    Some(match primitive {
        MemoryPrimitive::Load(MemoryScalar::Int8) => "load_int8",
        MemoryPrimitive::Load(MemoryScalar::Int16) => "load_int16",
        MemoryPrimitive::Load(MemoryScalar::Int32) => "load_int32",
        MemoryPrimitive::Load(MemoryScalar::Int64) => "load_int64",
        MemoryPrimitive::Load(MemoryScalar::UInt8) => "load_uint8",
        MemoryPrimitive::Load(MemoryScalar::UInt16) => "load_uint16",
        MemoryPrimitive::Load(MemoryScalar::UInt32) => "load_uint32",
        MemoryPrimitive::Load(MemoryScalar::UInt64) => "load_uint64",
        MemoryPrimitive::Load(MemoryScalar::Float32) => "load_float32",
        MemoryPrimitive::Load(MemoryScalar::Float64) => "load_float64",
        MemoryPrimitive::Store(MemoryScalar::Int8) => "store_int8",
        MemoryPrimitive::Store(MemoryScalar::Int16) => "store_int16",
        MemoryPrimitive::Store(MemoryScalar::Int32) => "store_int32",
        MemoryPrimitive::Store(MemoryScalar::Int64) => "store_int64",
        MemoryPrimitive::Store(MemoryScalar::UInt8) => "store_uint8",
        MemoryPrimitive::Store(MemoryScalar::UInt16) => "store_uint16",
        MemoryPrimitive::Store(MemoryScalar::UInt32) => "store_uint32",
        MemoryPrimitive::Store(MemoryScalar::UInt64) => "store_uint64",
        MemoryPrimitive::Store(MemoryScalar::Float32) => "store_float32",
        MemoryPrimitive::Store(MemoryScalar::Float64) => "store_float64",
        MemoryPrimitive::LoadPtr => "load_ptr",
        MemoryPrimitive::StorePtr => "store_ptr",
        MemoryPrimitive::LoadSymbol => "load_symbol",
        MemoryPrimitive::StoreSymbol => "store_symbol",
        MemoryPrimitive::OffsetForward
        | MemoryPrimitive::OffsetBackward
        | MemoryPrimitive::Place
        | MemoryPrimitive::Region
        | MemoryPrimitive::ProjectAddress
        | MemoryPrimitive::Align
        | MemoryPrimitive::LoadValue
        | MemoryPrimitive::StoreValue => return None,
    })
}
