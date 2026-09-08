use std::collections::{HashMap, HashSet};

use crate::closure::ast::{self as closure, AtomKind, FunctionId, Reference};
use crate::control::ast::{self as control, StateId, Terminator};

use super::super::direct_function_id;
use super::ClosureUsePlan;

mod forwarder;
mod graph;

use forwarder::forwarded_self_tail_argument;
use graph::{creates_cycle, direct_graph, is_acyclic};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::c_emit::body) enum ControlCallMode {
    Direct(FunctionId),
    DirectSelfTail,
    Dispatch,
}

pub(in crate::c_emit::body) struct ControlCallPlan {
    modes: HashMap<StateId, ControlCallMode>,
    dispatch_targets: HashMap<StateId, Vec<FunctionId>>,
    forwarded_self_arguments: HashMap<StateId, closure::Atom>,
    dispatch_bindings: HashSet<crate::anf::ast::ValueId>,
    recursive_targets: HashMap<StateId, Vec<FunctionId>>,
}

struct Candidate {
    site: StateId,
    caller: FunctionId,
    callee: FunctionId,
}

impl ControlCallPlan {
    pub(in crate::c_emit::body) fn new(
        closure: &closure::Program,
        control: &control::Program,
        closure_uses: &ClosureUsePlan,
    ) -> Self {
        let mut modes = HashMap::new();
        let mut dispatch_targets = HashMap::new();
        let mut forwarded_self_arguments = HashMap::new();

        for binding in &control.bindings {
            for site in reachable_states(control, binding.entry) {
                let Some(callee) = application_callee(&control.states[site.0].terminator) else {
                    continue;
                };
                let mode = direct_function_id(closure_uses, callee)
                    .map(ControlCallMode::Direct)
                    .unwrap_or(ControlCallMode::Dispatch);
                modes.insert(site, mode);
                if mode == ControlCallMode::Dispatch {
                    dispatch_targets.insert(site, compatible_targets(closure, callee));
                }
            }
        }

        let mut candidates = Vec::new();
        let mut callers = HashMap::new();
        let mut possible_graph: HashMap<FunctionId, Vec<FunctionId>> = HashMap::new();
        for function in &control.functions {
            for site in reachable_states(control, function.entry) {
                let state = &control.states[site.0];
                let terminator = &state.terminator;
                let Some(callee) = application_callee(terminator) else {
                    continue;
                };
                callers.insert(site, function.id);
                if let Some(argument) = forwarded_self_tail_argument(
                    closure,
                    state,
                    terminator,
                    function.id,
                    closure_uses,
                ) {
                    modes.insert(site, ControlCallMode::DirectSelfTail);
                    forwarded_self_arguments.insert(site, argument);
                } else if is_direct_self_tail(terminator, function.id) {
                    modes.insert(site, ControlCallMode::DirectSelfTail);
                } else if let Some(callee) = direct_function_id(closure_uses, callee) {
                    possible_graph.entry(function.id).or_default().push(callee);
                    candidates.push(Candidate {
                        site,
                        caller: function.id,
                        callee,
                    });
                } else {
                    modes.insert(site, ControlCallMode::Dispatch);
                    let targets = compatible_targets(closure, callee);
                    possible_graph
                        .entry(function.id)
                        .or_default()
                        .extend(targets.iter().copied());
                    dispatch_targets.insert(site, targets);
                }
            }
        }

        for candidate in candidates {
            if creates_cycle(&possible_graph, candidate.caller, candidate.callee) {
                modes.insert(candidate.site, ControlCallMode::Dispatch);
                dispatch_targets.insert(candidate.site, vec![candidate.callee]);
            } else {
                modes.insert(candidate.site, ControlCallMode::Direct(candidate.callee));
            }
        }
        let mut recursive_targets = HashMap::new();
        for (site, targets) in &dispatch_targets {
            let Some(caller) = callers.get(site).copied() else {
                continue;
            };
            let targets = targets
                .iter()
                .copied()
                .filter(|target| creates_cycle(&possible_graph, caller, *target))
                .collect::<Vec<_>>();
            if !targets.is_empty() {
                recursive_targets.insert(*site, targets);
            }
        }

        let direct_graph = direct_graph(control, &modes);
        debug_assert!(is_acyclic(
            &direct_graph,
            closure.functions.iter().map(|function| function.id)
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
            forwarded_self_arguments,
            dispatch_bindings,
            recursive_targets,
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

    pub(in crate::c_emit::body) fn is_recursive_dispatch(&self, site: StateId) -> bool {
        self.recursive_targets.contains_key(&site)
    }

    pub(in crate::c_emit::body) fn recursive_dispatch_targets(
        &self,
        site: StateId,
    ) -> Option<&[FunctionId]> {
        self.recursive_targets.get(&site).map(Vec::as_slice)
    }

    pub(in crate::c_emit::body) fn dispatch_targets(&self, site: StateId) -> Option<&[FunctionId]> {
        self.dispatch_targets.get(&site).map(Vec::as_slice)
    }

    pub(in crate::c_emit::body) fn forwarded_self_argument(
        &self,
        site: StateId,
    ) -> Option<&closure::Atom> {
        self.forwarded_self_arguments.get(&site)
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
    use crate::check::ast::{MemoryPrimitive, MemoryScalar};
    use crate::source::{FileId, SourceFile};
    use crate::{anf, check, closure, control, core, parser, resolve};

    const A: FunctionId = FunctionId::Memory(MemoryPrimitive::Load(MemoryScalar::Int8));
    const B: FunctionId = FunctionId::Memory(MemoryPrimitive::Load(MemoryScalar::Int16));
    const C: FunctionId = FunctionId::Memory(MemoryPrimitive::Load(MemoryScalar::Int32));

    #[test]
    fn rejects_an_edge_exactly_when_it_would_close_a_cycle() {
        let mut graph = HashMap::new();
        graph.insert(A, vec![B]);
        graph.insert(B, vec![C]);

        assert!(!creates_cycle(&graph, A, C));
        assert!(creates_cycle(&graph, C, A));
        assert!(creates_cycle(&graph, A, A));
    }

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
        let plan = ControlCallPlan::new(&closure, &control, &uses);

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
        let plan = ControlCallPlan::new(&closure, &control, &uses);
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
                && plan.is_recursive_dispatch(StateId(index))
        }));
        assert!(control.states.iter().enumerate().any(|(index, state)| {
            let (Terminator::TailCall { callee, .. } | Terminator::Call { callee, .. }) =
                &state.terminator
            else {
                return false;
            };
            direct_function_id(&uses, callee).is_none()
                && plan.is_recursive_dispatch(StateId(index))
                && plan
                    .dispatch_targets(StateId(index))
                    .is_some_and(|targets| !targets.is_empty())
        }));
        assert!(
            plan.recursive_targets
                .values()
                .all(|targets| !targets.contains(&identity))
        );
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
        let plan = ControlCallPlan::new(&closure, &control, &uses);
        let apply = top_level_function_id(&closure, "apply");

        assert!(control.states.iter().enumerate().any(|(index, state)| {
            let Terminator::TailCall { callee, .. } = &state.terminator else {
                return false;
            };
            let site = StateId(index);
            direct_function_id(&uses, callee) == Some(apply)
                && plan.mode(site) == Some(ControlCallMode::DirectSelfTail)
                && plan.forwarded_self_argument(site).is_some()
        }));
        assert!(control.states.iter().enumerate().any(|(index, state)| {
            let (Terminator::TailCall { callee, .. } | Terminator::Call { callee, .. }) =
                &state.terminator
            else {
                return false;
            };
            direct_function_id(&uses, callee).is_none()
                && !plan.is_recursive_dispatch(StateId(index))
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
