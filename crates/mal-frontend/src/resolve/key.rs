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
