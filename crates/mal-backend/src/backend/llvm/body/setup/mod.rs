//! One function emitter: the states, slots, frames, and scratch storage it owns, then its prologue, states, and output.

use super::*;

mod frames;
mod prologue;
mod region;
mod scratch;

impl<'a> FunctionEmitter<'a> {
    pub(in crate::backend::llvm::body) fn new(
        execution: &'a crate::execution::Program,
        index: &'a ProgramIndex<'a>,
        id: FunctionId,
        target: super::super::TargetLayout,
        top_levels: &'a TopLevelConstants,
        ownership: &'a crate::execution::OwnershipPlan,
        optimizations: &'a super::super::optimization::OptimizationPlan,
    ) -> Option<Self> {
        let types = Types::for_program(target, &execution.lowered.functions);
        let source_layouts = crate::backend::source_layout::SourceLayouts::new(target);
        let function = *index.control_functions.get(&id)?;
        let lowered = *index.lowered_functions.get(&id)?;
        if types.value(&function.parameter.ty).is_none()
            || types.value(&lowered.body.result.ty).is_none()
        {
            return None;
        }
        let common_region = execution
            .control_regions
            .function_region(id)
            .filter(|region| execution.control_calls.requires_common_control(*region));
        let functions = common_region.map_or_else(
            || vec![id],
            |region| execution.control_regions.functions(region).to_vec(),
        );
        let region = region::RegionStates::collect(index, &functions)?;
        let slots = region.slots(execution, index, &types)?;
        let local_control_top = optimizations.localizes_control_top(id);
        let local_control_storage = optimizations.localizes_control_storage(id);
        let (frame_sites, frame_tags) = frames::frame_sites(
            execution,
            &region.states,
            id,
            common_region,
            &types,
            local_control_top || local_control_storage,
        )?;
        let scratch =
            scratch::Scratch::measure(execution, index, &region.states, &types, &source_layouts)?;
        Some(Self {
            mode: EmissionMode::Standard,
            execution,
            index,
            control: &execution.control,
            function,
            current_function: id,
            common_region,
            result_type: lowered.body.result.ty.clone(),
            states: region.states,
            state_functions: region.state_functions,
            slots,
            frame_sites,
            frame_tags,
            local_control_storage,
            local_control_top,
            external_storage: scratch.external,
            buffer_value_storage: scratch.buffer_value,
            needs_symbol_result_slot: scratch.symbol_result,
            types,
            source_layouts,
            top_levels,
            ownership,
            optimizations,
            next_register: 0,
            next_entry_alloca: 0,
            current_definition: None,
            definitions: Vec::new(),
            emission_failed: false,
            globals: Vec::new(),
        })
    }

    pub(in crate::backend::llvm::body) fn emit(mut self) -> Option<EmittedFunction> {
        if self.mode != EmissionMode::Frames {
            self.emit_environment_destructor()?;
        }
        self.begin_signature()?;
        self.emit_control_prologue();
        self.emit_slot_allocas()?;
        self.emit_active_environment();
        self.emit_scratch_allocas()?;
        if self.mode == EmissionMode::Native && self.native_worker_parameters().is_some() {
            self.emit_native_context_entry()?;
        }
        if self.mode == EmissionMode::Native {
            self.emit_native_entry_guard()?;
        }
        self.emit_entry_parameter()?;
        emit_terminator! {
            self;
            branch {
                target: #{ format!("mal_state_{}", self.function.entry.0) },
            };
        };

        for site in self.states.clone() {
            self.emit_state(site)?;
        }
        self.finish_function()?;
        if self.emission_failed {
            return None;
        }
        Some(EmittedFunction {
            globals: self.globals,
            definitions: self.definitions,
        })
    }
}
