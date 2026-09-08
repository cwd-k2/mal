use std::collections::{HashMap, HashSet};

use crate::closure::ast::{self as closure, AtomKind, FunctionId, Reference};
use crate::control::ast::{self as control, StateId, Terminator};

use super::{ApplicationGraph, ControlRegionId, ControlRegionPlan, TailCallPlan};

mod graph;

use graph::{direct_graph, is_acyclic};

pub(super) use super::application_graph::reachable_states;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::c_emit::body) enum ControlCallMode {
    Direct(FunctionId),
    DirectSelfTail,
    Dispatch,
}

pub(in crate::c_emit::body) struct ControlCallPlan {
    modes: HashMap<StateId, ControlCallMode>,
    dispatch_targets: HashMap<StateId, Vec<FunctionId>>,
    dispatch_bindings: HashSet<crate::anf::ast::ValueId>,
}

impl ControlCallPlan {
    pub(in crate::c_emit::body) fn new(
        control: &control::Program,
        applications: &ApplicationGraph,
        tail_calls: &TailCallPlan,
        regions: &ControlRegionPlan,
    ) -> Self {
        let mut modes = HashMap::new();
        let mut dispatch_targets = HashMap::new();

        for binding in &control.bindings {
            for site in reachable_states(control, binding.entry) {
                let mode = applications
                    .direct_target(site)
                    .map(ControlCallMode::Direct)
                    .unwrap_or(ControlCallMode::Dispatch);
                modes.insert(site, mode);
                if mode == ControlCallMode::Dispatch {
                    dispatch_targets.insert(
                        site,
                        applications.targets(site).unwrap_or_default().to_vec(),
                    );
                }
            }
        }

        for function in &control.functions {
            for site in reachable_states(control, function.entry) {
                let state = &control.states[site.0];
                let terminator = &state.terminator;
                if application_callee(terminator).is_none() {
                    continue;
                }
                if tail_calls.is_fused(site) {
                    modes.insert(site, ControlCallMode::DirectSelfTail);
                } else if regions.site_region(site).is_some() {
                    modes.insert(site, ControlCallMode::Dispatch);
                    dispatch_targets.insert(
                        site,
                        applications.targets(site).unwrap_or_default().to_vec(),
                    );
                } else if let Some(callee) = applications.direct_target(site) {
                    modes.insert(site, ControlCallMode::Direct(callee));
                } else {
                    modes.insert(site, ControlCallMode::Dispatch);
                    dispatch_targets.insert(
                        site,
                        applications.targets(site).unwrap_or_default().to_vec(),
                    );
                }
            }
        }

        let direct_graph = direct_graph(control, &modes);
        debug_assert!(is_acyclic(
            &direct_graph,
            control.functions.iter().map(|function| function.id)
        ));
        let dispatch_bindings = control
            .states
            .iter()
            .enumerate()
            .filter(|(index, _)| modes.get(&StateId(*index)) == Some(&ControlCallMode::Dispatch))
            .filter_map(|(_, state)| application_callee(&state.terminator))
            .filter_map(|callee| match callee.kind {
                AtomKind::Reference(Reference::Binding(id)) => Some(id),
                _ => None,
            })
            .collect();
        Self {
            modes,
            dispatch_targets,
            dispatch_bindings,
        }
    }

    pub(in crate::c_emit::body) fn mode(&self, site: StateId) -> Option<ControlCallMode> {
        self.modes.get(&site).copied()
    }

    pub(in crate::c_emit::body) fn has_direct_target(&self, function: FunctionId) -> bool {
        self.modes
            .values()
            .any(|mode| *mode == ControlCallMode::Direct(function))
    }

    pub(in crate::c_emit::body) fn needs_closure_binding(
        &self,
        id: crate::anf::ast::ValueId,
    ) -> bool {
        self.dispatch_bindings.contains(&id)
    }

    pub(in crate::c_emit::body) fn dispatch_targets(&self, site: StateId) -> Option<&[FunctionId]> {
        self.dispatch_targets.get(&site).map(Vec::as_slice)
    }

    pub(in crate::c_emit::body) fn requires_common_control(
        &self,
        program: &control::Program,
        regions: &ControlRegionPlan,
        region: ControlRegionId,
    ) -> bool {
        regions.functions(region).iter().any(|function| {
            let entry = program
                .functions
                .iter()
                .find(|candidate| candidate.id == *function)
                .expect("region function has a control entry")
                .entry;
            reachable_states(program, entry).into_iter().any(|site| {
                regions.site_region(site) == Some(region)
                    && self.mode(site) == Some(ControlCallMode::Dispatch)
                    && !is_direct_self_call(&program.states[site.0].terminator, *function)
            })
        })
    }
}

fn application_callee(terminator: &Terminator) -> Option<&closure::Atom> {
    match terminator {
        Terminator::Call { callee, .. } | Terminator::TailCall { callee, .. } => Some(callee),
        _ => None,
    }
}

fn is_direct_self_call(terminator: &Terminator, caller: FunctionId) -> bool {
    matches!(
        terminator,
        Terminator::Call {
            callee:
                closure::Atom {
                    kind: AtomKind::Reference(Reference::SelfClosure(target)),
                    ..
                },
            ..
        } if *target == caller
    )
}

#[cfg(test)]
mod tests {
    use super::super::super::direct_function_id;
    use super::super::{ClosureUsePlan, ContinuationGraph};
    use super::*;
    use crate::source::{FileId, SourceFile};
    use crate::{anf, check, closure, control, core, parser, resolve};

    #[test]
    fn preserves_direct_edges_without_admitting_recursive_c_call_cycles() {
        let source = SourceFile::new(
            FileId::new(69),
            "control-call-plan.mal",
            "helper :: Int32 -> Int32 := \\(x :: Int32) { x + 1i32; };\n\
             recursive :: Int32 -> Int32 := \\(n :: Int32) {\n\
               if (n == 0i32)\n\
                 then { helper(n) }\n\
                 else {\n\
                   child := recursive(n - 1i32);\n\
                   helper(child);\n\
                 };\n\
             };\n\
             tail :: Int32 -> Int32 := \\(n :: Int32) {\n\
               if (n == 0i32) then { n } else { tail(n - 1i32) };\n\
             };"
            .into(),
        );
        let parsed = parser::parse(&source).expect("parse plan fixture");
        let resolved = resolve::resolve(&parsed).expect("resolve plan fixture");
        let checked = check::check(&resolved).expect("check plan fixture");
        let core = core::lower(&checked);
        let anf = anf::lower(&core);
        let closure = closure::convert(&anf);
        let control = control::lower(&closure);
        let uses = ClosureUsePlan::new(&closure);
        let applications = ApplicationGraph::new(&closure, &control, &uses);
        let tail_calls = TailCallPlan::new(&closure, &control, &applications);
        let continuations = ContinuationGraph::new(&control, &applications, &tail_calls);
        let regions = ControlRegionPlan::new(&control, &continuations);
        let plan = ControlCallPlan::new(&control, &applications, &tail_calls, &regions);

        let helper = top_level_function_id(&closure, "helper");
        let recursive = control
            .functions
            .iter()
            .find(|function| function.id == top_level_function_id(&closure, "recursive"))
            .expect("recursive function");
        let recursive_modes = reachable_states(&control, recursive.entry)
            .into_iter()
            .filter_map(|site| plan.mode(site))
            .collect::<Vec<_>>();
        assert!(recursive_modes.contains(&ControlCallMode::Dispatch));
        assert!(recursive_modes.contains(&ControlCallMode::Direct(helper)));

        let tail = control
            .functions
            .iter()
            .find(|function| function.id == top_level_function_id(&closure, "tail"))
            .expect("tail-recursive function");
        assert!(
            reachable_states(&control, tail.entry)
                .into_iter()
                .filter_map(|site| plan.mode(site))
                .any(|mode| mode == ControlCallMode::DirectSelfTail)
        );
    }

    #[test]
    fn closes_known_edges_over_type_compatible_indirect_targets() {
        let source = SourceFile::new(
            FileId::new(70),
            "indirect-control-cycle.mal",
             "apply :: ((Int32 -> Int32), Int32) -> Int32 := \\(function :: Int32 -> Int32, value :: Int32) {\n\
               function(value);\n\
             };\n\
             identity :: Int32 -> Int32 := \\(value :: Int32) { value; };\n\
             main :: Unit -> Int32 := \\() {\n\
               recurse :: Int32 -> Int32 := \\(value :: Int32) {\n\
                 if (value == 0i32) then { 0i32 } else {\n\
                   child := apply(recurse, value - 1i32);\n\
                   child + 1i32;\n\
                 };\n\
               };\n\
               recurse(2i32) - 2i32;\n\
             };"
            .into(),
        );
        let parsed = parser::parse(&source).expect("parse indirect cycle fixture");
        let resolved = resolve::resolve(&parsed).expect("resolve indirect cycle fixture");
        let checked = check::check(&resolved).expect("check indirect cycle fixture");
        let core = core::lower(&checked);
        let anf = anf::lower(&core);
        let closure = closure::convert(&anf);
        let control = control::lower(&closure);
        let uses = ClosureUsePlan::new(&closure);
        let applications = ApplicationGraph::new(&closure, &control, &uses);
        let tail_calls = TailCallPlan::new(&closure, &control, &applications);
        let continuations = ContinuationGraph::new(&control, &applications, &tail_calls);
        let regions = ControlRegionPlan::new(&control, &continuations);
        let plan = ControlCallPlan::new(&control, &applications, &tail_calls, &regions);
        let apply = top_level_function_id(&closure, "apply");
        let identity = top_level_function_id(&closure, "identity");

        assert!(control.states.iter().enumerate().any(|(index, state)| {
            let (Terminator::TailCall { callee, .. } | Terminator::Call { callee, .. }) =
                &state.terminator
            else {
                return false;
            };
            direct_function_id(&uses, callee) == Some(apply)
                && plan.mode(StateId(index)) == Some(ControlCallMode::Dispatch)
                && regions.site_region(StateId(index)).is_some()
        }));
        assert!(control.states.iter().enumerate().any(|(index, state)| {
            let (Terminator::TailCall { callee, .. } | Terminator::Call { callee, .. }) =
                &state.terminator
            else {
                return false;
            };
            direct_function_id(&uses, callee).is_none()
                && regions.site_region(StateId(index)).is_some()
                && plan
                    .dispatch_targets(StateId(index))
                    .is_some_and(|targets| !targets.is_empty())
        }));
        assert!(control.states.iter().enumerate().all(|(index, _)| {
            regions
                .recursive_targets(StateId(index))
                .is_none_or(|targets| !targets.contains(&identity))
        }));
    }

    #[test]
    fn fuses_a_pure_indirect_tail_forwarder_back_into_self_recursion() {
        let source = SourceFile::new(
            FileId::new(71),
            "indirect-tail-forwarder.mal",
            "apply :: ((Int32 -> Int32), Int32) -> Int32 := \\(function :: Int32 -> Int32, value :: Int32) {\n\
               function(value);\n\
             };\n\
             main :: Unit -> Int32 := \\() {\n\
               recurse :: Int32 -> Int32 := \\(value :: Int32) {\n\
                 if (value == 0i32) then { 0i32 } else { apply(recurse, value - 1i32) };\n\
               };\n\
               recurse(2i32);\n\
             };"
            .into(),
        );
        let parsed = parser::parse(&source).expect("parse tail forwarder fixture");
        let resolved = resolve::resolve(&parsed).expect("resolve tail forwarder fixture");
        let checked = check::check(&resolved).expect("check tail forwarder fixture");
        let core = core::lower(&checked);
        let anf = anf::lower(&core);
        let closure = closure::convert(&anf);
        let control = control::lower(&closure);
        let uses = ClosureUsePlan::new(&closure);
        let applications = ApplicationGraph::new(&closure, &control, &uses);
        let tail_calls = TailCallPlan::new(&closure, &control, &applications);
        let continuations = ContinuationGraph::new(&control, &applications, &tail_calls);
        let regions = ControlRegionPlan::new(&control, &continuations);
        let plan = ControlCallPlan::new(&control, &applications, &tail_calls, &regions);
        let apply = top_level_function_id(&closure, "apply");

        assert!(regions.ids().next().is_none());

        assert!(control.states.iter().enumerate().any(|(index, state)| {
            let Terminator::TailCall { callee, .. } = &state.terminator else {
                return false;
            };
            let site = StateId(index);
            direct_function_id(&uses, callee) == Some(apply)
                && plan.mode(site) == Some(ControlCallMode::DirectSelfTail)
                && tail_calls.forwarded_self_argument(site).is_some()
        }));
        assert!(control.states.iter().enumerate().any(|(index, state)| {
            let (Terminator::TailCall { callee, .. } | Terminator::Call { callee, .. }) =
                &state.terminator
            else {
                return false;
            };
            direct_function_id(&uses, callee).is_none()
                && regions.site_region(StateId(index)).is_none()
        }));
    }

    fn top_level_function_id(program: &closure::ast::Program, name: &str) -> FunctionId {
        let binding = program
            .bindings
            .iter()
            .find(|binding| {
                matches!(
                    &binding.pattern,
                    closure::ast::TopLevelPattern::Binding { name: candidate, .. }
                        if candidate == name
                )
            })
            .expect("top-level binding");
        binding
            .value
            .bindings
            .iter()
            .find_map(|binding| match binding.operation {
                closure::ast::Operation::MakeClosure { function, .. } => Some(function),
                _ => None,
            })
            .expect("top-level function closure")
    }
}
