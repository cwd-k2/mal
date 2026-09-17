use super::*;
impl<'a> FunctionEmitter<'a> {
    pub(super) fn new(
        execution: &'a crate::execution::Program,
        index: &'a ProgramIndex<'a>,
        id: FunctionId,
        target: super::super::TargetLayout,
        top_levels: &'a TopLevelConstants,
        ownership: &'a ownership::Plan,
        optimizations: &'a super::super::optimization::OptimizationPlan,
    ) -> Option<Self> {
        let types = Types::for_target(target)?;
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
                let entry = index
                    .control_functions
                    .get(function)
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
            let region_function = *index.control_functions.get(function_id)?;
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
            let mut seen = std::collections::HashSet::new();
            for id in ids {
                if seen.insert(id) {
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
        let frame_tags = frame_sites
            .iter()
            .enumerate()
            .map(|(tag, site)| Some((*site, u32::try_from(tag).ok()?)))
            .collect::<Option<HashMap<_, _>>>()?;
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
            let external = *index.externals.get(&id)?;
            for ty in [&external.parameter, &external.result] {
                let value = types.value(ty)?;
                let (size, alignment) = external_storage.unwrap_or((0usize, 1usize));
                external_storage = Some((size.max(value.size), alignment.max(value.alignment)));
            }
        }
        Some(Self {
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
            function_slots,
            frame_sites,
            frame_tags,
            external_storage,
            types,
            source_layouts,
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
                self.types.index_alignment()
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
                self.types.pointer_alignment()
            ));
            let environment = self.register();
            self.line(format!(
                "  {environment} = call ptr @mal_runtime_environment_retain(ptr %mal_context, ptr %mal_environment)"
            ));
            self.line(format!(
                "  store ptr {environment}, ptr %mal_active_environment, align {}",
                self.types.pointer_alignment()
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
