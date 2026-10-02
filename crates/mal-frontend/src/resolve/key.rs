//! Binders of an operation implementation key. A key is a type pattern: every name in an argument position
//! that is not a visible type is a variable the pattern introduces, as a value pattern introduces the names it
//! binds. Constructor positions always name types.

use std::collections::HashSet;

use mal_syntax::ast::{Name, Node, TypeExpression};

/// Names in `arguments` that `is_type` does not recognize, in order of first occurrence.
pub(super) fn key_binders(
    arguments: &[Node<TypeExpression>],
    is_type: impl Fn(&str) -> bool,
) -> Vec<Name> {
    let mut binders = Vec::new();
    let mut seen = HashSet::new();
    let mut pending = arguments.iter().rev().collect::<Vec<_>>();
    while let Some(ty) = pending.pop() {
        match &ty.kind {
            TypeExpression::Named(name) => {
                if !is_type(&name.text) && seen.insert(name.text.clone()) {
                    binders.push(name.clone());
                }
            }
            TypeExpression::Application { arguments, .. }
            | TypeExpression::Product(arguments)
            | TypeExpression::Sum(arguments) => pending.extend(arguments.iter().rev()),
            TypeExpression::Parenthesized(inner) => pending.push(inner),
            TypeExpression::Function { parameter, result } => {
                pending.push(result);
                pending.push(parameter);
            }
            TypeExpression::Unit => {}
        }
    }
    binders
}

/// The visible type name one edit away from `binder`, so that a diagnostic can ask whether the binder is a misspelled
/// type. A single-letter binder is the usual spelling of a type variable and is never suspected. Among several
/// candidates the first in name order is chosen, so the note does not depend on table order.
pub(super) fn similar_type<'a>(
    binder: &str,
    types: impl IntoIterator<Item = &'a str>,
) -> Option<&'a str> {
    if binder.chars().count() < 2 {
        return None;
    }
    types
        .into_iter()
        .filter(|name| *name != binder && within_one_edit(binder, name))
        .min()
}

/// Whether one insertion, deletion, substitution, or transposition of adjacent characters turns `left` into `right`.
fn within_one_edit(left: &str, right: &str) -> bool {
    let left = left.chars().collect::<Vec<_>>();
    let right = right.chars().collect::<Vec<_>>();
    let (shorter, longer) = if left.len() <= right.len() {
        (&left, &right)
    } else {
        (&right, &left)
    };
    match longer.len() - shorter.len() {
        0 => {
            let differences = (0..shorter.len())
                .filter(|&index| shorter[index] != longer[index])
                .collect::<Vec<_>>();
            match differences.as_slice() {
                [_] => true,
                [first, second] => {
                    *second == first + 1
                        && shorter[*first] == longer[*second]
                        && shorter[*second] == longer[*first]
                }
                _ => false,
            }
        }
        1 => {
            let prefix = shorter
                .iter()
                .zip(longer.iter())
                .take_while(|(left, right)| left == right)
                .count();
            shorter[prefix..] == longer[prefix + 1..]
        }
        _ => false,
    }
}
