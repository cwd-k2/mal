use std::collections::{HashMap, HashSet};

use crate::anf::ast::ValueId;
use crate::closure::ast::Pattern;
use crate::control::ast::{Operation, Program, StateId, Terminator};

use super::liveness::{
    collect_pattern_binding_order, managed_binding_id, remove_pattern_bindings, terminator_live,
    visit_operation_atoms,
};
use super::parameter::ParameterBorrows;

pub(super) fn collect(
    control: &Program,
    parameters: &ParameterBorrows,
    live_in: &[HashSet<ValueId>],
) -> HashMap<ValueId, HashSet<ValueId>> {
    let mut authorities = parameters
        .bindings
        .iter()
        .map(|binding| (*binding, HashSet::new()))
        .collect::<HashMap<_, _>>();
    let mut deferred_aliases = HashMap::<ValueId, Vec<ValueId>>::new();
    let mut discarded_results = HashSet::new();
    collect_aliases(
        control,
        live_in,
        &mut authorities,
        &mut deferred_aliases,
        &mut discarded_results,
    );
    collect_case_payloads(control, live_in, &mut authorities);
    collect_bounded_arguments(control, parameters, &mut discarded_results);
    trace_pure_construction(control, discarded_results, &mut authorities);
    resolve_aliases(&mut authorities, &mut deferred_aliases);
    remove_unbounded_aliases(control, &mut authorities);
    authorities
}

fn collect_aliases(
    control: &Program,
    live_in: &[HashSet<ValueId>],
    authorities: &mut HashMap<ValueId, HashSet<ValueId>>,
    deferred: &mut HashMap<ValueId, Vec<ValueId>>,
    discarded: &mut HashSet<ValueId>,
) {
    for state in &control.states {
        let mut live = terminator_live(&state.terminator, live_in);
        for binding in state.bindings.iter().rev() {
            if let Operation::Atom(atom) = &binding.operation
                && let Some(source) = managed_binding_id(atom)
            {
                let mut bindings = Vec::new();
                collect_pattern_binding_order(&binding.pattern, &mut bindings);
                for borrowed in &bindings {
                    if !live.contains(borrowed) {
                        continue;
                    }
                    if let Some(lenders) = authorities.get(&source).cloned() {
                        authorities.insert(*borrowed, lenders);
                    } else if live.contains(&source) {
                        authorities.insert(*borrowed, HashSet::from([source]));
                    } else {
                        deferred.entry(source).or_default().push(*borrowed);
                    }
                }
                if !live.contains(&source) && !bindings.iter().any(|binding| live.contains(binding))
                {
                    discarded.insert(source);
                }
            }
            remove_pattern_bindings(&binding.pattern, &mut live);
            visit_operation_atoms(&binding.operation, |atom| {
                if let Some(binding) = managed_binding_id(atom) {
                    live.insert(binding);
                }
            });
        }
    }
}

fn collect_case_payloads(
    control: &Program,
    live_in: &[HashSet<ValueId>],
    authorities: &mut HashMap<ValueId, HashSet<ValueId>>,
) {
    for state in &control.states {
        let Terminator::Case { scrutinee, arms } = &state.terminator else {
            continue;
        };
        let Some(source) = managed_binding_id(scrutinee) else {
            continue;
        };
        for arm in arms {
            let target = &control.states[arm.target.0];
            if !target.live.iter().any(|value| value.id == source) {
                continue;
            }
            let Some(input) = &target.input else {
                continue;
            };
            let live = baseline_body_live(target, live_in);
            let mut bindings = Vec::new();
            collect_pattern_binding_order(input, &mut bindings);
            for borrowed in bindings {
                if live.contains(&borrowed) {
                    authorities.insert(borrowed, HashSet::from([source]));
                }
            }
        }
    }
}

fn baseline_body_live(
    state: &crate::control::ast::State,
    live_in: &[HashSet<ValueId>],
) -> HashSet<ValueId> {
    let mut live = terminator_live(&state.terminator, live_in);
    for binding in state.bindings.iter().rev() {
        remove_pattern_bindings(&binding.pattern, &mut live);
        visit_operation_atoms(&binding.operation, |atom| {
            if let Some(binding) = managed_binding_id(atom) {
                live.insert(binding);
            }
        });
    }
    live
}

fn collect_bounded_arguments(
    control: &Program,
    parameters: &ParameterBorrows,
    discarded: &mut HashSet<ValueId>,
) {
    for (state_index, state) in control.states.iter().enumerate() {
        let site = StateId(state_index);
        if parameters.call_sites.contains(&site)
            && let Terminator::Call { argument, .. } | Terminator::TailCall { argument, .. } =
                &state.terminator
            && let Some(source) = managed_binding_id(argument)
        {
            discarded.insert(source);
        }
    }
}

fn trace_pure_construction(
    control: &Program,
    discarded: HashSet<ValueId>,
    authorities: &mut HashMap<ValueId, HashSet<ValueId>>,
) {
    let mut definitions = HashMap::new();
    let mut input_states = HashMap::new();
    for (state_index, state) in control.states.iter().enumerate() {
        if let Some(Pattern::Binding { id, .. }) = &state.input {
            input_states.insert(*id, StateId(state_index));
        }
        for binding in &state.bindings {
            if let Pattern::Binding { id, .. } = &binding.pattern {
                definitions.insert(*id, &binding.operation);
            }
        }
    }
    let mut pending = discarded.into_iter().collect::<Vec<_>>();
    let mut visited = HashSet::new();
    while let Some(discarded) = pending.pop() {
        if !visited.insert(discarded) {
            continue;
        }
        if let Some(operation) = definitions.get(&discarded) {
            let sources: HashSet<ValueId> = match operation {
                Operation::Atom(atom) => managed_binding_id(atom).into_iter().collect(),
                Operation::Product(elements) => {
                    elements.iter().filter_map(managed_binding_id).collect()
                }
                Operation::SumInjection { value, .. } => {
                    managed_binding_id(value).into_iter().collect()
                }
                _ => continue,
            };
            pending.extend(sources.iter().copied());
            authorities.insert(discarded, sources);
            continue;
        }
        let Some(target) = input_states.get(&discarded) else {
            continue;
        };
        let sources = control
            .states
            .iter()
            .filter_map(|state| match &state.terminator {
                Terminator::Jump {
                    target: successor,
                    value,
                } if successor == target => managed_binding_id(value),
                _ => None,
            })
            .collect::<Vec<_>>();
        if !sources.is_empty() {
            authorities.insert(discarded, HashSet::new());
            pending.extend(sources);
        }
    }
}

fn resolve_aliases(
    authorities: &mut HashMap<ValueId, HashSet<ValueId>>,
    deferred: &mut HashMap<ValueId, Vec<ValueId>>,
) {
    let mut resolved = authorities.keys().copied().collect::<Vec<_>>();
    while let Some(source) = resolved.pop() {
        let Some(lenders) = authorities.get(&source).cloned() else {
            continue;
        };
        for borrowed in deferred.remove(&source).unwrap_or_default() {
            if authorities.insert(borrowed, lenders.clone()).is_none() {
                resolved.push(borrowed);
            }
        }
    }
}

fn remove_unbounded_aliases(
    control: &Program,
    authorities: &mut HashMap<ValueId, HashSet<ValueId>>,
) {
    let mut invalid = HashSet::new();
    for state in &control.states {
        let live = state
            .live
            .iter()
            .map(|value| value.id)
            .collect::<HashSet<_>>();
        for borrowed in &live {
            if let Some(sources) = authorities.get(borrowed)
                && !sources.iter().all(|source| live.contains(source))
            {
                invalid.insert(*borrowed);
            }
        }
    }
    authorities.retain(|borrowed, _| !invalid.contains(borrowed));
}
