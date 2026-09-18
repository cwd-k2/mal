use std::collections::HashSet;

use crate::anf::ast::ValueId;
use crate::closure::ast::Pattern;

use super::managed::managed_paths;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum PatternHandoff {
    Unmanaged,
    Store(ValueId),
    Drop,
    Product(Vec<Self>),
}

impl PatternHandoff {
    pub(super) fn has_owner_successor(&self) -> bool {
        match self {
            Self::Store(_) => true,
            Self::Product(elements) => elements.iter().any(Self::has_owner_successor),
            Self::Unmanaged | Self::Drop => false,
        }
    }
}

pub(super) fn plan_pattern(pattern: &Pattern, live_after: &HashSet<ValueId>) -> PatternHandoff {
    match pattern {
        Pattern::Binding { ty, .. } if managed_paths(ty).next().is_none() => {
            PatternHandoff::Unmanaged
        }
        Pattern::Binding { id, .. } if live_after.contains(id) => PatternHandoff::Store(*id),
        Pattern::Binding { .. } => PatternHandoff::Drop,
        Pattern::Product { elements, .. } => PatternHandoff::Product(
            elements
                .iter()
                .map(|element| plan_pattern(element, live_after))
                .collect(),
        ),
        Pattern::Wildcard { ty, .. } if managed_paths(ty).next().is_some() => PatternHandoff::Drop,
        Pattern::Wildcard { .. } => PatternHandoff::Unmanaged,
    }
}
