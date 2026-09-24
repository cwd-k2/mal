use std::collections::{HashMap, HashSet};

use crate::anf::ast::ValueId;
use crate::closure::ast::{Atom, AtomKind, Pattern, Reference};
use crate::control::ast::{Operation, Program, StateId};

pub(super) struct ParameterPassThrough<'a> {
    bindings: HashMap<ValueId, &'a Operation>,
}

impl<'a> ParameterPassThrough<'a> {
    pub(super) fn new(program: &'a Program) -> Self {
        let bindings = program
            .states
            .iter()
            .flat_map(|state| &state.bindings)
            .filter_map(|binding| {
                let Pattern::Binding { id, .. } = binding.pattern else {
                    return None;
                };
                Some((id, &binding.operation))
            })
            .collect();
        Self { bindings }
    }

    pub(super) fn parameter_pattern<'p>(
        &self,
        program: &'p Program,
        entry: StateId,
        parameter: ValueId,
    ) -> Option<&'p Pattern> {
        program.states[entry.0].bindings.iter().find_map(|binding| {
            matches!(
                binding.operation,
                Operation::Atom(Atom {
                    kind: AtomKind::Reference(Reference::Binding(id)),
                    ..
                }) if id == parameter
            )
            .then_some(&binding.pattern)
        })
    }

    pub(super) fn fields(&self, pattern: &Pattern, argument: &Atom) -> HashSet<ValueId> {
        let mut preserved = HashSet::new();
        self.collect(pattern, argument, &mut preserved);
        preserved
    }

    fn collect(&self, pattern: &Pattern, argument: &Atom, preserved: &mut HashSet<ValueId>) {
        match pattern {
            Pattern::Binding { id, .. } => {
                if self.resolves_to_binding(argument, *id) {
                    preserved.insert(*id);
                }
            }
            Pattern::Product { elements, .. } => {
                let Some(arguments) = self.product_elements(argument) else {
                    return;
                };
                if elements.len() == arguments.len() {
                    for (element, argument) in elements.iter().zip(arguments) {
                        self.collect(element, argument, preserved);
                    }
                }
            }
            Pattern::Wildcard { .. } => {}
        }
    }

    fn resolves_to_binding(&self, atom: &Atom, expected: ValueId) -> bool {
        let AtomKind::Reference(Reference::Binding(id)) = atom.kind else {
            return false;
        };
        id == expected
            || self
                .bindings
                .get(&id)
                .is_some_and(|operation| match operation {
                    Operation::Atom(alias) => self.resolves_to_binding(alias, expected),
                    _ => false,
                })
    }

    fn product_elements(&self, atom: &Atom) -> Option<&'a [Atom]> {
        let AtomKind::Reference(Reference::Binding(id)) = atom.kind else {
            return None;
        };
        match self.bindings.get(&id)? {
            Operation::Product(elements) => Some(elements),
            Operation::Atom(alias) => self.product_elements(alias),
            _ => None,
        }
    }
}
