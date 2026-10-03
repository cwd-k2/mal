use std::collections::{HashMap, HashSet};

use crate::closure::ast::{AtomId, Block, FunctionId, Operation, Program};
use crate::control::ast::{StateId, Terminator};
use crate::flow::{ClosureFlow, CompatibleTargets};

use super::plan::{ApplicationStep, ProducerResult, ProducerStep};

pub(super) fn trace(program: &Program, producers: &[ProducerStep]) -> Vec<ApplicationStep> {
    let control = crate::control::lower(program);
    let mut compatible = CompatibleTargets::new(program);
    let flow = ClosureFlow::new(program, &control, &mut compatible);
    let sites = control
        .states
        .iter()
        .enumerate()
        .filter_map(|(index, state)| {
            let (Terminator::Call { callee, .. } | Terminator::TailCall { callee, .. }) =
                &state.terminator
            else {
                return None;
            };
            Some((callee.id, StateId(index)))
        })
        .collect::<HashMap<_, _>>();
    let functions = program
        .functions
        .iter()
        .map(|function| (function.id, function))
        .collect::<HashMap<_, _>>();
    let mut pending = producers
        .iter()
        .filter_map(|step| match &step.result {
            ProducerResult::Closure { function, .. } => Some(*function),
            ProducerResult::Call { .. } => None,
        })
        .collect::<Vec<_>>();
    let mut visited = HashSet::new();
    let mut applications = Vec::new();
    while let Some(host) = pending.pop() {
        if !visited.insert(host) {
            continue;
        }
        let Some(function) = functions.get(&host) else {
            continue;
        };
        trace_block(
            host,
            &function.body,
            &sites,
            &flow,
            &mut pending,
            &mut applications,
        );
        for join in &function.joins {
            trace_block(
                host,
                &join.body,
                &sites,
                &flow,
                &mut pending,
                &mut applications,
            );
        }
    }
    applications
}

fn trace_block(
    host: FunctionId,
    block: &Block,
    sites: &HashMap<AtomId, StateId>,
    flow: &ClosureFlow,
    pending: &mut Vec<FunctionId>,
    applications: &mut Vec<ApplicationStep>,
) {
    for_each_call(block, &mut |site, argument| {
        let Some(state) = sites.get(&site).copied() else {
            return;
        };
        let Some(reaching) = flow.callee(state) else {
            return;
        };
        let mut targets = reaching.iter().copied().collect::<Vec<_>>();
        targets.sort_by_key(|function| function_number(*function));
        pending.extend(targets.iter().copied());
        applications.push(ApplicationStep {
            host,
            site,
            argument: argument.clone(),
            targets,
        });
    });
}

fn function_number(function: FunctionId) -> u32 {
    let FunctionId::Lambda(mal_frontend::resolve::ast::LambdaId(number)) = function;
    number
}

fn for_each_call(block: &Block, visit: &mut impl FnMut(AtomId, &crate::closure::ast::Atom)) {
    for binding in &block.bindings {
        if let Operation::Call { callee, argument } = &binding.operation {
            visit(callee.id, argument);
        }
        binding
            .operation
            .for_each_nested_block(|nested| for_each_call(nested, visit));
    }
}
