use std::collections::HashSet;

use crate::anf::ast::ValueId;
use crate::closure::ast::Pattern;

use super::is_managed;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum PatternDestination {
    Unmanaged,
    Initialize(ValueId),
    Borrow(ValueId),
    Discard,
    Product(Vec<Self>),
}

impl PatternDestination {
    pub(super) fn has_owner_successor(&self) -> bool {
        match self {
            Self::Initialize(_) => true,
            Self::Product(elements) => elements.iter().any(Self::has_owner_successor),
            Self::Unmanaged | Self::Borrow(_) | Self::Discard => false,
        }
    }
}

pub(super) fn plan_borrowed_pattern(
    pattern: &Pattern,
    live_after: &HashSet<ValueId>,
    borrowed_bindings: &HashSet<ValueId>,
) -> PatternDestination {
    match pattern {
        Pattern::Binding { ty, .. } if !is_managed(ty) => PatternDestination::Unmanaged,
        Pattern::Binding { id, .. }
            if live_after.contains(id) && borrowed_bindings.contains(id) =>
        {
            PatternDestination::Borrow(*id)
        }
        Pattern::Binding { id, .. } if live_after.contains(id) => {
            PatternDestination::Initialize(*id)
        }
        Pattern::Binding { .. } => PatternDestination::Discard,
        Pattern::Product { elements, .. } => PatternDestination::Product(
            elements
                .iter()
                .map(|element| plan_borrowed_pattern(element, live_after, borrowed_bindings))
                .collect(),
        ),
        Pattern::Wildcard { ty, .. } if is_managed(ty) => PatternDestination::Discard,
        Pattern::Wildcard { .. } => PatternDestination::Unmanaged,
    }
}
