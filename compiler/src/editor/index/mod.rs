use std::collections::{HashMap, HashSet};

use crate::check::ast as checked;
use crate::resolve::ast as resolved;
use crate::source::{FileId, Span};

use super::{Occurrence, OccurrenceRole, SemanticDocument, Symbol, SymbolId, SymbolKind};

mod aliases;
mod checked_ast;
mod resolved_ast;

pub(super) fn build(
    resolved: &resolved::Program,
    checked: &checked::Program,
    file: Option<FileId>,
    visible: Option<&HashSet<FileId>>,
) -> SemanticDocument {
    Index::new(resolved, checked).finish(file, visible)
}

struct Index<'a> {
    checked: &'a checked::Program,
    aliases: HashMap<resolved::ValueId, resolved::ValueId>,
    value_types: HashMap<resolved::ValueId, String>,
    type_details: HashMap<resolved::TypeId, String>,
    parameters: HashSet<resolved::ValueId>,
    typed_regions: Vec<(Span, String)>,
    raw_occurrences: Vec<RawOccurrence>,
    top_level: Vec<SymbolId>,
}

struct RawOccurrence {
    id: SymbolId,
    name: String,
    span: Span,
    role: OccurrenceRole,
}

impl<'a> Index<'a> {
    fn new(resolved: &'a resolved::Program, checked: &'a checked::Program) -> Self {
        let mut index = Self {
            checked,
            aliases: HashMap::new(),
            value_types: HashMap::new(),
            type_details: crate::resolve::PREDEFINED_TYPES
                .iter()
                .map(|&(name, id)| (id, name.to_owned()))
                .collect(),
            parameters: HashSet::new(),
            typed_regions: Vec::new(),
            raw_occurrences: Vec::new(),
            top_level: Vec::new(),
        };
        for item in &resolved.items {
            index.collect_aliases_top(&item.kind);
        }
        for item in &checked.items {
            index.collect_checked_top(&item.kind);
        }
        for item in &resolved.items {
            index.collect_resolved_top(&item.kind);
        }
        index
    }

    fn finish(
        mut self,
        file: Option<FileId>,
        visible: Option<&HashSet<FileId>>,
    ) -> SemanticDocument {
        let raw_occurrences = std::mem::take(&mut self.raw_occurrences);
        let mut occurrences = raw_occurrences
            .into_iter()
            .map(|raw| {
                let id = self.canonical_symbol(raw.id);
                Occurrence {
                    kind: self.kind(id),
                    detail: self.detail(id),
                    id,
                    name: raw.name,
                    span: raw.span,
                    role: raw.role,
                }
            })
            .collect::<Vec<_>>();
        occurrences.sort_by_key(|occurrence| {
            (
                occurrence.span.file().index(),
                occurrence.span.start(),
                occurrence.span.end(),
            )
        });

        let top_level = self
            .top_level
            .iter()
            .filter_map(|&id| symbol_for(id, &occurrences))
            .collect::<Vec<_>>();
        let document_symbols = top_level
            .iter()
            .filter(|symbol| {
                file.is_none_or(|file| symbol.span.is_some_and(|span| span.file() == file))
            })
            .cloned()
            .collect::<Vec<_>>();
        let mut completions = predefined_symbols();
        completions.extend(top_level.into_iter().filter(|symbol| {
            let Some(span) = symbol.span else {
                return false;
            };
            file.is_none_or(|file| {
                span.file() == file
                    || (visible.is_some_and(|visible| visible.contains(&span.file()))
                        && !symbol.name.starts_with('_'))
            })
        }));
        completions.sort_by(|left, right| left.name.cmp(&right.name));
        completions.dedup_by(|left, right| left.name == right.name);

        SemanticDocument {
            file,
            occurrences,
            typed_regions: self.typed_regions,
            document_symbols,
            completions,
        }
    }

    fn canonical_value(&self, mut id: resolved::ValueId) -> resolved::ValueId {
        while let Some(&source) = self.aliases.get(&id) {
            id = source;
        }
        id
    }

    fn canonical_symbol(&self, id: SymbolId) -> SymbolId {
        match id {
            SymbolId::Value(id) => SymbolId::Value(self.canonical_value(id)),
            other => other,
        }
    }

    fn kind(&self, id: SymbolId) -> SymbolKind {
        match id {
            SymbolId::Type(_) => SymbolKind::Type,
            SymbolId::ExternalOperation(_) => SymbolKind::Function,
            SymbolId::Value(id) if self.parameters.contains(&id) => SymbolKind::Parameter,
            SymbolId::Value(id)
                if self
                    .value_types
                    .get(&id)
                    .is_some_and(|ty| ty.contains(" -> ")) =>
            {
                SymbolKind::Function
            }
            SymbolId::Value(_) => SymbolKind::Value,
        }
    }

    fn detail(&self, id: SymbolId) -> Option<String> {
        match id {
            SymbolId::Type(id) => self.type_details.get(&id).cloned(),
            SymbolId::Value(id) => self.value_types.get(&id).cloned(),
            SymbolId::ExternalOperation(id) => {
                self.checked.items.iter().find_map(|item| match &item.kind {
                    checked::TopItem::ExternalOperation {
                        id: item_id,
                        parameter,
                        result,
                        ..
                    } if *item_id == id => Some(format!(
                        "{} -> {}",
                        crate::check::type_name(parameter),
                        crate::check::type_name(result)
                    )),
                    _ => None,
                })
            }
        }
    }

    fn add_raw(&mut self, id: SymbolId, name: &crate::ast::Name, role: OccurrenceRole) {
        self.raw_occurrences.push(RawOccurrence {
            id,
            name: name.text.clone(),
            span: name.span,
            role,
        });
    }
}
fn symbol_for(id: SymbolId, occurrences: &[Occurrence]) -> Option<Symbol> {
    let occurrence = occurrences
        .iter()
        .find(|occurrence| occurrence.id == id && occurrence.role == OccurrenceRole::Declaration)?;
    Some(Symbol {
        id,
        name: occurrence.name.clone(),
        kind: occurrence.kind,
        detail: occurrence.detail.clone(),
        span: Some(occurrence.span),
    })
}

fn predefined_symbols() -> Vec<Symbol> {
    let types = crate::resolve::PREDEFINED_TYPES
        .iter()
        .map(|&(name, id)| Symbol {
            id: SymbolId::Type(id),
            name: name.to_owned(),
            kind: SymbolKind::Type,
            detail: Some(name.to_owned()),
            span: None,
        });
    let values = crate::resolve::PREDEFINED_VALUES
        .iter()
        .map(|&(name, id)| Symbol {
            id: SymbolId::Value(id),
            name: name.to_owned(),
            kind: if id.0 >= 2 {
                SymbolKind::Function
            } else {
                SymbolKind::Value
            },
            detail: None,
            span: None,
        });
    types.chain(values).collect()
}
