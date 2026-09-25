//! Records the functions a pattern can hold, filtered by what its type can hold.

use crate::anf::ast::ValueId;
use crate::closure::ast::{self as closure, FunctionId, Pattern};
use crate::control::ast::StateId;
use mal_frontend::check::ast::Type;

use super::analysis::{Analysis, Functions, merge};

impl Analysis<'_> {
    pub(super) fn assign_parameter(&mut self, function: FunctionId, value: &Functions) {
        if let Some(&(Some(binding), ty)) = self.parameters.get(&function) {
            self.assign_binding(binding, ty, value);
        }
    }

    pub(super) fn assign_input(&mut self, target: StateId, value: &Functions) {
        if let Some(input) = &self.control.states[target.0].input {
            self.assign(input, value);
        }
    }

    pub(super) fn assign(&mut self, pattern: &Pattern, value: &Functions) {
        match pattern {
            Pattern::Binding { id, ty } => self.assign_binding(*id, ty, value),
            Pattern::Product { elements, .. } => {
                for element in elements {
                    self.assign(element, value);
                }
            }
            Pattern::Wildcard { .. } => {}
        }
    }

    pub(super) fn assign_top_level(
        &mut self,
        pattern: &closure::TopLevelPattern,
        value: &Functions,
    ) {
        match pattern {
            closure::TopLevelPattern::Binding { id, ty, .. } => self.assign_binding(*id, ty, value),
            closure::TopLevelPattern::Product { elements, .. } => {
                for element in elements {
                    self.assign_top_level(element, value);
                }
            }
            closure::TopLevelPattern::Wildcard { .. } => {}
        }
    }

    /// Records the functions a binding of type `ty` can hold; a binding that cannot hold a function records none.
    pub(super) fn assign_binding(&mut self, id: ValueId, ty: &Type, value: &Functions) {
        let held = match ty {
            Type::Function { parameter, result } => value
                .iter()
                .copied()
                .filter(|function| {
                    self.signatures
                        .get(function)
                        .is_some_and(|(p, r)| *p == parameter.as_ref() && *r == result.as_ref())
                })
                .collect(),
            _ if holds_function(ty) => value.clone(),
            _ => Functions::new(),
        };
        if !held.is_empty() {
            let slot = self.values.entry(id).or_default();
            self.changed |= merge(slot, &held);
        }
    }
}

fn holds_function(ty: &Type) -> bool {
    let mut pending = vec![ty];
    while let Some(ty) = pending.pop() {
        match ty {
            Type::Function { .. } => return true,
            Type::Product(elements) | Type::Sum(elements) => pending.extend(elements.iter()),
            Type::Buffer(element) => pending.push(element),
            _ => {}
        }
    }
    false
}
