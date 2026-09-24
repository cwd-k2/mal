use std::collections::{HashMap, HashSet};

use crate::anf::ast::ValueId;
use crate::control::ast::{Program, StateId, Terminator};

use super::super::ApplicationGraph;
use super::super::pass_through::ParameterPassThrough;

pub(super) fn plan(
    program: &Program,
    applications: &ApplicationGraph,
) -> HashMap<StateId, HashSet<ValueId>> {
    let mut result = HashMap::new();
    let pass_through = ParameterPassThrough::new(program);
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
        let Some(pattern) = pass_through.parameter_pattern(program, function.entry, parameter)
        else {
            continue;
        };
        let mut common = None::<HashSet<ValueId>>;
        for (site, _) in &recursive_sites {
            let argument = match &program.states[site.0].terminator {
                Terminator::Call { argument, .. } | Terminator::TailCall { argument, .. } => {
                    argument
                }
                _ => {
                    common = Some(HashSet::new());
                    break;
                }
            };
            let preserved = pass_through.fields(pattern, argument);
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
