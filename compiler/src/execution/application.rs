use std::collections::{HashMap, HashSet};

use crate::check::ast::Type;
use crate::check::type_fingerprint::TypeFingerprints;
use crate::closure::ast::{self as closure, FunctionId};
use crate::control::ast::{self as control, StateId, Terminator};

use super::{ClosureUsePlan, direct_function_id};

pub(crate) struct ApplicationGraph {
    sites: HashMap<StateId, ApplicationSite>,
    callers: HashMap<FunctionId, Vec<StateId>>,
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
        let mut compatible_targets = CompatibleTargets::new(closure);
        for binding in &control.bindings {
            collect_sites(
                control,
                closure_uses,
                binding.entry,
                None,
                &mut compatible_targets,
                &mut sites,
            );
        }
        for function in &control.functions {
            collect_sites(
                control,
                closure_uses,
                function.entry,
                Some(function.id),
                &mut compatible_targets,
                &mut sites,
            );
        }

        let mut callers = HashMap::<FunctionId, Vec<StateId>>::new();
        for (site, application) in &sites {
            if let Some(caller) = application.caller {
                callers.entry(caller).or_default().push(*site);
            }
        }
        for sites in callers.values_mut() {
            sites.sort_unstable_by_key(|site| site.0);
        }

        Self { sites, callers }
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
        self.callers
            .get(&function)
            .into_iter()
            .flatten()
            .filter_map(|id| {
                self.sites
                    .get(id)
                    .map(|site| (*id, site.targets.as_slice()))
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
        let expected = Self::new(closure, control, closure_uses);
        let functions = closure
            .functions
            .iter()
            .map(|function| (function.id, function))
            .collect::<HashMap<_, _>>();
        self.sites == expected.sites
            && self.callers == expected.callers
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
                        None => site_plan
                            .caller
                            .and_then(|caller| functions.get(&caller))
                            .is_some_and(|function| function.body.result.ty == **result),
                    }
                    && site_plan.targets.iter().all(|target| {
                        functions.get(target).is_some_and(|function| {
                            function.parameter.ty == **parameter
                                && function.body.result.ty == **result
                        })
                    })
            })
    }
}

fn collect_sites(
    control: &control::Program,
    closure_uses: &ClosureUsePlan,
    entry: StateId,
    caller: Option<FunctionId>,
    compatible_targets: &mut CompatibleTargets,
    sites: &mut HashMap<StateId, ApplicationSite>,
) {
    for site in reachable_states(control, entry) {
        let Some(callee) = application_callee(&control.states[site.0].terminator) else {
            continue;
        };
        let direct_target = direct_function_id(closure_uses, callee);
        let targets = direct_target
            .map(|target| vec![target])
            .unwrap_or_else(|| compatible_targets.for_callee(callee));
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

struct CompatibleTargets {
    groups: HashMap<(u64, u64), Vec<TargetGroup>>,
    fingerprints: TypeFingerprints,
}

struct TargetGroup {
    parameter: Type,
    result: Type,
    targets: Vec<FunctionId>,
}

impl CompatibleTargets {
    fn new(program: &closure::Program) -> Self {
        let mut index = Self {
            groups: HashMap::new(),
            fingerprints: TypeFingerprints::default(),
        };
        for function in &program.functions {
            let parameter = &function.parameter.ty;
            let result = &function.body.result.ty;
            let fingerprint = index.fingerprints.signature(parameter, result);
            let groups = index.groups.entry(fingerprint).or_default();
            if let Some(group) = groups
                .iter_mut()
                .find(|group| group.parameter == *parameter && group.result == *result)
            {
                group.targets.push(function.id);
            } else {
                groups.push(TargetGroup {
                    parameter: parameter.clone(),
                    result: result.clone(),
                    targets: vec![function.id],
                });
            }
        }
        index
    }

    fn for_callee(&mut self, callee: &closure::Atom) -> Vec<FunctionId> {
        let Type::Function { parameter, result } = &callee.ty else {
            return Vec::new();
        };
        let fingerprint = self.fingerprints.signature(parameter, result);
        self.groups
            .get(&fingerprint)
            .and_then(|groups| {
                groups
                    .iter()
                    .find(|group| group.parameter == **parameter && group.result == **result)
            })
            .map_or_else(Vec::new, |group| group.targets.clone())
    }
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
            "identity :: Int32 -> Int32 := (value) -> { value; }; apply :: ((Int32 -> Int32), Int32) -> Int32 := (function, value) -> { function(value); }; main :: Unit -> Int32 := () -> { apply(identity, 0i32); };"
                .into(),
        );
        let parsed = parser::parse(&source).expect("parse application plan fixture");
        let resolved = resolve::resolve(&parsed).expect("resolve application plan fixture");
        let checked = check::check(&resolved).expect("check application plan fixture");
        let core =
            core::lower(&check::admit_monomorphic(checked).expect("specialize checked program"));
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

        graph = ApplicationGraph::new(&closure, &control, &uses);
        graph.callers.clear();
        assert!(!graph.is_valid(&closure, &control, &uses));
    }

    #[test]
    fn groups_structurally_equal_target_signatures() {
        let source = SourceFile::new(
            FileId::new(83),
            "application-structural-targets.mal",
            "Left :: (Int32, Unit); Right :: (Int32, Unit); left :: Left -> Left := (value) -> { value; }; right :: Right -> Right := (value) -> { value; }; apply :: ((Left -> Left), Right) -> Right := (function, value) -> { function(value); }; main :: Unit -> Int32 := () -> { (result, _) := apply(right, (0i32, ())); result; };"
                .into(),
        );
        let parsed = parser::parse(&source).expect("parse structural target fixture");
        let resolved = resolve::resolve(&parsed).expect("resolve structural target fixture");
        let checked = check::check(&resolved).expect("check structural target fixture");
        let core =
            core::lower(&check::admit_monomorphic(checked).expect("specialize checked program"));
        let anf = anf::lower(&core);
        let closure = crate::closure::convert(&anf);
        let control = crate::control::lower(&closure);
        let uses = ClosureUsePlan::new(&closure);

        let graph = ApplicationGraph::new(&closure, &control, &uses);

        assert!(
            graph
                .sites
                .values()
                .any(|site| site.direct_target.is_none() && site.targets.len() == 2)
        );
    }
}
