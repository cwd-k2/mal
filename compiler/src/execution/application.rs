use std::collections::{HashMap, HashSet};

use crate::closure::ast::{self as closure, FunctionId};
use crate::control::ast::{self as control, StateId, Terminator};

use super::{ClosureUsePlan, direct_function_id};

pub(crate) struct ApplicationGraph {
    sites: HashMap<StateId, ApplicationSite>,
}

struct ApplicationSite {
    caller: Option<FunctionId>,
    direct_target: Option<FunctionId>,
    targets: Vec<FunctionId>,
}

impl ApplicationGraph {
    pub(crate) fn new(
        closure: &closure::Program,
        control: &control::Program,
        closure_uses: &ClosureUsePlan,
    ) -> Self {
        let mut sites = HashMap::new();
        for binding in &control.bindings {
            collect_sites(
                closure,
                control,
                closure_uses,
                binding.entry,
                None,
                &mut sites,
            );
        }
        for function in &control.functions {
            collect_sites(
                closure,
                control,
                closure_uses,
                function.entry,
                Some(function.id),
                &mut sites,
            );
        }

        Self { sites }
    }

    pub(crate) fn direct_target(&self, site: StateId) -> Option<FunctionId> {
        self.sites.get(&site).and_then(|site| site.direct_target)
    }

    pub(crate) fn targets(&self, site: StateId) -> Option<&[FunctionId]> {
        self.sites.get(&site).map(|site| site.targets.as_slice())
    }

    pub(crate) fn sites_from(
        &self,
        function: FunctionId,
    ) -> impl Iterator<Item = (StateId, &[FunctionId])> + '_ {
        self.sites.iter().filter_map(move |(id, site)| {
            (site.caller == Some(function)).then_some((*id, site.targets.as_slice()))
        })
    }
}

fn collect_sites(
    closure: &closure::Program,
    control: &control::Program,
    closure_uses: &ClosureUsePlan,
    entry: StateId,
    caller: Option<FunctionId>,
    sites: &mut HashMap<StateId, ApplicationSite>,
) {
    for site in reachable_states(control, entry) {
        let Some(callee) = application_callee(&control.states[site.0].terminator) else {
            continue;
        };
        let direct_target = direct_function_id(closure_uses, callee);
        let targets = direct_target
            .map(|target| vec![target])
            .unwrap_or_else(|| compatible_targets(closure, callee));
        let previous = sites.insert(
            site,
            ApplicationSite {
                caller,
                direct_target,
                targets,
            },
        );
        debug_assert!(previous.is_none(), "control states have a unique owner");
    }
}

fn compatible_targets(program: &closure::Program, callee: &closure::Atom) -> Vec<FunctionId> {
    let crate::check::ast::Type::Function { parameter, result } = &callee.ty else {
        return Vec::new();
    };
    program
        .functions
        .iter()
        .filter(|target| target.parameter.ty == **parameter && target.body.result.ty == **result)
        .map(|target| target.id)
        .collect()
}

fn application_callee(terminator: &Terminator) -> Option<&closure::Atom> {
    match terminator {
        Terminator::Call { callee, .. } | Terminator::TailCall { callee, .. } => Some(callee),
        _ => None,
    }
}

pub(super) fn reachable_states(program: &control::Program, entry: StateId) -> Vec<StateId> {
    let mut pending = vec![entry];
    let mut seen = HashSet::new();
    let mut states = Vec::new();
    while let Some(id) = pending.pop() {
        if !seen.insert(id) {
            continue;
        }
        states.push(id);
        match &program.states[id.0].terminator {
            Terminator::Return(_) | Terminator::TailCall { .. } => {}
            Terminator::Goto(target) | Terminator::Jump { target, .. } => pending.push(*target),
            Terminator::Call { resume, .. } => pending.push(*resume),
            Terminator::Case { arms, .. } => {
                pending.extend(arms.iter().map(|arm| arm.target));
            }
            Terminator::PrimitiveBranch {
                otherwise, then, ..
            } => pending.extend([*otherwise, *then]),
        }
    }
    states
}
