//! The control states one emitted function owns, and the slots their bindings need.

use super::*;

/// The states of the emitted function, or of every function in its common control region, in emission order.
pub(super) struct RegionStates {
    function_states: Vec<(FunctionId, Vec<StateId>)>,
    pub(super) states: Vec<StateId>,
    pub(super) state_functions: HashMap<StateId, FunctionId>,
}

impl RegionStates {
    pub(super) fn collect(index: &ProgramIndex<'_>, functions: &[FunctionId]) -> Option<Self> {
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
        // A state shared by two functions would be emitted twice.
        (state_functions.len() == states.len()).then_some(Self {
            function_states,
            states,
            state_functions,
        })
    }

    /// One slot per bound parameter, state input, and binding pattern name.
    pub(super) fn slots(
        &self,
        execution: &crate::execution::Program,
        index: &ProgramIndex<'_>,
        types: &Types,
    ) -> Option<HashMap<ValueId, Slot>> {
        let mut slots = HashMap::new();
        for (function_id, function_states) in &self.function_states {
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
        Some(slots)
    }
}
