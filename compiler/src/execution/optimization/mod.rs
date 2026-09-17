use std::collections::{HashMap, HashSet};

use crate::closure::ast::{self as closure, FunctionId};
use crate::control::ast::{self as control, StateId};

use super::ApplicationGraph;

mod direct_call;
mod self_tail;
mod tail_forwarder;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Technique {
    DirectCall,
    SelfTail,
    TailForwarder,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct OptimizationSet(u8);

impl OptimizationSet {
    pub(crate) const fn none() -> Self {
        Self(0)
    }

    pub(crate) const fn production() -> Self {
        Self::none()
            .with(Technique::DirectCall)
            .with(Technique::SelfTail)
            .with(Technique::TailForwarder)
    }

    pub(crate) const fn with(self, technique: Technique) -> Self {
        Self(self.0 | (1 << technique as u8))
    }

    const fn contains(self, technique: Technique) -> bool {
        self.0 & (1 << technique as u8) != 0
    }
}

pub(crate) struct OptimizationPlan {
    fused_sites: HashSet<StateId>,
    forwarded_self_arguments: HashMap<StateId, closure::Atom>,
    direct_targets: HashMap<StateId, FunctionId>,
}

impl OptimizationPlan {
    pub(crate) fn new(
        closure: &closure::Program,
        control: &control::Program,
        applications: &ApplicationGraph,
        enabled: OptimizationSet,
    ) -> Self {
        let forwarded_self_arguments = if enabled.contains(Technique::TailForwarder) {
            tail_forwarder::plan(closure, control, applications)
        } else {
            HashMap::new()
        };
        let mut fused_sites = forwarded_self_arguments
            .keys()
            .copied()
            .collect::<HashSet<_>>();
        if enabled.contains(Technique::SelfTail) {
            fused_sites.extend(self_tail::plan(control, applications));
        }
        let direct_targets = if enabled.contains(Technique::DirectCall) {
            direct_call::plan(applications)
        } else {
            HashMap::new()
        };
        Self {
            fused_sites,
            forwarded_self_arguments,
            direct_targets,
        }
    }

    pub(crate) fn is_fused(&self, site: StateId) -> bool {
        self.fused_sites.contains(&site)
    }

    pub(crate) fn is_valid(
        &self,
        closure: &closure::Program,
        control: &control::Program,
        applications: &ApplicationGraph,
        enabled: OptimizationSet,
    ) -> bool {
        let expected = Self::new(closure, control, applications, enabled);
        self.fused_sites == expected.fused_sites
            && self.forwarded_self_arguments == expected.forwarded_self_arguments
            && self.direct_targets == expected.direct_targets
    }

    pub(super) fn forwarded_self_arguments(&self) -> &HashMap<StateId, closure::Atom> {
        &self.forwarded_self_arguments
    }

    pub(crate) fn direct_target(&self, site: StateId) -> Option<FunctionId> {
        self.direct_targets.get(&site).copied()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::execution::ClosureUsePlan;
    use crate::source::{FileId, SourceFile};
    use crate::{anf, check, core, parser, resolve};

    #[test]
    fn validates_the_exact_fused_tail_site_set() {
        let source = SourceFile::new(
            FileId::new(84),
            "tail-plan.mal",
            "walk :: Int32 -> Int32 := (value) -> { if (value == 0i32) then { 0i32 } else { walk(value - 1i32) }; }; main :: Unit -> Int32 := () -> { walk(1i32); };"
                .into(),
        );
        let parsed = parser::parse(&source).expect("parse tail plan fixture");
        let resolved = resolve::resolve(&parsed).expect("resolve tail plan fixture");
        let checked = check::check(&resolved).expect("check tail plan fixture");
        let core =
            core::lower(&check::admit_monomorphic(checked).expect("specialize checked program"));
        let anf = anf::lower(&core);
        let closure = crate::closure::convert(&anf);
        let control = crate::control::lower(&closure);
        let uses = ClosureUsePlan::new(&closure);
        let applications = ApplicationGraph::new(&closure, &control, &uses);
        let enabled = OptimizationSet::none().with(Technique::SelfTail);
        let mut plan = OptimizationPlan::new(&closure, &control, &applications, enabled);

        assert!(plan.is_valid(&closure, &control, &applications, enabled));
        let site = *plan.fused_sites.iter().next().expect("fused tail site");
        plan.fused_sites.remove(&site);
        assert!(!plan.is_valid(&closure, &control, &applications, enabled));
    }

    #[test]
    fn an_empty_set_makes_no_optional_execution_decisions() {
        let source = SourceFile::new(
            FileId::new(85),
            "baseline-plan.mal",
            "walk :: Int32 -> Int32 := (value) -> { if (value == 0i32) then { 0i32 } else { walk(value - 1i32) }; }; main :: Unit -> Int32 := () -> { walk(1i32); };"
                .into(),
        );
        let parsed = parser::parse(&source).expect("parse baseline fixture");
        let resolved = resolve::resolve(&parsed).expect("resolve baseline fixture");
        let checked = check::check(&resolved).expect("check baseline fixture");
        let core =
            core::lower(&check::admit_monomorphic(checked).expect("specialize checked program"));
        let anf = anf::lower(&core);
        let closure = crate::closure::convert(&anf);
        let control = crate::control::lower(&closure);
        let uses = ClosureUsePlan::new(&closure);
        let applications = ApplicationGraph::new(&closure, &control, &uses);
        let plan =
            OptimizationPlan::new(&closure, &control, &applications, OptimizationSet::none());

        assert!(plan.fused_sites.is_empty());
        assert!(plan.forwarded_self_arguments.is_empty());
        assert!(plan.direct_targets.is_empty());
        assert!(plan.is_valid(&closure, &control, &applications, OptimizationSet::none()));
    }

    #[test]
    fn composes_each_execution_technique_independently() {
        let source = SourceFile::new(
            FileId::new(89),
            "independent-execution-techniques.mal",
            "apply :: ((Int32 -> Int32), Int32) -> Int32 := (function, value) -> { function(value); };\n\
             direct :: Int32 -> Int32 := (value) -> { if (value == 0i32) then { 0i32 } else { direct(value - 1i32) }; };\n\
             forwarded :: Int32 -> Int32 := (value) -> { if (value == 0i32) then { 0i32 } else { apply(forwarded, value - 1i32) }; };\n\
             main :: Unit -> Int32 := () -> { direct(1i32) + forwarded(1i32); };"
                .into(),
        );
        let parsed = parser::parse(&source).expect("parse independent technique fixture");
        let resolved = resolve::resolve(&parsed).expect("resolve independent technique fixture");
        let checked = check::check(&resolved).expect("check independent technique fixture");
        let core =
            core::lower(&check::admit_monomorphic(checked).expect("specialize checked program"));
        let anf = anf::lower(&core);
        let closure = crate::closure::convert(&anf);
        let control = crate::control::lower(&closure);
        let uses = ClosureUsePlan::new(&closure);
        let applications = ApplicationGraph::new(&closure, &control, &uses);

        let direct = OptimizationPlan::new(
            &closure,
            &control,
            &applications,
            OptimizationSet::none().with(Technique::DirectCall),
        );
        assert!(!direct.direct_targets.is_empty());
        assert!(direct.fused_sites.is_empty());

        let self_tail = OptimizationPlan::new(
            &closure,
            &control,
            &applications,
            OptimizationSet::none().with(Technique::SelfTail),
        );
        assert!(self_tail.direct_targets.is_empty());
        assert!(!self_tail.fused_sites.is_empty());
        assert!(self_tail.forwarded_self_arguments.is_empty());

        let forwarder = OptimizationPlan::new(
            &closure,
            &control,
            &applications,
            OptimizationSet::none().with(Technique::TailForwarder),
        );
        assert!(forwarder.direct_targets.is_empty());
        assert!(!forwarder.forwarded_self_arguments.is_empty());
        assert_eq!(
            forwarder.fused_sites,
            forwarder.forwarded_self_arguments.keys().copied().collect()
        );
    }

    #[test]
    fn direct_call_selects_the_only_type_compatible_target() {
        let source = SourceFile::new(
            FileId::new(90),
            "singleton-target.mal",
            "make :: Int32 -> (Int32 -> Int32) := (captured) -> { (value) -> { captured + value; }; };\n\
             apply :: ((Int32 -> Int32), Int32) -> Int32 := (function, value) -> { function(value); };\n\
             main :: Unit -> Int32 := () -> { apply(make(40i32), 2i32); };"
                .into(),
        );
        let parsed = parser::parse(&source).expect("parse singleton target fixture");
        let resolved = resolve::resolve(&parsed).expect("resolve singleton target fixture");
        let checked = check::check(&resolved).expect("check singleton target fixture");
        let core =
            core::lower(&check::admit_monomorphic(checked).expect("specialize checked program"));
        let anf = anf::lower(&core);
        let closure = crate::closure::convert(&anf);
        let control = crate::control::lower(&closure);
        let uses = ClosureUsePlan::new(&closure);
        let applications = ApplicationGraph::new(&closure, &control, &uses);
        let site = applications
            .sites()
            .map(|(site, _)| site)
            .find(|site| {
                applications.direct_target(*site).is_none()
                    && matches!(applications.targets(*site), Some([_]))
            })
            .expect("indirect site with one compatible target");

        let baseline =
            OptimizationPlan::new(&closure, &control, &applications, OptimizationSet::none());
        assert_eq!(baseline.direct_target(site), None);

        let direct = OptimizationPlan::new(
            &closure,
            &control,
            &applications,
            OptimizationSet::none().with(Technique::DirectCall),
        );
        assert_eq!(
            direct.direct_target(site),
            applications.targets(site).map(|targets| targets[0])
        );
    }
}
