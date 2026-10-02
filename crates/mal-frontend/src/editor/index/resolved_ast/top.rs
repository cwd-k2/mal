//! Top-level declarations: the identities they introduce and the types and bodies they contain.

use crate::resolve::ast as resolved;
use mal_syntax::ast::{Name, Node};
use mal_syntax::source::Span;

use super::super::Index;
use super::super::type_display::type_name;
use crate::editor::{OccurrenceRole, SymbolId};

impl Index {
    pub(in crate::editor::index) fn collect_resolved_top(
        &mut self,
        item: &Node<resolved::TopItem>,
    ) {
        match &item.kind {
            resolved::TopItem::TypeAlias { binding, value } => {
                self.type_details.insert(binding.id, type_name(value));
                self.declare_top(SymbolId::Type(binding.id), &binding.name, item.span);
                self.collect_resolved_type(value);
            }
            resolved::TopItem::GenericTypeAlias {
                binding,
                parameters,
                value,
            } => {
                self.type_details.insert(binding.id, type_name(value));
                self.declare_top(SymbolId::Type(binding.id), &binding.name, item.span);
                self.declare_parameters(parameters);
                self.collect_resolved_type(value);
            }
            resolved::TopItem::OpaqueType {
                binding,
                parameters,
                representation,
            } => {
                self.type_details
                    .insert(binding.id, binding.name.text.clone());
                self.declare_top(SymbolId::Type(binding.id), &binding.name, item.span);
                self.declare_parameters(parameters);
                self.collect_resolved_type(representation);
            }
            resolved::TopItem::ExternalType { binding } => {
                self.declare_top(SymbolId::Type(binding.id), &binding.name, item.span);
            }
            resolved::TopItem::ExternalOperation { binding, ty, .. } => {
                self.value_types.insert(binding.id, type_name(ty));
                self.functions.insert(binding.id);
                self.declare_top(SymbolId::Value(binding.id), &binding.name, item.span);
                self.collect_resolved_type(ty);
            }
            resolved::TopItem::Binding(binding) => {
                self.collect_resolved_binding(binding, true, item.span);
            }
            resolved::TopItem::GenericBinding {
                binding,
                parameters,
                annotation,
                value,
            } => {
                self.value_types.insert(binding.id, type_name(annotation));
                self.declare_top(SymbolId::Value(binding.id), &binding.name, item.span);
                self.declare_parameters(parameters);
                self.collect_resolved_type(annotation);
                self.collect_resolved_expression_with_expected(value, Some(annotation));
            }
            resolved::TopItem::OperationFamily {
                binding,
                parameters,
                annotation,
            } => {
                self.value_types.insert(binding.id, type_name(annotation));
                self.declare_top(SymbolId::Value(binding.id), &binding.name, item.span);
                self.declare_parameters(parameters);
                self.collect_resolved_type(annotation);
            }
            resolved::TopItem::OperationImplementation {
                family,
                parameters,
                arguments,
                annotation,
                value,
                ..
            } => {
                self.add_raw(
                    SymbolId::Value(family.id),
                    &family.name,
                    OccurrenceRole::Reference,
                    None,
                );
                self.declare_parameters(parameters);
                self.collect_implementation_key(arguments, parameters);
                self.collect_resolved_type(annotation);
                self.collect_resolved_expression_with_expected(value, Some(annotation));
            }
        }
    }

    fn declare_top(&mut self, id: SymbolId, name: &Name, span: Span) {
        self.top_level.push(id);
        self.add_raw(id, name, OccurrenceRole::Declaration, Some(span));
    }

    fn declare_parameters(&mut self, parameters: &[resolved::TypeBinding]) {
        for parameter in parameters {
            self.add_raw(
                SymbolId::Type(parameter.id),
                &parameter.name,
                OccurrenceRole::Declaration,
                None,
            );
        }
    }

    /// A key binder is declared at its first occurrence in the key, so the key walk leaves that occurrence out
    /// instead of recording it again as a reference.
    fn collect_implementation_key(
        &mut self,
        arguments: &[Node<resolved::TypeExpression>],
        binders: &[resolved::TypeBinding],
    ) {
        let start = self.raw_occurrences.len();
        for argument in arguments {
            self.collect_resolved_type(argument);
        }
        let mut occurrences = self.raw_occurrences.split_off(start);
        occurrences.retain(|occurrence| {
            !binders
                .iter()
                .any(|binder| binder.name.span == occurrence.span)
        });
        self.raw_occurrences.extend(occurrences);
    }
}
