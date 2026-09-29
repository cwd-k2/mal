use super::*;
use crate::backend::llvm::syntax::{
    llvm_function_attributes, llvm_parameters, llvm_signature, llvm_type,
};
impl<'a> FunctionEmitter<'a> {
    pub(super) fn new(
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
        let function_states = functions
            .iter()
            .map(|function| {
                let states = &index
                    .control_functions
                    .get(function)
                    .expect("control region function has control states")
                    .states;
                (*function, states.clone())
            })
            .collect::<Vec<_>>();
        let states = function_states
            .iter()
            .flat_map(|(_, states)| states.iter().copied())
            .collect::<Vec<_>>();
        let state_functions = function_states
            .iter()
            .flat_map(|(function, states)| states.iter().map(|state| (*state, *function)))
            .collect::<HashMap<_, _>>();
        if state_functions.len() != states.len() {
            return None;
        }
        let mut slots = HashMap::new();
        for (function_id, function_states) in &function_states {
            let region_function = *index.control_functions.get(function_id)?;
            if let ParameterDestination::Bind(id) =
                execution.parameters.destination(*function_id)?
            {
                insert_slot(&mut slots, id, region_function.parameter.ty.clone());
            }
            for state in function_states {
                let state = &execution.control.states[state.0];
                if let Some(pattern) = &state.input {
                    collect_pattern_slot(pattern, &mut slots, types.clone())?;
                }
                for binding in &state.bindings {
                    collect_pattern_slot(&binding.pattern, &mut slots, types.clone())?;
                }
            }
        }
        let frame_sites = states
            .iter()
            .filter(|site| execution.control_frames.frame(**site).is_some())
            .copied()
            .collect::<Vec<_>>();
        let frame_tags = frame_sites
            .iter()
            .enumerate()
            .map(|(tag, site)| Some((*site, u32::try_from(tag).ok()?)))
            .collect::<Option<HashMap<_, _>>>()?;
        let local_control_top = optimizations.localizes_control_top(id);
        let local_control_storage = optimizations.localizes_control_storage(id);
        if (local_control_top || local_control_storage) && frame_sites.is_empty() {
            return None;
        }
        if frame_sites.iter().any(|site| {
            let frame = execution
                .control_frames
                .frame(*site)
                .expect("collected frame site");
            (common_region.is_none()
                && (execution.control_calls.mode(*site) != Some(ControlCallMode::DirectRegion(id))
                    || frame.carries_environment))
                || common_region.is_some_and(|region| {
                    execution.control_regions.site_region(*site) != Some(region)
                })
                || frame
                    .fields
                    .iter()
                    .any(|field| types.value(&field.ty).is_none())
        }) {
            return None;
        }
        let external_ids = states.iter().flat_map(|site| {
            execution.control.states[site.0]
                .bindings
                .iter()
                .filter_map(|binding| match binding.operation {
                    Operation::ExternalCall { id, .. } => Some(id),
                    _ => None,
                })
        });
        let needs_symbol_result_slot = states.iter().any(|state| {
            execution.control.states[state.0]
                .bindings
                .iter()
                .any(|binding| {
                    matches!(
                        &binding.operation,
                        Operation::Symbol {
                            primitive: mal_frontend::check::ast::SymbolPrimitive::Concatenate
                                | mal_frontend::check::ast::SymbolPrimitive::Prefix
                                | mal_frontend::check::ast::SymbolPrimitive::Suffix,
                            ..
                        }
                    )
                })
        });
        let buffer_value_storage =
            states
                .iter()
                .flat_map(|state| &execution.control.states[state.0].bindings)
                .filter_map(|binding| match &binding.operation {
                    Operation::Buffer {
                        operation:
                            crate::core::ast::BufferOperation::New
                            | crate::core::ast::BufferOperation::Fill,
                        element,
                        ..
                    } => source_layouts
                        .layout(element)
                        .map(|layout| (layout.stride, layout.alignment))
                        .or_else(|| {
                            let value = types.value(element)?;
                            Some((value.size, value.alignment))
                        }),
                    _ => None,
                })
                .filter(|(size, _)| *size != 0)
                .fold(None, |storage, (element_size, element_alignment)| {
                    let (size, alignment) = storage.unwrap_or((0usize, 1usize));
                    Some((size.max(element_size), alignment.max(element_alignment)))
                });
        let mut external_storage = None;
        for id in external_ids {
            let external = *index.externals.get(&id)?;
            for ty in [&external.parameter, &external.result] {
                let value = types.value(ty)?;
                let (size, alignment) = external_storage.unwrap_or((0usize, 1usize));
                external_storage = Some((size.max(value.size), alignment.max(value.alignment)));
            }
        }
        Some(Self {
            mode: EmissionMode::Standard,
            execution,
            index,
            control: &execution.control,
            function,
            current_function: id,
            common_region,
            result_type: lowered.body.result.ty.clone(),
            states,
            state_functions,
            slots,
            frame_sites,
            frame_tags,
            local_control_storage,
            local_control_top,
            external_storage,
            buffer_value_storage,
            needs_symbol_result_slot,
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

    pub(super) fn emit(mut self) -> Option<EmittedFunction> {
        if self.mode != EmissionMode::Frames {
            self.emit_environment_destructor()?;
        }
        let parameters =
            if self.mode == EmissionMode::Native && self.native_worker_parameters().is_some() {
                self.native_worker_parameters()?
            } else if self.function.parameter.ty == Type::Unit {
                llvm_parameters! {
                    "%mal_context" : ptr,
                    "%mal_control_top" : ptr,
                    "%mal_environment" : ptr,
                }
            } else {
                let parameter = self.types.value(&self.function.parameter.ty)?;
                llvm_parameters! {
                    "%mal_context" : ptr,
                    "%mal_control_top" : ptr,
                    "%mal_environment" : ptr,
                    "%mal_parameter" : #{ parameter.llvm },
                }
            };
        let result = self.types.value(&self.result_type)?;
        let suffix = match self.mode {
            EmissionMode::Frames => "_frames",
            EmissionMode::Native if self.native_worker_parameters().is_some() => "_native",
            EmissionMode::Standard | EmissionMode::Native => "",
        };
        // The native version must stay small on its hot path, so the frames version is never inlined into it.
        let attributes = if self.mode == EmissionMode::Frames {
            llvm_function_attributes!(noinline)
        } else {
            Vec::new()
        };
        self.begin_function(llvm_signature! {
            #[linkage(internal)]
            #[attributes(...#{ attributes })]
            fn #{ format!("{}{suffix}", function_name(self.function.id)) }(
                ...#{ parameters },
            ) -> #{ result.llvm }
        });
        self.block("entry");
        if !self.frame_sites.is_empty() {
            emit_instruction! {
                self;
                let "%mal_control_base" = load {
                    ty: #{ self.types.index_llvm_type() },
                    pointer: "%mal_control_top",
                    alignment: #{ self.types.index_alignment() },
                    metadata: [],
                };
            };
            if self.local_control_top {
                emit_instruction! {
                    self;
                    let "%mal_local_control_top" = alloca {
                        ty: #{ self.types.index_llvm_type() },
                        alignment: #{ self.types.index_alignment() },
                    };
                };
                emit_instruction! {
                    self;
                    store {
                        value: typed(#{ self.types.index_llvm_type() }, "%mal_control_base"),
                        pointer: "%mal_local_control_top",
                        alignment: #{ self.types.index_alignment() },
                        metadata: [],
                    };
                };
            }
            if self.local_control_storage {
                emit_instruction! {
                    self;
                    let "%mal_local_control_storage" = alloca {
                        ty: (ptr),
                        alignment: #{ self.types.pointer_alignment() },
                    };
                };
                emit_instruction! {
                    self;
                    let "%mal_local_control_capacity" = alloca {
                        ty: #{ self.types.index_llvm_type() },
                        alignment: #{ self.types.index_alignment() },
                    };
                };
                self.refresh_control_storage();
            }
        }
        let mut slots = self.slots.values().cloned().collect::<Vec<_>>();
        slots.sort_by_key(|slot| slot.index);
        for slot in slots {
            let value_type = self.types.value(&slot.ty)?;
            emit_instruction! {
                self;
                let #{ format!("%mal_slot_{}", slot.index) } = alloca {
                    ty: #{ value_type.llvm.clone() },
                    alignment: #{ value_type.alignment },
                };
            };
            if crate::execution::ownership::is_managed(&slot.ty) {
                self.vacate_slot(&slot)?;
            }
        }
        if self.common_region.is_some() {
            emit_instruction! {
                self;
                let "%mal_active_environment" = alloca {
                    ty: (ptr),
                    alignment: #{ self.types.pointer_alignment() },
                };
            };
            let environment = self.register();
            emit_instruction! {
                self;
                let #{ environment.clone() } = call {
                    tail: false,
                    result_type: (ptr),
                    callee: direct("mal_runtime_environment_retain"),
                    arguments: [typed((ptr), "%mal_context"), typed((ptr), "%mal_environment")],
                };
            };
            emit_instruction! {
                self;
                store {
                    value: typed((ptr), #{ environment }),
                    pointer: "%mal_active_environment",
                    alignment: #{ self.types.pointer_alignment() },
                    metadata: [],
                };
            };
        }
        if let Some((size, alignment)) = self.external_storage {
            let storage_type = llvm_type!(array(#{ size }, int(8)));
            emit_instruction! {
                self;
                let "%mal_bridge_argument" = alloca {
                    ty: #{ storage_type.clone() },
                    alignment: #{ alignment },
                };
            };
            emit_instruction! {
                self;
                let "%mal_bridge_result" = alloca {
                    ty: #{ storage_type },
                    alignment: #{ alignment },
                };
            };
        }
        if self.needs_symbol_result_slot {
            let symbol = self.types.value(&Type::Symbol)?;
            emit_instruction! {
                self;
                let "%mal_symbol_result" = alloca {
                    ty: #{ symbol.llvm },
                    alignment: #{ symbol.alignment },
                };
            };
        }
        if let Some((size, alignment)) = self.buffer_value_storage {
            emit_instruction! {
                self;
                let "%mal_buffer_value" = alloca {
                    ty: #{ llvm_type!(array(#{ size }, int(8_u16))) },
                    alignment: #{ alignment },
                };
            };
        }
        if self.mode == EmissionMode::Native && self.native_worker_parameters().is_some() {
            self.emit_native_context_entry()?;
        }
        if self.mode == EmissionMode::Native {
            self.emit_native_entry_guard()?;
        }
        let parameter_destination = self.execution.parameters.destination(self.function.id)?;
        if matches!(parameter_destination, ParameterDestination::Bind(_))
            || crate::execution::ownership::is_managed(&self.function.parameter.ty)
        {
            let entry = self.ownership.native_entry(self.function.id);
            let parameter = EmittedValue {
                ty: self.function.parameter.ty.clone(),
                representation: if self.function.parameter.ty == Type::Unit {
                    "0".into()
                } else {
                    "%mal_parameter".into()
                },
                owned: entry == crate::execution::ownership::ParameterEntry::OwnedAbi,
            };
            self.emit_parameter_handoff(self.function.id, &parameter, entry)?;
        }
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
