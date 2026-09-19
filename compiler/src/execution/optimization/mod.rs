use std::collections::{HashMap, HashSet};

use crate::closure::ast::{self as closure, AtomId, FunctionId};
use crate::control::ast::{self as control, StateId};

use super::ApplicationGraph;

mod direct_call;
mod self_tail;
mod tail_forwarder;
mod unique_capture;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Technique {
    DirectCall,
    SelfTail,
    TailForwarder,
    UniqueCapture,
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
            .with(Technique::UniqueCapture)
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
    unique_captures: HashSet<AtomId>,
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
        let unique_captures = if enabled.contains(Technique::UniqueCapture) {
            unique_capture::plan(closure, control, applications)
        } else {
            HashSet::new()
        };
        Self {
            fused_sites,
            forwarded_self_arguments,
            direct_targets,
            unique_captures,
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
            && self.unique_captures == expected.unique_captures
    }

    pub(super) fn forwarded_self_arguments(&self) -> &HashMap<StateId, closure::Atom> {
        &self.forwarded_self_arguments
    }

    pub(crate) fn direct_target(&self, site: StateId) -> Option<FunctionId> {
        self.direct_targets.get(&site).copied()
    }

    pub(crate) fn takes_unique_capture(&self, atom: AtomId) -> bool {
        self.unique_captures.contains(&atom)
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
        assert!(plan.unique_captures.is_empty());
        assert!(plan.is_valid(&closure, &control, &applications, OptimizationSet::none()));
    }

    #[test]
    fn selects_a_managed_capture_only_for_one_final_invocation() {
        let source = SourceFile::new(
            FileId::new(92),
            "unique-capture-plan.mal",
            "main :: Unit -> Int32 := () -> {
               source := pack<Int32>((buffer) -> { _ := buffer.new(1i32); (); });
               _ := pack<Unit>((_) -> {
                 changed := source.edit<Int32>((buffer) -> buffer.put(0usize, 2i32));
                 _ := changed # 0usize;
                 ();
               });
               0;
             };"
            .into(),
        );
        let checked = crate::pipeline::check(&source).expect("check unique capture fixture");
        let core = crate::core::lower(
            &crate::check::specialize(checked).expect("specialize unique capture fixture"),
        );
        let anf = crate::anf::lower(&core);
        let closure = crate::closure::convert(&anf);
        let control = crate::control::lower(&closure);
        let uses = ClosureUsePlan::new(&closure);
        let applications = ApplicationGraph::new(&closure, &control, &uses);
        let enabled = OptimizationSet::none().with(Technique::UniqueCapture);
        let plan = OptimizationPlan::new(&closure, &control, &applications, enabled);

        assert!(!plan.unique_captures.is_empty());
        assert!(plan.is_valid(&closure, &control, &applications, enabled));
        assert!(
            OptimizationPlan::new(&closure, &control, &applications, OptimizationSet::none())
                .unique_captures
                .is_empty()
        );
    }

    #[test]
    fn rejects_a_single_capture_site_repeated_by_a_recursive_caller() {
        let source = SourceFile::new(
            FileId::new(93),
            "repeated-capture-site.mal",
            "repeat :: ((Unit -> Unit), Int32) -> Unit := (callback, remaining) -> {
               if (remaining == 0i32)
               then { () }
               else { callback(); repeat(callback, remaining - 1i32) };
             };
             main :: Unit -> Int32 := () -> {
               value := \"capture\";
               callback :: Unit -> Unit := () -> { _ := #value; (); };
               repeat(callback, 2i32);
               0;
             };"
            .into(),
        );
        let checked = crate::pipeline::check(&source).expect("check repeated capture fixture");
        let core = crate::core::lower(
            &crate::check::specialize(checked).expect("specialize repeated capture fixture"),
        );
        let anf = crate::anf::lower(&core);
        let closure = crate::closure::convert(&anf);
        let control = crate::control::lower(&closure);
        let uses = ClosureUsePlan::new(&closure);
        let applications = ApplicationGraph::new(&closure, &control, &uses);
        let enabled = OptimizationSet::none().with(Technique::UniqueCapture);
        let plan = OptimizationPlan::new(&closure, &control, &applications, enabled);

        assert!(plan.unique_captures.is_empty());
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
