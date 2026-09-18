use std::collections::HashSet;

use crate::anf::ast::ValueId;
use crate::closure::ast::Pattern;

use super::is_managed;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum PatternDestination {
    Unmanaged,
    Store(ValueId),
    Discard,
    Product(Vec<Self>),
}

impl PatternDestination {
    pub(super) fn has_owner_successor(&self) -> bool {
        match self {
            Self::Store(_) => true,
            Self::Product(elements) => elements.iter().any(Self::has_owner_successor),
            Self::Unmanaged | Self::Discard => false,
        }
    }
}

pub(super) fn plan_pattern(pattern: &Pattern, live_after: &HashSet<ValueId>) -> PatternDestination {
    match pattern {
        Pattern::Binding { ty, .. } if !is_managed(ty) => PatternDestination::Unmanaged,
        Pattern::Binding { id, .. } if live_after.contains(id) => PatternDestination::Store(*id),
        Pattern::Binding { .. } => PatternDestination::Discard,
        Pattern::Product { elements, .. } => PatternDestination::Product(
            elements
                .iter()
                .map(|element| plan_pattern(element, live_after))
                .collect(),
        ),
        Pattern::Wildcard { ty, .. } if is_managed(ty) => PatternDestination::Discard,
        Pattern::Wildcard { .. } => PatternDestination::Unmanaged,
    }
}
