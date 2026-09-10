use std::collections::{HashMap, HashSet};

use crate::closure::ast::{self as closure, AtomKind, FunctionId, Reference};
use crate::control::ast::{self as control, StateId, Terminator};

use super::ApplicationGraph;

mod forwarder;

use forwarder::forwarded_self_tail_argument;

pub(crate) struct TailCallPlan {
    fused_sites: HashSet<StateId>,
    forwarded_self_arguments: HashMap<StateId, closure::Atom>,
}

impl TailCallPlan {
    pub(crate) fn new(
        closure: &closure::Program,
        control: &control::Program,
        applications: &ApplicationGraph,
    ) -> Self {
        let mut fused_sites = HashSet::new();
        let mut forwarded_self_arguments = HashMap::new();
        for function in &control.functions {
            for (site, _) in applications.sites_from(function.id) {
                let state = &control.states[site.0];
                let terminator = &state.terminator;
                if let Some(argument) = applications.direct_target(site).and_then(|forwarder| {
                    forwarded_self_tail_argument(closure, state, terminator, function.id, forwarder)
                }) {
                    fused_sites.insert(site);
                    forwarded_self_arguments.insert(site, argument);
                } else if is_direct_self_tail(terminator, function.id) {
                    fused_sites.insert(site);
                }
            }
        }
        Self {
            fused_sites,
            forwarded_self_arguments,
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
    ) -> bool {
        let expected = Self::new(closure, control, applications);
        self.fused_sites == expected.fused_sites
            && self.forwarded_self_arguments == expected.forwarded_self_arguments
    }

    pub(super) fn forwarded_self_arguments(&self) -> &HashMap<StateId, closure::Atom> {
        &self.forwarded_self_arguments
    }
}

fn is_direct_self_tail(terminator: &Terminator, function: FunctionId) -> bool {
    matches!(
        terminator,
        Terminator::TailCall {
            callee:
                closure::Atom {
                    kind: AtomKind::Reference(Reference::SelfClosure(target)),
                    ..
                },
            ..
        } if *target == function
    )
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
            "walk :: Int32 -> Int32 := (value) { if (value == 0i32) then { 0i32 } else { walk(value - 1i32) }; }; main :: Unit -> Int32 := () { walk(1i32); };"
                .into(),
        );
        let parsed = parser::parse(&source).expect("parse tail plan fixture");
        let resolved = resolve::resolve(&parsed).expect("resolve tail plan fixture");
        let checked = check::check(&resolved).expect("check tail plan fixture");
        let core = core::lower(&checked);
        let anf = anf::lower(&core);
        let closure = crate::closure::convert(&anf);
        let control = crate::control::lower(&closure);
        let uses = ClosureUsePlan::new(&closure);
        let applications = ApplicationGraph::new(&closure, &control, &uses);
        let mut plan = TailCallPlan::new(&closure, &control, &applications);

        assert!(plan.is_valid(&closure, &control, &applications));
        let site = *plan.fused_sites.iter().next().expect("fused tail site");
        plan.fused_sites.remove(&site);
        assert!(!plan.is_valid(&closure, &control, &applications));
    }
}
