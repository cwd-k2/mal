use std::collections::{HashMap, HashSet};

use crate::closure::ast::{self as closure, Atom, AtomId, AtomKind, FunctionId, Reference};
use crate::control::ast::{Operation, Program, StateId, Terminator};

use super::super::ApplicationGraph;

pub(super) fn plan(
    lowered: &closure::Program,
    program: &Program,
    applications: &ApplicationGraph,
) -> HashSet<AtomId> {
    let recursive = recursive_functions(program, applications);
    let final_entry = lowered
        .entry
        .map(|entry| entry.function)
        .filter(|entry| is_final_entry(*entry, &recursive, applications));
    let invocations = invocation_sites(applications);
    program
        .functions
        .iter()
        .filter(|function| {
            !recursive.contains(&function.id)
                && final_entry.is_some_and(|entry| {
                    has_one_final_entry_invocation(
                        function.id,
                        entry,
                        program,
                        applications,
                        &invocations,
                    )
                })
        })
        .flat_map(|function| {
            let mut capture_uses = HashMap::<usize, usize>::new();
            for site in &function.states {
                visit_state_atoms(&program.states[site.0], |atom| {
                    if let AtomKind::Reference(Reference::Capture(index)) = atom.kind {
                        *capture_uses.entry(index).or_default() += 1;
                    }
                });
            }
            program.states[function.entry.0]
                .bindings
                .iter()
                .filter_map(|binding| {
                    let Operation::Atom(atom) = &binding.operation else {
                        return None;
                    };
                    let AtomKind::Reference(Reference::Capture(index)) = atom.kind else {
                        return None;
                    };
                    (crate::execution::ownership::is_managed(&atom.ty)
                        && capture_uses.get(&index) == Some(&1))
                    .then_some(atom.id)
                })
                .collect::<Vec<_>>()
        })
        .collect()
}

/// The entry runs once when nothing applies it and it cannot reach itself.
fn is_final_entry(
    entry: FunctionId,
    recursive: &HashSet<FunctionId>,
    applications: &ApplicationGraph,
) -> bool {
    !recursive.contains(&entry)
        && !applications.sites().any(|(site, _)| {
            applications
                .targets(site)
                .is_some_and(|targets| targets.contains(&entry))
        })
}

/// The application sites that may invoke each function.
fn invocation_sites(applications: &ApplicationGraph) -> HashMap<FunctionId, Vec<StateId>> {
    let mut invocations = HashMap::<FunctionId, Vec<StateId>>::new();
    for (site, _) in applications.sites() {
        for target in applications.targets(site).into_iter().flatten() {
            let sites = invocations.entry(*target).or_default();
            if sites.last() != Some(&site) {
                sites.push(site);
            }
        }
    }
    invocations
}

fn has_one_final_entry_invocation(
    function: FunctionId,
    entry: FunctionId,
    program: &Program,
    applications: &ApplicationGraph,
    invocations: &HashMap<FunctionId, Vec<StateId>>,
) -> bool {
    let Some([site]) = invocations.get(&function).map(Vec::as_slice) else {
        return false;
    };
    let site = *site;
    if applications.caller(site) != Some(entry) {
        return false;
    }
    match &program.states[site.0].terminator {
        Terminator::Call { callee, resume, .. } => {
            let AtomKind::Reference(Reference::Binding(callee)) = callee.kind else {
                return false;
            };
            !program.states[resume.0]
                .live
                .iter()
                .any(|value| value.id == callee)
        }
        Terminator::TailCall { callee, .. } => {
            matches!(callee.kind, AtomKind::Reference(Reference::Binding(_)))
        }
        _ => false,
    }
}

/// The functions that can reach themselves through the possible targets of their application sites.
fn recursive_functions(program: &Program, applications: &ApplicationGraph) -> HashSet<FunctionId> {
    let indices = program
        .functions
        .iter()
        .enumerate()
        .map(|(index, function)| (function.id, index))
        .collect::<HashMap<_, _>>();
    let mut graph = vec![Vec::new(); program.functions.len()];
    let mut recursive = HashSet::new();
    for (index, function) in program.functions.iter().enumerate() {
        for (_, targets) in applications.sites_from(function.id) {
            for target in targets {
                if *target == function.id {
                    recursive.insert(function.id);
                }
                if let Some(target) = indices.get(target) {
                    graph[index].push(*target);
                }
            }
        }
    }
    for component in super::super::region::strongly_connected_components(&graph) {
        if component.len() > 1 {
            recursive.extend(component.iter().map(|index| program.functions[*index].id));
        }
    }
    recursive
}

fn visit_state_atoms(state: &crate::control::ast::State, mut visit: impl FnMut(&Atom)) {
    for binding in &state.bindings {
        binding.operation.for_each_atom(&mut visit);
    }
    state.terminator.for_each_atom(visit);
}
