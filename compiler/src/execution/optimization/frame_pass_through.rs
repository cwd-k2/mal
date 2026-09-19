use std::collections::{HashMap, HashSet};

use crate::anf::ast::ValueId;
use crate::closure::ast::{Atom, AtomKind, Pattern, Reference};
use crate::control::ast::{Operation, Program, StateId, Terminator};

use super::super::ApplicationGraph;

pub(super) fn plan(
    program: &Program,
    applications: &ApplicationGraph,
) -> HashMap<StateId, HashSet<ValueId>> {
    let mut result = HashMap::new();
    let bindings = binding_operations(program);
    for function in &program.functions {
        if has_mutual_recursion(function.id, applications) {
            continue;
        }
        let recursive_sites = applications
            .sites_from(function.id)
            .filter(|(_, targets)| targets.contains(&function.id))
            .collect::<Vec<_>>();
        if recursive_sites.is_empty()
            || recursive_sites
                .iter()
                .any(|(_, targets)| *targets != [function.id])
        {
            continue;
        }
        let Some(parameter) = function.parameter.binding else {
            continue;
        };
        let Some(pattern) = parameter_pattern(program, function.entry, parameter) else {
            continue;
        };
        let mut common = None::<HashSet<ValueId>>;
        for (site, _) in &recursive_sites {
            let Some(argument) = call_argument(&program.states[site.0].terminator) else {
                common = Some(HashSet::new());
                break;
            };
            let mut preserved = HashSet::new();
            collect_preserved(&bindings, pattern, argument, &mut preserved);
            common = Some(match common {
                Some(current) => current.intersection(&preserved).copied().collect(),
                None => preserved,
            });
        }
        let common = common.unwrap_or_default();
        for (site, _) in recursive_sites {
            let Terminator::Call { resume, .. } = program.states[site.0].terminator else {
                continue;
            };
            let fields = program.states[resume.0]
                .live
                .iter()
                .filter_map(|field| common.contains(&field.id).then_some(field.id))
                .collect::<HashSet<_>>();
            if !fields.is_empty() {
                result.insert(site, fields);
            }
        }
    }
    result
}

fn has_mutual_recursion(
    function: crate::closure::ast::FunctionId,
    applications: &ApplicationGraph,
) -> bool {
    let mut pending = applications
        .sites_from(function)
        .flat_map(|(_, targets)| targets.iter().copied())
        .filter(|target| *target != function)
        .collect::<Vec<_>>();
    let mut seen = HashSet::new();
    while let Some(current) = pending.pop() {
        if !seen.insert(current) {
            continue;
        }
        for (_, targets) in applications.sites_from(current) {
            if targets.contains(&function) {
                return true;
            }
            pending.extend(targets.iter().copied());
        }
    }
    false
}

fn parameter_pattern(program: &Program, entry: StateId, parameter: ValueId) -> Option<&Pattern> {
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

fn call_argument(terminator: &Terminator) -> Option<&Atom> {
    match terminator {
        Terminator::Call { argument, .. } | Terminator::TailCall { argument, .. } => Some(argument),
        _ => None,
    }
}

fn collect_preserved(
    bindings: &HashMap<ValueId, &Operation>,
    pattern: &Pattern,
    argument: &Atom,
    preserved: &mut HashSet<ValueId>,
) {
    match pattern {
        Pattern::Binding { id, .. } => {
            if resolves_to_binding(bindings, argument, *id) {
                preserved.insert(*id);
            }
        }
        Pattern::Product { elements, .. } => {
            let Some(arguments) = product_elements(bindings, argument) else {
                return;
            };
            if elements.len() == arguments.len() {
                for (element, argument) in elements.iter().zip(arguments) {
                    collect_preserved(bindings, element, argument, preserved);
                }
            }
        }
        Pattern::Wildcard { .. } => {}
    }
}

fn resolves_to_binding(
    bindings: &HashMap<ValueId, &Operation>,
    atom: &Atom,
    expected: ValueId,
) -> bool {
    let AtomKind::Reference(Reference::Binding(id)) = atom.kind else {
        return false;
    };
    if id == expected {
        return true;
    }
    bindings.get(&id).is_some_and(|operation| match operation {
        Operation::Atom(alias) => resolves_to_binding(bindings, alias, expected),
        _ => false,
    })
}

fn product_elements<'a>(
    bindings: &HashMap<ValueId, &'a Operation>,
    atom: &'a Atom,
) -> Option<&'a [Atom]> {
    let AtomKind::Reference(Reference::Binding(id)) = atom.kind else {
        return None;
    };
    match bindings.get(&id)? {
        Operation::Product(elements) => Some(elements),
        Operation::Atom(alias) => product_elements(bindings, alias),
        _ => None,
    }
}

fn binding_operations(program: &Program) -> HashMap<ValueId, &Operation> {
    program
        .states
        .iter()
        .flat_map(|state| &state.bindings)
        .filter_map(|binding| {
            let Pattern::Binding { id, .. } = binding.pattern else {
                return None;
            };
            Some((id, &binding.operation))
        })
        .collect()
}
