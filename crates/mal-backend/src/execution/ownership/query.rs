//! The facts the backend reads from a finished plan: drops, destinations, use effects, and parameter conventions.

use super::*;

impl Plan {
    pub(crate) fn drops_after_binding(&self, state: StateId, binding: usize) -> &[ValueId] {
        self.drops_after_binding
            .get(&(state, binding))
            .map_or(&[], Vec::as_slice)
    }

    pub(crate) fn input_destination(&self, state: StateId) -> Option<&PatternDestination> {
        self.input_destinations.get(&state)
    }

    pub(crate) fn binding_destination(
        &self,
        state: StateId,
        binding: usize,
    ) -> Option<&PatternDestination> {
        self.binding_destinations.get(&(state, binding))
    }

    pub(crate) fn drops_on_edge(&self, state: StateId, path: ControlPath) -> &[ValueId] {
        self.drops_on_edge
            .get(&EdgeId { state, path })
            .map_or(&[], Vec::as_slice)
    }

    pub(crate) fn binding_use(
        &self,
        state: StateId,
        binding: usize,
        operand: BindingOperand,
    ) -> Option<UseEffect> {
        self.uses
            .get(&UseId {
                state,
                location: UseLocation::Binding { binding, operand },
            })
            .copied()
    }

    /// How binding `binding` at `state` uses its operand `atom`. A value without managed parts carries no
    /// responsibility, so it is borrowed whether or not the plan records it.
    pub(crate) fn binding_operand_use(
        &self,
        state: StateId,
        binding: usize,
        operand: BindingOperand,
        atom: &crate::closure::ast::Atom,
    ) -> Option<UseEffect> {
        self.binding_use(state, binding, operand)
            .or_else(|| (!is_managed(&atom.ty)).then_some(UseEffect::Borrow))
    }

    /// How the terminator of `state` uses its operand `atom`, with unmanaged values borrowed as above.
    pub(crate) fn terminator_operand_use(
        &self,
        state: StateId,
        operand: TerminatorOperand,
        atom: &crate::closure::ast::Atom,
    ) -> Option<UseEffect> {
        self.terminator_use(state, operand)
            .or_else(|| (!is_managed(&atom.ty)).then_some(UseEffect::Borrow))
    }

    pub(crate) fn terminator_use(
        &self,
        state: StateId,
        operand: TerminatorOperand,
    ) -> Option<UseEffect> {
        self.uses
            .get(&UseId {
                state,
                location: UseLocation::Terminator(operand),
            })
            .copied()
    }

    pub(crate) fn frame_field_use(&self, state: StateId, field: usize) -> Option<UseEffect> {
        self.uses
            .get(&UseId {
                state,
                location: UseLocation::FrameField(field),
            })
            .copied()
    }

    /// How the native entry of `function` receives its managed argument.
    pub(crate) fn native_entry(&self, function: FunctionId) -> ParameterEntry {
        if self.owned_functions.contains(&function) {
            ParameterEntry::OwnedAbi
        } else {
            ParameterEntry::BorrowedAbi
        }
    }

    /// Whether the native call at `site` hands its managed argument to the callee.
    pub(crate) fn passes_owned_argument(&self, site: StateId) -> bool {
        self.owned_sites.contains(&site)
    }

    pub(crate) fn binding_is_borrowed(&self, binding: ValueId) -> bool {
        self.borrowed_bindings.contains(&binding)
    }

    pub(crate) fn case_payload_use(&self, state: StateId, arm: usize) -> Option<UseEffect> {
        self.uses
            .get(&UseId {
                state,
                location: UseLocation::CasePayload(arm),
            })
            .copied()
    }

    pub(crate) fn parameter_effect(
        &self,
        function: FunctionId,
        entry: ParameterEntry,
    ) -> Option<ParameterEffect> {
        self.parameters.get(&(function, entry)).copied()
    }
}
