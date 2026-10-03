use std::collections::{HashMap, HashSet};

use crate::anf::ast::ValueId;
use crate::closure::ast::{Atom, Function, Operation, Pattern, Program};
use crate::core::ast::JoinId;

use super::index::Index;
use super::plan::{Demand, ProducerResult, ProducerStep};

pub(super) fn producer_steps(
    index: &Index,
    program: &Program,
    demands: &[Demand],
) -> Vec<ProducerStep> {
    let functions = program
        .functions
        .iter()
        .map(|function| (function.id, function))
        .collect::<HashMap<_, _>>();
    let mut pending = demands
        .iter()
        .map(|demand| demand.producer)
        .collect::<Vec<_>>();
    let mut visited = HashSet::new();
    let mut steps = Vec::new();
    while let Some(function) = pending.pop() {
        if !visited.insert(function) {
            continue;
        }
        let Some(definition) = functions.get(&function).copied() else {
            continue;
        };
        let Some(results) = producer_results(index, definition) else {
            continue;
        };
        for result in results {
            if let ProducerResult::Call { function, .. } = &result {
                pending.push(*function);
            }
            steps.push(ProducerStep { function, result });
        }
    }
    steps
}

fn producer_results(index: &Index, function: &Function) -> Option<Vec<ProducerResult>> {
    let mut results = trace_atom(
        index,
        &function.body.result,
        function,
        &HashMap::new(),
        &mut HashSet::new(),
    )?;
    results.dedup();
    (!results.is_empty()).then_some(results)
}

fn trace_atom(
    index: &Index,
    atom: &Atom,
    function: &Function,
    substitutions: &HashMap<ValueId, Atom>,
    active_joins: &mut HashSet<JoinId>,
) -> Option<Vec<ProducerResult>> {
    let binding = atom.binding()?;
    if let Some(value) = substitutions.get(&binding) {
        return trace_atom(index, value, function, substitutions, active_joins);
    }
    let operation = index.definitions.get(&index.origin(binding))?;
    match operation {
        Operation::Atom(value) => trace_atom(index, value, function, substitutions, active_joins),
        Operation::Call { callee, argument } => Some(vec![ProducerResult::Call {
            function: index.function(callee)?,
            site: callee.id,
            argument: argument.clone(),
        }]),
        Operation::MakeClosure {
            function: target,
            captures,
        } => Some(vec![ProducerResult::Closure {
            creator: index.origin(binding),
            function: *target,
            captures: captures.clone(),
        }]),
        Operation::Goto { target, value } => {
            trace_join(index, *target, value, function, substitutions, active_joins)
        }
        Operation::Case { arms, .. } => arms.iter().try_fold(Vec::new(), |mut all, arm| {
            all.extend(trace_atom(
                index,
                &arm.value.result,
                function,
                substitutions,
                active_joins,
            )?);
            Some(all)
        }),
        Operation::PrimitiveBranch {
            otherwise, then, ..
        } => {
            let mut results = trace_atom(
                index,
                &otherwise.result,
                function,
                substitutions,
                active_joins,
            )?;
            results.extend(trace_atom(
                index,
                &then.result,
                function,
                substitutions,
                active_joins,
            )?);
            Some(results)
        }
        _ => None,
    }
}

fn trace_join(
    index: &Index,
    target: JoinId,
    value: &Atom,
    function: &Function,
    substitutions: &HashMap<ValueId, Atom>,
    active_joins: &mut HashSet<JoinId>,
) -> Option<Vec<ProducerResult>> {
    if !active_joins.insert(target) {
        return None;
    }
    let join = function.joins.get(target.0)?;
    let mut substitutions = substitutions.clone();
    match &join.parameter {
        Pattern::Binding { id, .. } => {
            substitutions.insert(*id, value.clone());
        }
        Pattern::Wildcard { .. } => {}
        Pattern::Product { .. } => return None,
    }
    let result = trace_atom(
        index,
        &join.body.result,
        function,
        &substitutions,
        active_joins,
    );
    active_joins.remove(&target);
    result
}
