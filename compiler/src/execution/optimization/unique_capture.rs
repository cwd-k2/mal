use std::collections::{HashMap, HashSet};

use crate::closure::ast::{self as closure, Atom, AtomId, AtomKind, FunctionId, Reference};
use crate::control::ast::{Operation, Program, Terminator};

use super::super::ApplicationGraph;

pub(super) fn plan(
    lowered: &closure::Program,
    program: &Program,
    applications: &ApplicationGraph,
) -> HashSet<AtomId> {
    program
        .functions
        .iter()
        .filter(|function| {
            !is_recursive(function.id, applications)
                && has_one_final_entry_invocation(
                    function.id,
                    lowered.entry.map(|entry| entry.function),
                    program,
                    applications,
                )
        })
        .flat_map(|function| {
            let states = super::super::application::reachable_states(program, function.entry);
            let mut capture_uses = HashMap::<usize, usize>::new();
            for site in &states {
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

fn has_one_final_entry_invocation(
    function: FunctionId,
    entry: Option<FunctionId>,
    program: &Program,
    applications: &ApplicationGraph,
) -> bool {
    let Some(entry) = entry else {
        return false;
    };
    if is_recursive(entry, applications)
        || applications.sites().any(|(site, _)| {
            applications
                .targets(site)
                .is_some_and(|targets| targets.contains(&entry))
        })
    {
        return false;
    }
    let mut sites = applications
        .sites()
        .filter(|(site, _)| {
            applications
                .targets(*site)
                .is_some_and(|targets| targets.contains(&function))
        })
        .map(|(site, _)| site);
    let Some(site) = sites.next() else {
        return false;
    };
    if sites.next().is_some() {
        return false;
    }
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

fn is_recursive(function: FunctionId, applications: &ApplicationGraph) -> bool {
    let mut pending = vec![function];
    let mut seen = HashSet::new();
    while let Some(caller) = pending.pop() {
        if !seen.insert(caller) {
            continue;
        }
        for (_, targets) in applications.sites_from(caller) {
            if targets.contains(&function) {
                return true;
            }
            pending.extend(targets.iter().copied());
        }
    }
    false
}

fn visit_state_atoms(state: &crate::control::ast::State, mut visit: impl FnMut(&Atom)) {
    for binding in &state.bindings {
        visit_operation_atoms(&binding.operation, &mut visit);
    }
    visit_terminator_atoms(&state.terminator, visit);
}

fn visit_operation_atoms(operation: &Operation, visit: &mut impl FnMut(&Atom)) {
    match operation {
        Operation::Atom(atom)
        | Operation::SymbolLength { value: atom }
        | Operation::SymbolAt { argument: atom }
        | Operation::Buffer { argument: atom, .. }
        | Operation::ExternalCall { argument: atom, .. }
        | Operation::NumericConversion { operand: atom }
        | Operation::SumInjection { value: atom, .. }
        | Operation::PrimitiveUnary { operand: atom, .. } => visit(atom),
        Operation::MakeClosure { captures, .. }
        | Operation::Memory {
            operands: captures, ..
        }
        | Operation::Product(captures) => captures.iter().for_each(visit),
        Operation::PrimitiveBinary { left, right, .. } => {
            visit(left);
            visit(right);
        }
    }
}

fn visit_terminator_atoms(terminator: &Terminator, mut visit: impl FnMut(&Atom)) {
    match terminator {
        Terminator::Return(atom)
        | Terminator::Jump { value: atom, .. }
        | Terminator::Case {
            scrutinee: atom, ..
        } => visit(atom),
        Terminator::Call {
            callee, argument, ..
        }
        | Terminator::TailCall { callee, argument } => {
            visit(callee);
            visit(argument);
        }
        Terminator::PrimitiveBranch { left, right, .. } => {
            visit(left);
            visit(right);
        }
        Terminator::Goto(_) => {}
    }
}
