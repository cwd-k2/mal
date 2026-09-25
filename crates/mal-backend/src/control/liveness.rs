use std::collections::{HashMap, HashSet};

use crate::anf::ast::ValueId;
use crate::closure::ast::{self as closure, Atom, AtomKind, Pattern, Reference};

use super::Lowerer;
use super::ast::{LiveValue, State};

pub(crate) fn binding_use_counts(program: &super::ast::Program) -> HashMap<ValueId, usize> {
    let mut uses = HashMap::new();
    for state in &program.states {
        for binding in &state.bindings {
            binding
                .operation
                .for_each_atom(|atom| count_binding_use(atom, &mut uses));
        }
        state
            .terminator
            .for_each_atom(|atom| count_binding_use(atom, &mut uses));
    }
    uses
}

fn count_binding_use(atom: &Atom, uses: &mut HashMap<ValueId, usize>) {
    if let AtomKind::Reference(Reference::Binding(id)) = atom.kind {
        *uses.entry(id).or_default() += 1;
    }
}

impl Lowerer {
    pub(super) fn resolve_liveness(&mut self, start: usize, locals: &[LiveValue]) {
        let end = self.states.len();
        let local_values = locals
            .iter()
            .enumerate()
            .map(|(rank, value)| (value.id, (rank, value)))
            .collect::<std::collections::HashMap<_, _>>();
        let mut live_in = vec![HashSet::new(); end - start];
        let mut environment_in = vec![false; end - start];
        for index in start..end {
            let state = &self.states[index];
            let (mut next, definitions, uses_environment) = state_facts(state);
            // Top-level and predefined bindings are never live values; carrying them would make every state hold
            // every later reference to them.
            next.retain(|id| local_values.contains_key(id));
            let mut next_environment = uses_environment;
            for successor in state.terminator.successors() {
                debug_assert!(
                    (start..index).contains(&successor.0),
                    "control successors are emitted before their predecessors"
                );
                next.extend(
                    live_in[successor.0 - start]
                        .iter()
                        .filter(|id| !definitions.contains(id))
                        .copied(),
                );
                next_environment |= environment_in[successor.0 - start];
            }
            live_in[index - start] = next;
            environment_in[index - start] = next_environment;
        }

        for index in start..end {
            let state_live = &live_in[index - start];
            let mut live = state_live
                .iter()
                .map(|id| local_values[id])
                .collect::<Vec<_>>();
            live.sort_unstable_by_key(|(rank, _)| *rank);
            self.states[index].live = live.into_iter().map(|(_, value)| value.clone()).collect();
            self.states[index].needs_environment = environment_in[index - start];
        }
    }
}

pub(super) fn local_values(
    block: &closure::Block,
    parameter: Option<&closure::Parameter>,
    joins: &[closure::Join],
) -> Vec<LiveValue> {
    let mut values = Vec::new();
    if let Some(parameter) = parameter
        && let Some(id) = parameter.binding
    {
        values.push(LiveValue {
            id,
            ty: parameter.ty.clone(),
            span: parameter.span,
        });
    }
    for join in joins {
        collect_pattern_values(&join.parameter, join.span, &mut values);
        collect_local_values(&join.body, &mut values);
    }
    collect_local_values(block, &mut values);
    values
}

fn collect_local_values(block: &closure::Block, values: &mut Vec<LiveValue>) {
    for binding in &block.bindings {
        collect_pattern_values(&binding.pattern, binding.span, values);
        match &binding.operation {
            closure::Operation::Case { arms, .. } => {
                for arm in arms {
                    collect_pattern_values(&arm.pattern, arm.span, values);
                    collect_local_values(&arm.value, values);
                }
            }
            closure::Operation::PrimitiveBranch {
                otherwise, then, ..
            } => {
                collect_local_values(otherwise, values);
                collect_local_values(then, values);
            }
            _ => {}
        }
    }
}

fn collect_pattern_values(
    pattern: &Pattern,
    span: mal_syntax::source::Span,
    values: &mut Vec<LiveValue>,
) {
    match pattern {
        Pattern::Binding { id, ty } => values.push(LiveValue {
            id: *id,
            ty: ty.clone(),
            span,
        }),
        Pattern::Wildcard { .. } => {}
        Pattern::Product { elements, .. } => {
            for element in elements {
                collect_pattern_values(element, span, values);
            }
        }
    }
}

fn state_facts(state: &State) -> (HashSet<ValueId>, HashSet<ValueId>, bool) {
    let mut uses = HashSet::new();
    let mut definitions = HashSet::new();
    let mut environment = false;
    if let Some(input) = &state.input {
        collect_pattern_definitions(input, &mut definitions);
    }
    for binding in &state.bindings {
        binding.operation.for_each_atom(|atom| {
            collect_atom_uses(atom, &definitions, &mut uses, &mut environment)
        });
        collect_pattern_definitions(&binding.pattern, &mut definitions);
    }
    state
        .terminator
        .for_each_atom(|atom| collect_atom_uses(atom, &definitions, &mut uses, &mut environment));
    (uses, definitions, environment)
}

fn collect_pattern_definitions(pattern: &Pattern, definitions: &mut HashSet<ValueId>) {
    match pattern {
        Pattern::Binding { id, .. } => {
            definitions.insert(*id);
        }
        Pattern::Wildcard { .. } => {}
        Pattern::Product { elements, .. } => {
            for element in elements {
                collect_pattern_definitions(element, definitions);
            }
        }
    }
}

fn collect_atom_uses(
    atom: &Atom,
    definitions: &HashSet<ValueId>,
    uses: &mut HashSet<ValueId>,
    environment: &mut bool,
) {
    match atom.kind {
        AtomKind::Reference(Reference::Binding(id)) if !definitions.contains(&id) => {
            uses.insert(id);
        }
        AtomKind::Reference(Reference::Capture(_) | Reference::SelfClosure(_)) => {
            *environment = true;
        }
        _ => {}
    }
}
