//! Fresh instantiation of admitted principal kind schemes.

use std::collections::HashMap;

use crate::check::ast::Kind;

pub(super) fn freshen_all(kinds: &[Kind], next: &mut u32) -> Vec<Kind> {
    fn visit(kind: &Kind, variables: &mut HashMap<u32, u32>, next: &mut u32) -> Kind {
        match kind {
            Kind::Type => Kind::Type,
            Kind::Variable(id) => Kind::Variable(*variables.entry(*id).or_insert_with(|| {
                let fresh = *next;
                *next = next.checked_add(1).expect("kind identity space");
                fresh
            })),
            Kind::Function { parameter, result } => Kind::function(
                visit(parameter, variables, next),
                visit(result, variables, next),
            ),
        }
    }
    let mut variables = HashMap::new();
    kinds
        .iter()
        .map(|kind| visit(kind, &mut variables, next))
        .collect()
}
