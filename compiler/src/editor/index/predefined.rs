use std::collections::{HashMap, HashSet};

use crate::resolve::ast as resolved;

use super::{Symbol, SymbolId, SymbolKind};

pub(super) fn value_types() -> HashMap<resolved::ValueId, String> {
    crate::resolve::PREDEFINED_VALUES
        .iter()
        .filter_map(|entry| entry.detail.map(|detail| (entry.id, detail.to_owned())))
        .collect()
}

pub(super) fn type_details() -> HashMap<resolved::TypeId, String> {
    crate::resolve::PREDEFINED_TYPES
        .iter()
        .map(|entry| (entry.id, entry.detail.to_owned()))
        .collect()
}

pub(super) fn functions() -> HashSet<resolved::ValueId> {
    crate::resolve::PREDEFINED_VALUES
        .iter()
        .filter(|entry| entry.callable)
        .map(|entry| entry.id)
        .collect()
}

pub(super) fn symbols() -> Vec<Symbol> {
    let types = crate::resolve::PREDEFINED_TYPES.iter().map(|entry| Symbol {
        id: SymbolId::Type(entry.id),
        name: entry.name.to_owned(),
        kind: SymbolKind::Type,
        detail: Some(entry.detail.to_owned()),
        documentation: Some(entry.documentation.to_owned()),
        span: None,
    });
    let values = crate::resolve::PREDEFINED_VALUES
        .iter()
        .map(|entry| Symbol {
            id: SymbolId::Value(entry.id),
            name: entry.name.to_owned(),
            kind: if entry.callable {
                SymbolKind::Function
            } else {
                SymbolKind::Value
            },
            detail: entry.detail.map(str::to_owned),
            documentation: Some(entry.documentation.to_owned()),
            span: None,
        });
    types.chain(values).collect()
}

pub(super) fn documentation(id: SymbolId) -> Option<&'static str> {
    match id {
        SymbolId::Type(id) => crate::resolve::PREDEFINED_TYPES
            .iter()
            .find(|entry| entry.id == id)
            .map(|entry| entry.documentation),
        SymbolId::Value(id) => crate::resolve::PREDEFINED_VALUES
            .iter()
            .find(|entry| entry.id == id)
            .map(|entry| entry.documentation),
    }
}
