use std::collections::{HashMap, HashSet};

use crate::anf::ast::ValueId;
use crate::closure::ast::Pattern;
use crate::control::ast::{Operation, Program, StateId, Terminator};

use super::liveness::{
    collect_pattern_binding_order, managed_binding_id, remove_pattern_bindings, terminator_live,
};
use super::parameter::ParameterBorrows;
use crate::execution::EnvironmentAliasPlan;

/// Derives the managed values whose lifetime authorizes each borrowed binding.
///
/// Collection first records parameter, alias, case-payload, and bounded-call candidates. It then
/// traces discarded pure aggregates to their managed leaves, resolves deferred aliases to
/// canonical lenders, and finally removes authorities invalidated by tail environments or
/// unbounded lifetimes. Later phases rely on this order rather than repairing partial authorities.
pub(super) fn collect(
    control: &Program,
    parameters: &ParameterBorrows,
    live_in: &[HashSet<ValueId>],
    persistent_lenders: &HashSet<ValueId>,
    environment: &EnvironmentAliasPlan,
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
        environment,
    );
    collect_case_payloads(control, live_in, &mut authorities);
    collect_bounded_arguments(control, parameters, &mut discarded_results);
    trace_pure_construction(control, discarded_results, &mut authorities);
    resolve_aliases(&mut authorities, &mut deferred_aliases);
    canonicalize_lenders(&mut authorities);
    remove_environment_aliases_used_by_tail_calls(control, environment, &mut authorities);
    remove_unbounded_aliases(control, persistent_lenders, &mut authorities);
    authorities
}

fn collect_aliases(
    control: &Program,
    live_in: &[HashSet<ValueId>],
    authorities: &mut HashMap<ValueId, HashSet<ValueId>>,
    deferred: &mut HashMap<ValueId, Vec<ValueId>>,
    discarded: &mut HashSet<ValueId>,
    environment: &EnvironmentAliasPlan,
) {
    for state in &control.states {
        let mut live = terminator_live(&state.terminator, live_in);
        for binding in state.bindings.iter().rev() {
            borrow_capture_reads(binding, &live, environment, authorities);
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
            binding.operation.for_each_atom(|atom| {
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
        binding.operation.for_each_atom(|atom| {
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

/// Traces a discarded alias, product, sum, or join input back to managed source bindings.
///
/// Effectful and otherwise unknown constructions deliberately stop the trace. Join inputs collect
/// every incoming jump source so no path lends authority that another path cannot provide.
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
    let mut join_inputs = Vec::new();
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
            join_inputs.push((discarded, sources.clone()));
            pending.extend(sources);
        }
    }
    // A join input can borrow only when every value that jumps to it is itself borrowed, so its lenders
    // already outlive the jump. If any source is an owner that ends with its predecessor, the input owns
    // the value instead and each jump hands the responsibility over.
    for _ in 0..join_inputs.len() {
        for (input, sources) in &join_inputs {
            if sources
                .iter()
                .all(|source| authorities.contains_key(source))
            {
                let lenders = sources
                    .iter()
                    .flat_map(|source| authorities[source].iter().copied())
                    .collect::<HashSet<_>>();
                authorities.insert(*input, lenders);
            }
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

/// Replace intermediate aliases with the authorities that actually own their storage. An alias carrier need not
/// remain live merely because a value projected from it crosses a state boundary; only its final local lenders do.
/// A cycle without a resolvable authority is not a valid borrow proof and is discarded conservatively.
fn canonicalize_lenders(authorities: &mut HashMap<ValueId, HashSet<ValueId>>) {
    let collected = authorities.clone();
    let mut resolved = HashMap::<ValueId, Option<HashSet<ValueId>>>::new();
    for binding in collected.keys() {
        let mut visiting = HashSet::new();
        resolve_lenders(*binding, &collected, &mut visiting, &mut resolved);
    }
    authorities.clear();
    authorities.extend(
        resolved
            .into_iter()
            .filter_map(|(binding, lenders)| lenders.map(|lenders| (binding, lenders))),
    );
}

fn resolve_lenders(
    binding: ValueId,
    authorities: &HashMap<ValueId, HashSet<ValueId>>,
    visiting: &mut HashSet<ValueId>,
    resolved: &mut HashMap<ValueId, Option<HashSet<ValueId>>>,
) -> Option<HashSet<ValueId>> {
    if let Some(lenders) = resolved.get(&binding) {
        return lenders.clone();
    }
    let Some(sources) = authorities.get(&binding) else {
        return Some(HashSet::from([binding]));
    };
    if !visiting.insert(binding) {
        return None;
    }
    let mut lenders = HashSet::new();
    for source in sources {
        let Some(source_lenders) = resolve_lenders(*source, authorities, visiting, resolved) else {
            visiting.remove(&binding);
            resolved.insert(binding, None);
            return None;
        };
        lenders.extend(source_lenders);
    }
    visiting.remove(&binding);
    resolved.insert(binding, Some(lenders.clone()));
    Some(lenders)
}

fn remove_unbounded_aliases(
    control: &Program,
    persistent_lenders: &HashSet<ValueId>,
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
                && !sources
                    .iter()
                    .all(|source| live.contains(source) || persistent_lenders.contains(source))
            {
                invalid.insert(*borrowed);
            }
        }
    }
    authorities.retain(|borrowed, _| !invalid.contains(borrowed));
}

/// A managed value read from a capture is held by the active environment, which outlives the activation's
/// suspensions (the frame keeps it while a derived value is live), so the read borrows without a lender.
fn borrow_capture_reads(
    binding: &crate::control::ast::Binding,
    live: &HashSet<ValueId>,
    environment: &EnvironmentAliasPlan,
    authorities: &mut HashMap<ValueId, HashSet<ValueId>>,
) {
    let Operation::Atom(atom) = &binding.operation else {
        return;
    };
    if !matches!(
        atom.kind,
        crate::closure::ast::AtomKind::Reference(crate::closure::ast::Reference::Capture(_))
    ) {
        return;
    }
    let mut leaves = Vec::new();
    collect_pattern_binding_order(&binding.pattern, &mut leaves);
    for leaf in leaves {
        if live.contains(&leaf) && environment.is_root(leaf) {
            authorities.insert(leaf, HashSet::new());
        }
    }
}

/// A tail call hands its operands to code that runs after the activation's environment is released, so a value
/// tied to that environment must be an owner there.
fn remove_environment_aliases_used_by_tail_calls(
    control: &Program,
    environment: &EnvironmentAliasPlan,
    authorities: &mut HashMap<ValueId, HashSet<ValueId>>,
) {
    for state in &control.states {
        if let Terminator::TailCall { callee, argument } = &state.terminator {
            for atom in [callee, argument] {
                if let Some(binding) = managed_binding_id(atom)
                    && environment.is_tied(binding)
                {
                    authorities.remove(&binding);
                }
            }
        }
    }
}
