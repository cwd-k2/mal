//! Construction of declaration, occurrence, type, exit, symbol, and completion indexes.

use std::collections::{HashMap, HashSet};

use crate::check::ast as checked;
use crate::resolve::ast as resolved;
use mal_syntax::source::{FileId, Span};

use super::{Exit, Occurrence, OccurrenceRole, SemanticDocument, Symbol, SymbolId, SymbolKind};

mod aliases;
mod checked_ast;
mod exits;
mod expression_display;
mod predefined;
mod resolved_ast;
mod type_display;

use exits::merged_exits;

pub(super) fn build(
    resolved: &resolved::Program,
    checked: &checked::Program,
    file: Option<FileId>,
    visible: Option<&HashSet<FileId>>,
) -> SemanticDocument {
    Index::new(resolved, checked).finish(file, visible)
}

struct Index {
    aliases: HashMap<resolved::ValueId, resolved::ValueId>,
    value_types: HashMap<resolved::ValueId, String>,
    type_details: HashMap<resolved::TypeId, String>,
    type_aliases: HashMap<resolved::TypeId, mal_syntax::ast::Node<resolved::TypeExpression>>,
    generic_type_aliases: HashMap<resolved::TypeId, GenericTypeAlias>,
    declared_value_types:
        HashMap<resolved::ValueId, mal_syntax::ast::Node<resolved::TypeExpression>>,
    generic_value_types: HashMap<resolved::ValueId, GenericValueType>,
    inferred_type_arguments: HashMap<Span, Vec<String>>,
    functions: HashSet<resolved::ValueId>,
    parameters: HashSet<resolved::ValueId>,
    result_binders: HashSet<resolved::ValueId>,
    /// Binder names by result block and position in its binder group.
    result_binder_names: HashMap<(resolved::ValueId, Option<usize>), String>,
    typed_regions: Vec<(Span, String)>,
    exits: Vec<Exit>,
    raw_occurrences: Vec<RawOccurrence>,
    top_level: Vec<SymbolId>,
}

#[derive(Clone)]
struct GenericTypeAlias {
    parameters: Vec<resolved::TypeId>,
    value: mal_syntax::ast::Node<resolved::TypeExpression>,
}

#[derive(Clone)]
struct GenericValueType {
    parameters: Vec<resolved::TypeId>,
    ty: mal_syntax::ast::Node<resolved::TypeExpression>,
}

struct RawOccurrence {
    id: SymbolId,
    name: String,
    span: Span,
    role: OccurrenceRole,
    detail: Option<String>,
    declaration_span: Option<Span>,
}

impl Index {
    fn new(resolved: &resolved::Program, checked: &checked::Program) -> Self {
        let mut index = Self {
            aliases: HashMap::new(),
            value_types: predefined::value_types(),
            type_details: predefined::type_details(),
            type_aliases: HashMap::new(),
            generic_type_aliases: HashMap::new(),
            declared_value_types: HashMap::new(),
            generic_value_types: HashMap::new(),
            inferred_type_arguments: HashMap::new(),
            functions: predefined::functions(),
            parameters: HashSet::new(),
            result_binders: HashSet::new(),
            result_binder_names: HashMap::new(),
            typed_regions: Vec::new(),
            exits: Vec::new(),
            raw_occurrences: Vec::new(),
            top_level: Vec::new(),
        };
        for item in &resolved.items {
            index.collect_aliases_top(&item.kind);
            match &item.kind {
                resolved::TopItem::TypeAlias { binding, value } => {
                    index.type_aliases.insert(binding.id, value.clone());
                }
                resolved::TopItem::GenericTypeAlias {
                    binding,
                    parameters,
                    value,
                } => {
                    index.generic_type_aliases.insert(
                        binding.id,
                        GenericTypeAlias {
                            parameters: parameters.iter().map(|parameter| parameter.id).collect(),
                            value: value.clone(),
                        },
                    );
                }
                resolved::TopItem::OpaqueType { .. } => {}
                resolved::TopItem::ExternalOperation { binding, ty, .. } => {
                    index.declared_value_types.insert(binding.id, ty.clone());
                }
                resolved::TopItem::Binding(binding) => {
                    if let (resolved::Pattern::Binding(name), Some(annotation)) =
                        (&binding.pattern.kind, &binding.annotation)
                    {
                        index
                            .declared_value_types
                            .insert(name.id, annotation.clone());
                    }
                }
                resolved::TopItem::GenericBinding {
                    binding,
                    parameters,
                    annotation,
                    ..
                } => {
                    index.generic_value_types.insert(
                        binding.id,
                        GenericValueType {
                            parameters: parameters.iter().map(|parameter| parameter.id).collect(),
                            ty: annotation.clone(),
                        },
                    );
                }
                resolved::TopItem::OperationFamily {
                    binding,
                    parameters,
                    annotation,
                } => {
                    index.generic_value_types.insert(
                        binding.id,
                        GenericValueType {
                            parameters: parameters.iter().map(|parameter| parameter.id).collect(),
                            ty: annotation.clone(),
                        },
                    );
                }
                resolved::TopItem::OperationImplementation { .. } => {}
                resolved::TopItem::ExternalType { .. } => {}
            }
        }
        for item in &checked.items {
            index.collect_checked_top(&item.kind);
        }
        for item in &resolved.items {
            index.collect_resolved_top(item);
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
                    detail: raw.detail.or_else(|| self.detail(id)),
                    documentation: predefined::documentation(id).map(str::to_owned),
                    id,
                    name: raw.name,
                    span: raw.span,
                    role: raw.role,
                    declaration_span: raw.declaration_span,
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

        let declarations = occurrences
            .iter()
            .filter(|occurrence| occurrence.role == OccurrenceRole::Declaration)
            .map(|occurrence| (occurrence.id, occurrence))
            .collect::<HashMap<_, _>>();

        let top_level = self
            .top_level
            .iter()
            .filter_map(|id| {
                declarations
                    .get(id)
                    .map(|occurrence| symbol_for(occurrence))
            })
            .collect::<Vec<_>>();
        let document_symbols = top_level
            .iter()
            .filter(|symbol| {
                file.is_none_or(|file| symbol.span.is_some_and(|span| span.file() == file))
            })
            .cloned()
            .collect::<Vec<_>>();
        let mut completions = predefined::symbols();
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
            exits: merged_exits(self.exits),
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
            SymbolId::Value(id) if self.result_binders.contains(&id) => SymbolKind::ResultBinder,
            SymbolId::Value(id) if self.parameters.contains(&id) => SymbolKind::Parameter,
            SymbolId::Value(id) if self.functions.contains(&id) => SymbolKind::Function,
            SymbolId::Value(_) => SymbolKind::Value,
        }
    }

    fn detail(&self, id: SymbolId) -> Option<String> {
        match id {
            SymbolId::Type(id) => self.type_details.get(&id).cloned(),
            SymbolId::Value(id) => self.value_types.get(&id).cloned(),
        }
    }

    fn add_raw(
        &mut self,
        id: SymbolId,
        name: &mal_syntax::ast::Name,
        role: OccurrenceRole,
        declaration_span: Option<Span>,
    ) {
        self.add_raw_with_detail(id, name, role, declaration_span, None);
    }

    fn add_raw_with_detail(
        &mut self,
        id: SymbolId,
        name: &mal_syntax::ast::Name,
        role: OccurrenceRole,
        declaration_span: Option<Span>,
        detail: Option<String>,
    ) {
        self.raw_occurrences.push(RawOccurrence {
            id,
            name: name.text.clone(),
            span: name.span,
            role,
            detail,
            declaration_span,
        });
    }
}
fn symbol_for(occurrence: &Occurrence) -> Symbol {
    Symbol {
        id: occurrence.id,
        name: occurrence.name.clone(),
        kind: occurrence.kind,
        detail: occurrence.detail.clone(),
        documentation: occurrence.documentation.clone(),
        span: Some(occurrence.span),
    }
}
