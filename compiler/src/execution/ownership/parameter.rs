use std::collections::HashMap;

use crate::anf::ast::ValueId;
use crate::closure::ast::FunctionId;

use super::super::{ParameterDestination, ParameterPlan};
use super::managed::is_managed;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(crate) enum ParameterEntry {
    BorrowedAbi,
    OwnedHandoff,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ParameterEffect {
    ShareInto(ValueId),
    ConsumeInto(ValueId),
    Drop,
}

pub(super) fn collect_parameter_effects(
    control: &crate::control::ast::Program,
    parameters: &ParameterPlan,
) -> HashMap<(FunctionId, ParameterEntry), ParameterEffect> {
    let mut effects = HashMap::new();
    for function in &control.functions {
        if !is_managed(&function.parameter.ty) {
            continue;
        }
        match parameters
            .destination(function.id)
            .expect("every control function has a parameter destination")
        {
            ParameterDestination::Bind(binding)
                if control.states[function.entry.0]
                    .live
                    .iter()
                    .any(|value| value.id == binding) =>
            {
                effects.insert(
                    (function.id, ParameterEntry::BorrowedAbi),
                    ParameterEffect::ShareInto(binding),
                );
                effects.insert(
                    (function.id, ParameterEntry::OwnedHandoff),
                    ParameterEffect::ConsumeInto(binding),
                );
            }
            ParameterDestination::Bind(_) | ParameterDestination::Discard => {
                effects.insert(
                    (function.id, ParameterEntry::OwnedHandoff),
                    ParameterEffect::Drop,
                );
            }
        }
    }
    effects
}
