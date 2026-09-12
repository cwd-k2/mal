use super::*;
impl<'a> FunctionEmitter<'a> {
    pub(super) fn new(
        execution: &'a crate::execution::Program,
        id: FunctionId,
        types: Types,
        top_levels: &'a TopLevelConstants,
        ownership: &'a ownership::Plan,
        optimizations: &'a super::super::optimization::OptimizationPlan,
    ) -> Option<Self> {
        let function = execution
            .control
            .functions
            .iter()
            .find(|function| function.id == id)?;
        let lowered = execution
            .lowered
            .functions
            .iter()
            .find(|function| function.id == id)?;
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
                let entry = execution
                    .control
                    .functions
                    .iter()
                    .find(|candidate| candidate.id == *function)
                    .expect("control region function has an entry")
                    .entry;
                (*function, reachable_states(&execution.control, entry))
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
        let mut function_slots = HashMap::new();
        for (function_id, function_states) in &function_states {
            let region_function = execution
                .control
                .functions
                .iter()
                .find(|candidate| candidate.id == *function_id)?;
            let mut ids = Vec::new();
            if let ParameterDestination::Bind(id) =
                execution.parameters.destination(*function_id)?
            {
                insert_slot(&mut slots, id, region_function.parameter.ty.clone());
                ids.push(id);
            }
            for state in function_states {
                let state = &execution.control.states[state.0];
                if let Some(pattern) = &state.input {
                    collect_pattern_slot(pattern, &mut slots, types)?;
                    collect_pattern_ids(pattern, &mut ids);
                }
                for binding in &state.bindings {
                    collect_pattern_slot(&binding.pattern, &mut slots, types)?;
                    collect_pattern_ids(&binding.pattern, &mut ids);
                }
            }
            let mut unique = Vec::new();
            for id in ids {
                if !unique.contains(&id) {
                    unique.push(id);
                }
            }
            function_slots.insert(*function_id, unique);
        }
        let frame_sites = states
            .iter()
            .filter(|site| execution.control_frames.frame(**site).is_some())
            .copied()
            .collect::<Vec<_>>();
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
        let mut external_storage = None;
        for id in external_ids {
            let external = execution
                .lowered
                .interface
                .externals
                .iter()
                .find(|external| external.id == id)?;
            for ty in [&external.parameter, &external.result] {
                if !super::super::bridge_type_supported(ty) {
                    return None;
                }
                let value = types.value(ty)?;
                let (size, alignment) = external_storage.unwrap_or((0usize, 1usize));
                external_storage = Some((size.max(value.size), alignment.max(value.alignment)));
            }
        }
        Some(Self {
            execution,
            control: &execution.control,
            function,
            current_function: id,
            common_region,
            result_type: lowered.body.result.ty.clone(),
            states,
            state_functions,
            slots,
            function_slots,
            frame_sites,
            external_storage,
            types,
            top_levels,
            ownership,
            optimizations,
            next_register: 0,
            globals: String::new(),
            output: String::new(),
        })
    }

    pub(super) fn emit(mut self) -> Option<EmittedFunction> {
        self.emit_environment_destructor()?;
        let parameter = if self.function.parameter.ty == Type::Unit {
            "ptr %mal_context, ptr %mal_control_top, ptr %mal_environment".to_string()
        } else {
            let parameter = self.types.value(&self.function.parameter.ty)?;
            format!(
                "ptr %mal_context, ptr %mal_control_top, ptr %mal_environment, {} %mal_parameter",
                parameter.llvm
            )
        };
        let result = self.types.value(&self.result_type)?;
        self.line(format!(
            "define internal {} @{}({parameter}) {{",
            result.llvm,
            function_name(self.function.id)?
        ));
        self.line("entry:");
        if !self.frame_sites.is_empty() {
            self.line(format!(
                "  %mal_control_base = load {}, ptr %mal_control_top, align {}",
                self.types.pointer_integer()?,
                self.types.pointer_size()
            ));
        }
        let mut slots = self.slots.values().cloned().collect::<Vec<_>>();
        slots.sort_by_key(|slot| slot.index);
        for slot in slots {
            let value_type = self.types.value(&slot.ty)?;
            self.line(format!(
                "  %mal_slot_{} = alloca {}, align {}",
                slot.index, value_type.llvm, value_type.alignment
            ));
            if crate::execution::ownership::is_managed(&slot.ty) {
                self.line(format!(
                    "  store {} zeroinitializer, ptr %mal_slot_{}, align {}",
                    value_type.llvm, slot.index, value_type.alignment
                ));
            }
        }
        if self.common_region.is_some() {
            self.line(format!(
                "  %mal_active_environment = alloca ptr, align {}",
                self.types.pointer_size()
            ));
            let environment = self.register();
            self.line(format!(
                "  {environment} = call ptr @mal_runtime_environment_retain(ptr %mal_context, ptr %mal_environment)"
            ));
            self.line(format!(
                "  store ptr {environment}, ptr %mal_active_environment, align {}",
                self.types.pointer_size()
            ));
        }
        if let Some((size, alignment)) = self.external_storage {
            self.line(format!(
                "  %mal_bridge_argument = alloca [{size} x i8], align {alignment}"
            ));
            self.line(format!(
                "  %mal_bridge_result = alloca [{size} x i8], align {alignment}"
            ));
        }
        if let ParameterDestination::Bind(_) =
            self.execution.parameters.destination(self.function.id)?
        {
            let mut parameter = EmittedValue {
                ty: self.function.parameter.ty.clone(),
                representation: "%mal_parameter".into(),
                owned: false,
            };
            self.retain_if_borrowed(&mut parameter)?;
            self.emit_parameter_handoff(self.function.id, &parameter)?;
        }
        self.line(format!("  br label %mal_state_{}", self.function.entry.0));

        for site in self.states.clone() {
            self.emit_state(site)?;
        }
        self.line("}");
        Some(EmittedFunction {
            globals: self.globals,
            definition: self.output,
        })
    }
}
