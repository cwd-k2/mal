use std::collections::{HashMap, HashSet};

use crate::closure::ast::{AtomId, Block, FunctionId, Operation, Pattern, Program};
use crate::control::ast::{StateId, Terminator};
use crate::flow::{ClosureFlow, CompatibleTargets};

use super::plan::{ApplicationStep, CallSite, Creator, ProducerResult, ProducerStep, Scope};

/// Enumerates program identities before a rewrite decides whether every relevant instance is closed.
pub(super) fn collect(
    program: &Program,
    producers: &[ProducerStep],
    applications: &[ApplicationStep],
) -> (Vec<Creator>, Vec<CallSite>) {
    let functions = slice_functions(producers, applications);
    let creators = creators(program, &functions);
    let call_sites = call_sites(program, &functions);
    (creators, call_sites)
}

fn slice_functions(
    producers: &[ProducerStep],
    applications: &[ApplicationStep],
) -> HashSet<FunctionId> {
    let mut functions = HashSet::new();
    for step in producers {
        functions.insert(step.function);
        functions.insert(match step.result {
            ProducerResult::Call(target) | ProducerResult::Closure(target) => target,
        });
    }
    for application in applications {
        functions.insert(application.host);
        functions.extend(application.targets.iter().copied());
    }
    functions
}

fn creators(program: &Program, relevant: &HashSet<FunctionId>) -> Vec<Creator> {
    let mut creators = Vec::new();
    for (index, binding) in program.bindings.iter().enumerate() {
        collect_creators(
            Scope::TopLevel(index),
            &binding.value,
            relevant,
            &mut creators,
        );
    }
    for function in &program.functions {
        collect_creators(
            Scope::Function(function.id),
            &function.body,
            relevant,
            &mut creators,
        );
        for join in &function.joins {
            collect_creators(
                Scope::Function(function.id),
                &join.body,
                relevant,
                &mut creators,
            );
        }
    }
    creators
}

fn collect_creators(
    scope: Scope,
    block: &Block,
    relevant: &HashSet<FunctionId>,
    creators: &mut Vec<Creator>,
) {
    for binding in &block.bindings {
        if let (Pattern::Binding { id, .. }, Operation::MakeClosure { function, captures }) =
            (&binding.pattern, &binding.operation)
            && relevant.contains(function)
        {
            creators.push(Creator {
                scope,
                binding: *id,
                function: *function,
                captures: captures.iter().map(|capture| capture.id).collect(),
            });
        }
        binding.operation.for_each_nested_block(|nested| {
            collect_creators(scope, nested, relevant, creators);
        });
    }
}

fn call_sites(program: &Program, relevant: &HashSet<FunctionId>) -> Vec<CallSite> {
    let control = crate::control::lower(program);
    let mut compatible = CompatibleTargets::new(program);
    let flow = ClosureFlow::new(program, &control, &mut compatible);
    let scopes = call_scopes(program);
    control
        .states
        .iter()
        .enumerate()
        .filter_map(|(index, state)| {
            let (Terminator::Call { callee, .. } | Terminator::TailCall { callee, .. }) =
                &state.terminator
            else {
                return None;
            };
            let reaching = flow.callee(StateId(index))?;
            if reaching.is_disjoint(relevant) {
                return None;
            }
            let mut targets = reaching.iter().copied().collect::<Vec<_>>();
            targets.sort_by_key(|function| function_number(*function));
            Some(CallSite {
                scope: scopes[&callee.id],
                site: callee.id,
                targets,
            })
        })
        .collect()
}

fn call_scopes(program: &Program) -> HashMap<AtomId, Scope> {
    let mut scopes = HashMap::new();
    for (index, binding) in program.bindings.iter().enumerate() {
        collect_call_scopes(Scope::TopLevel(index), &binding.value, &mut scopes);
    }
    for function in &program.functions {
        collect_call_scopes(Scope::Function(function.id), &function.body, &mut scopes);
        for join in &function.joins {
            collect_call_scopes(Scope::Function(function.id), &join.body, &mut scopes);
        }
    }
    scopes
}

fn collect_call_scopes(scope: Scope, block: &Block, scopes: &mut HashMap<AtomId, Scope>) {
    for binding in &block.bindings {
        if let Operation::Call { callee, .. } = &binding.operation {
            scopes.insert(callee.id, scope);
        }
        binding
            .operation
            .for_each_nested_block(|nested| collect_call_scopes(scope, nested, scopes));
    }
}

fn function_number(function: FunctionId) -> u32 {
    let FunctionId::Lambda(mal_frontend::resolve::ast::LambdaId(number)) = function;
    number
}
