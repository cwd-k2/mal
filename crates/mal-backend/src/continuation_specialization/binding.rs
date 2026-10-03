//! Symbolic environment lookup and binding to closure-program patterns.

use std::collections::HashMap;

use mal_syntax::source::Span;

use crate::anf::ast::ValueId;
use crate::closure::ast::{Atom, AtomKind, Binding, FunctionId, Pattern, Reference};
use crate::closure::rewrite::Identities;

use super::symbolic::{Environment, Value};

pub(super) fn resolve(
    atom: &Atom,
    environment: &Environment,
    known: &HashMap<ValueId, FunctionId>,
) -> Option<Value> {
    match atom.kind {
        AtomKind::Reference(Reference::Binding(binding)) => {
            environment.values.get(&binding).cloned().or_else(|| {
                known.get(&binding).copied().map(|function| Value::Closure {
                    function,
                    captures: Vec::new(),
                    ty: atom.ty.clone(),
                })
            })
        }
        AtomKind::Reference(Reference::Capture(index)) => environment.captures.get(index).cloned(),
        AtomKind::Reference(Reference::SelfClosure(function)) => Some(Value::Closure {
            function,
            captures: environment.captures.clone(),
            ty: atom.ty.clone(),
        }),
        _ => Some(Value::Literal(
            atom.kind.clone(),
            atom.ty.clone(),
            atom.span,
        )),
    }
}

pub(super) fn bind(
    pattern: &Pattern,
    value: Value,
    environment: &mut Environment,
    output: &mut Vec<Binding>,
    ids: &mut Identities,
    fallback_span: Span,
) -> Option<()> {
    match (pattern, value) {
        (Pattern::Binding { id, .. }, value) => {
            environment.values.insert(*id, value);
            Some(())
        }
        (Pattern::Wildcard { .. }, value) => {
            (!matches!(value, Value::Closure { .. })).then_some(())
        }
        (Pattern::Product { elements, .. }, Value::Product(values, _, _))
            if elements.len() == values.len() =>
        {
            for (element, value) in elements.iter().zip(values) {
                bind(element, value, environment, output, ids, fallback_span)?;
            }
            Some(())
        }
        (Pattern::Product { .. }, value) => {
            let atom = super::materialize::value(value, output, ids, fallback_span)?;
            let span = atom.span;
            let pattern = fresh_pattern(pattern, environment, ids);
            output.push(Binding {
                pattern,
                operation: crate::closure::ast::Operation::Atom(atom),
                span,
            });
            Some(())
        }
    }
}

pub(super) fn fresh_pattern(
    pattern: &Pattern,
    environment: &mut Environment,
    ids: &mut Identities,
) -> Pattern {
    match pattern {
        Pattern::Binding { id, ty } => {
            let fresh = ids.value();
            environment
                .values
                .insert(*id, Value::Bound(fresh, ty.clone()));
            Pattern::Binding {
                id: fresh,
                ty: ty.clone(),
            }
        }
        Pattern::Wildcard { ty, span } => Pattern::Wildcard {
            ty: ty.clone(),
            span: *span,
        },
        Pattern::Product { elements, ty, span } => Pattern::Product {
            elements: elements
                .iter()
                .map(|element| fresh_pattern(element, environment, ids))
                .collect(),
            ty: ty.clone(),
            span: *span,
        },
    }
}
