use std::collections::{HashMap, HashSet};

use crate::closure::ast::{Atom, AtomKind, FunctionId, Pattern, Reference};
use crate::control::ast::{Operation, Program, Terminator};

use super::pass_through::ParameterPassThrough;
use super::{ApplicationGraph, ControlCallMode, ControlCallPlan, OwnershipPlan};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct SelfTailParameter {
    pub(crate) pattern: Pattern,
    pub(crate) binding_count: usize,
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct SelfTailParameterPlan {
    entries: HashMap<FunctionId, SelfTailParameter>,
}

impl SelfTailParameterPlan {
    pub(crate) fn new(
        control: &Program,
        applications: &ApplicationGraph,
        calls: &ControlCallPlan,
        ownership: &OwnershipPlan,
    ) -> Self {
        let mut plan = Self::candidates(control, applications, calls);
        plan.entries
            .retain(|_, parameter| managed_bindings_are_borrowed(&parameter.pattern, ownership));
        plan
    }

    pub(crate) fn candidates(
        control: &Program,
        applications: &ApplicationGraph,
        calls: &ControlCallPlan,
    ) -> Self {
        let pass_through = ParameterPassThrough::new(control);
        let use_counts = crate::control::binding_use_counts(control);
        let entries = control
            .functions
            .iter()
            .filter_map(|function| {
                let sites = applications
                    .sites_from(function.id)
                    .filter_map(|(site, _)| {
                        (calls.mode(site) == Some(ControlCallMode::DirectSelfTail)).then_some(site)
                    })
                    .collect::<Vec<_>>();
                if sites.is_empty() {
                    return None;
                }
                let parameter = function.parameter.binding?;
                let bindings = &control.states[function.entry.0].bindings;
                let first = bindings.first()?;
                if referenced_binding(&first.operation) != Some(parameter) {
                    return None;
                }
                let preserved = sites
                    .iter()
                    .map(|site| {
                        let argument = calls.forwarded_self_argument(*site).or_else(|| {
                            let Terminator::TailCall { argument, .. } =
                                &control.states[site.0].terminator
                            else {
                                return None;
                            };
                            Some(argument)
                        })?;
                        Some(pass_through.fields(&first.pattern, argument))
                    })
                    .collect::<Option<Vec<_>>>()?;
                let mut pattern = first.pattern.clone();
                let mut binding_count = 1;
                for binding in &bindings[1..] {
                    let Some(source) = referenced_binding(&binding.operation) else {
                        break;
                    };
                    let Some(source_type) = binding_type(&pattern, source) else {
                        break;
                    };
                    if super::ownership::is_managed(source_type) {
                        break;
                    }
                    if use_counts.get(&source) != Some(&1)
                        || !replace_binding(&mut pattern, source, &binding.pattern)
                    {
                        return None;
                    }
                    binding_count += 1;
                }
                managed_bindings_are_preserved(&pattern, &preserved).then_some((
                    function.id,
                    SelfTailParameter {
                        pattern,
                        binding_count,
                    },
                ))
            })
            .collect();
        Self { entries }
    }

    pub(crate) fn get(&self, function: FunctionId) -> Option<&SelfTailParameter> {
        self.entries.get(&function)
    }

    pub(crate) fn functions(&self) -> impl Iterator<Item = FunctionId> + '_ {
        self.entries.keys().copied()
    }

    pub(crate) fn is_valid(
        &self,
        control: &Program,
        applications: &ApplicationGraph,
        calls: &ControlCallPlan,
        ownership: &OwnershipPlan,
    ) -> bool {
        self == &Self::new(control, applications, calls, ownership)
    }

    pub(crate) fn persistent_lenders(
        &self,
        borrowing_functions: &HashSet<FunctionId>,
    ) -> HashSet<crate::anf::ast::ValueId> {
        self.entries
            .iter()
            .filter(|(function, _)| borrowing_functions.contains(function))
            .map(|(_, parameter)| parameter)
            .flat_map(|parameter| managed_bindings(&parameter.pattern))
            .collect()
    }
}

fn managed_bindings_are_borrowed(pattern: &Pattern, ownership: &OwnershipPlan) -> bool {
    match pattern {
        Pattern::Binding { id, ty } if super::ownership::is_managed(ty) => {
            ownership.binding_is_borrowed(*id)
        }
        Pattern::Product { elements, .. } => elements
            .iter()
            .all(|element| managed_bindings_are_borrowed(element, ownership)),
        Pattern::Wildcard { ty, .. } if super::ownership::is_managed(ty) => false,
        Pattern::Binding { .. } | Pattern::Wildcard { .. } => true,
    }
}

fn managed_bindings_are_preserved(
    pattern: &Pattern,
    preserved: &[HashSet<crate::anf::ast::ValueId>],
) -> bool {
    match pattern {
        Pattern::Binding { id, ty } if super::ownership::is_managed(ty) => {
            preserved.iter().all(|bindings| bindings.contains(id))
        }
        Pattern::Product { elements, .. } => elements
            .iter()
            .all(|element| managed_bindings_are_preserved(element, preserved)),
        Pattern::Wildcard { ty, .. } if super::ownership::is_managed(ty) => false,
        Pattern::Binding { .. } | Pattern::Wildcard { .. } => true,
    }
}

fn managed_bindings(pattern: &Pattern) -> Vec<crate::anf::ast::ValueId> {
    match pattern {
        Pattern::Binding { id, ty } if super::ownership::is_managed(ty) => vec![*id],
        Pattern::Product { elements, .. } => elements.iter().flat_map(managed_bindings).collect(),
        Pattern::Binding { .. } | Pattern::Wildcard { .. } => Vec::new(),
    }
}

fn referenced_binding(operation: &Operation) -> Option<crate::anf::ast::ValueId> {
    let Operation::Atom(Atom {
        kind: AtomKind::Reference(Reference::Binding(id)),
        ..
    }) = operation
    else {
        return None;
    };
    Some(*id)
}

fn binding_type(
    pattern: &Pattern,
    source: crate::anf::ast::ValueId,
) -> Option<&crate::check::ast::Type> {
    match pattern {
        Pattern::Binding { id, ty } if *id == source => Some(ty),
        Pattern::Product { elements, .. } => elements
            .iter()
            .find_map(|element| binding_type(element, source)),
        Pattern::Binding { .. } | Pattern::Wildcard { .. } => None,
    }
}

fn replace_binding(
    pattern: &mut Pattern,
    source: crate::anf::ast::ValueId,
    replacement: &Pattern,
) -> bool {
    match pattern {
        Pattern::Binding { id, ty } if *id == source && *ty == pattern_type(replacement) => {
            *pattern = replacement.clone();
            true
        }
        Pattern::Product { elements, .. } => elements
            .iter_mut()
            .any(|element| replace_binding(element, source, replacement)),
        Pattern::Binding { .. } | Pattern::Wildcard { .. } => false,
    }
}

fn pattern_type(pattern: &Pattern) -> crate::check::ast::Type {
    match pattern {
        Pattern::Binding { ty, .. }
        | Pattern::Wildcard { ty, .. }
        | Pattern::Product { ty, .. } => ty.clone(),
    }
}

#[cfg(test)]
#[path = "self_tail_parameter_tests.rs"]
mod tests;
