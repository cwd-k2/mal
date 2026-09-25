use std::collections::HashMap;

use crate::closure::ast::{self as closure, FunctionId};
use crate::control::ast::{self as control, StateId, Terminator};

use super::{ClosureUsePlan, direct_function_id};
use crate::flow::{ClosureFlow, CompatibleTargets};

pub(super) use crate::control::reachable_states;

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
        let mut compatible = CompatibleTargets::new(closure);
        let flow = ClosureFlow::new(closure, control, &mut compatible);
        let mut targets = TargetIndex { compatible, flow };
        for binding in &control.bindings {
            collect_sites(
                control,
                closure_uses,
                binding.entry,
                None,
                &mut targets,
                &mut sites,
            );
        }
        for function in &control.functions {
            collect_sites(
                control,
                closure_uses,
                function.entry,
                Some(function.id),
                &mut targets,
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
                let mal_frontend::check::ast::Type::Function { parameter, result } = &callee.ty
                else {
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
    targets: &mut TargetIndex,
    sites: &mut HashMap<StateId, ApplicationSite>,
) {
    for site in reachable_states(control, entry) {
        let Some(callee) = application_callee(&control.states[site.0].terminator) else {
            continue;
        };
        let direct_target = direct_function_id(closure_uses, callee);
        let targets = direct_target
            .map(|target| vec![target])
            .unwrap_or_else(|| targets.for_site(site, callee));
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

/// The targets of an indirect application: the functions of the callee's type that the closure flow reaches it with,
/// or every function of that type when the flow reaches none.
struct TargetIndex {
    compatible: CompatibleTargets,
    flow: ClosureFlow,
}

impl TargetIndex {
    fn for_site(&mut self, site: StateId, callee: &closure::Atom) -> Vec<FunctionId> {
        let compatible = self.compatible.for_callee(callee);
        let Some(reached) = self.flow.callee(site) else {
            return compatible;
        };
        let narrowed = compatible
            .iter()
            .copied()
            .filter(|target| reached.contains(target))
            .collect::<Vec<_>>();
        if narrowed.is_empty() {
            compatible
        } else {
            narrowed
        }
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

fn pattern_type(pattern: &closure::Pattern) -> &mal_frontend::check::ast::Type {
    match pattern {
        closure::Pattern::Binding { ty, .. }
        | closure::Pattern::Wildcard { ty, .. }
        | closure::Pattern::Product { ty, .. } => ty,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{anf, core};
    use mal_frontend::{check, resolve};
    use mal_syntax::parser;
    use mal_syntax::source::{FileId, SourceFile};

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
            "Left :: (Int32, Unit); Right :: (Int32, Unit); left :: Left -> Left := (value) -> { value; }; right :: Right -> Right := (value) -> { value; }; apply :: ((Left -> Left), Right) -> Right := (function, value) -> { function(value); }; main :: Unit -> Int32 := () -> { (first, _) := apply(right, (0i32, ())); (second, _) := apply(left, (0i32, ())); first + second; };"
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
