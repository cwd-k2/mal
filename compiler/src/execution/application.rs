use std::collections::{HashMap, HashSet};

use crate::closure::ast::{self as closure, FunctionId};
use crate::control::ast::{self as control, StateId, Terminator};

use super::{ClosureUsePlan, direct_function_id};

pub(crate) struct ApplicationGraph {
    sites: HashMap<StateId, ApplicationSite>,
}

#[derive(Eq, PartialEq)]
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

    pub(crate) fn caller(&self, site: StateId) -> Option<FunctionId> {
        self.sites.get(&site).and_then(|site| site.caller)
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

    pub(crate) fn sites(&self) -> impl Iterator<Item = (StateId, Option<FunctionId>)> + '_ {
        self.sites.iter().map(|(id, site)| (*id, site.caller))
    }

    pub(crate) fn is_valid(
        &self,
        closure: &closure::Program,
        control: &control::Program,
        closure_uses: &ClosureUsePlan,
    ) -> bool {
        self.sites == Self::new(closure, control, closure_uses).sites
            && self.sites.iter().all(|(site, site_plan)| {
                let terminator = &control.states[site.0].terminator;
                let Some((callee, argument, resume)) = application(terminator) else {
                    return false;
                };
                let crate::check::ast::Type::Function { parameter, result } = &callee.ty else {
                    return false;
                };
                argument.ty == **parameter
                    && match resume {
                        Some(resume) => control.states[resume.0]
                            .input
                            .as_ref()
                            .is_some_and(|input| pattern_type(input) == result.as_ref()),
                        None => site_plan.caller.is_some_and(|caller| {
                            closure.functions.iter().any(|function| {
                                function.id == caller && function.body.result.ty == **result
                            })
                        }),
                    }
                    && site_plan.targets.iter().all(|target| {
                        closure.functions.iter().any(|function| {
                            function.id == *target
                                && function.parameter.ty == **parameter
                                && function.body.result.ty == **result
                        })
                    })
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

fn application(
    terminator: &Terminator,
) -> Option<(&closure::Atom, &closure::Atom, Option<StateId>)> {
    match terminator {
        Terminator::Call {
            callee,
            argument,
            resume,
        } => Some((callee, argument, Some(*resume))),
        Terminator::TailCall { callee, argument } => Some((callee, argument, None)),
        _ => None,
    }
}

fn pattern_type(pattern: &closure::Pattern) -> &crate::check::ast::Type {
    match pattern {
        closure::Pattern::Binding { ty, .. }
        | closure::Pattern::Wildcard { ty, .. }
        | closure::Pattern::Product { ty, .. } => ty,
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::source::{FileId, SourceFile};
    use crate::{anf, check, core, parser, resolve};

    #[test]
    fn validates_exact_sites_and_type_compatible_targets() {
        let source = SourceFile::new(
            FileId::new(82),
            "application-plan.mal",
            "identity :: Int32 -> Int32 := (value) { value; }; apply :: ((Int32 -> Int32), Int32) -> Int32 := (function, value) { function(value); }; main :: Unit -> Int32 := () { apply(identity, 0i32); };"
                .into(),
        );
        let parsed = parser::parse(&source).expect("parse application plan fixture");
        let resolved = resolve::resolve(&parsed).expect("resolve application plan fixture");
        let checked = check::check(&resolved).expect("check application plan fixture");
        let core = core::lower(&checked);
        let anf = anf::lower(&core);
        let closure = crate::closure::convert(&anf);
        let control = crate::control::lower(&closure);
        let uses = ClosureUsePlan::new(&closure);
        let mut graph = ApplicationGraph::new(&closure, &control, &uses);

        assert!(graph.is_valid(&closure, &control, &uses));
        graph
            .sites
            .values_mut()
            .find(|site| site.direct_target.is_none())
            .expect("indirect application site")
            .targets
            .clear();
        assert!(!graph.is_valid(&closure, &control, &uses));
    }
}
