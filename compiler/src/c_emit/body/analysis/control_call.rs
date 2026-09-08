use std::collections::{HashMap, HashSet};

use crate::closure::ast::{self as closure, AtomKind, FunctionId, Reference};
use crate::control::ast::{self as control, StateId, Terminator};

use super::super::direct_function_id;
use super::ClosureUsePlan;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::c_emit::body) enum ControlCallMode {
    Direct(FunctionId),
    DirectSelfTail,
    Dispatch,
}

pub(in crate::c_emit::body) struct ControlCallPlan {
    modes: HashMap<StateId, ControlCallMode>,
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

        for binding in &control.bindings {
            for site in reachable_states(control, binding.entry) {
                let Some(callee) = application_callee(&control.states[site.0].terminator) else {
                    continue;
                };
                let mode = direct_function_id(closure_uses, callee)
                    .map(ControlCallMode::Direct)
                    .unwrap_or(ControlCallMode::Dispatch);
                modes.insert(site, mode);
            }
        }

        let mut candidates = Vec::new();
        for function in &control.functions {
            for site in reachable_states(control, function.entry) {
                let terminator = &control.states[site.0].terminator;
                let Some(callee) = application_callee(terminator) else {
                    continue;
                };
                if is_direct_self_tail(terminator, function.id) {
                    modes.insert(site, ControlCallMode::DirectSelfTail);
                } else if let Some(callee) = direct_function_id(closure_uses, callee) {
                    candidates.push(Candidate {
                        site,
                        caller: function.id,
                        callee,
                    });
                } else {
                    modes.insert(site, ControlCallMode::Dispatch);
                }
            }
        }

        let mut known_graph: HashMap<FunctionId, Vec<FunctionId>> = HashMap::new();
        for candidate in &candidates {
            known_graph
                .entry(candidate.caller)
                .or_default()
                .push(candidate.callee);
        }
        for candidate in candidates {
            if creates_cycle(&known_graph, candidate.caller, candidate.callee) {
                modes.insert(candidate.site, ControlCallMode::Dispatch);
            } else {
                modes.insert(candidate.site, ControlCallMode::Direct(candidate.callee));
            }
        }

        let direct_graph = direct_graph(control, &modes);
        debug_assert!(is_acyclic(
            &direct_graph,
            closure.functions.iter().map(|function| function.id)
        ));
        Self { modes }
    }

    pub(in crate::c_emit::body) fn mode(&self, site: StateId) -> Option<ControlCallMode> {
        self.modes.get(&site).copied()
    }

    pub(in crate::c_emit::body) fn requires_dispatch(&self) -> bool {
        self.modes
            .values()
            .any(|mode| *mode == ControlCallMode::Dispatch)
    }
}

fn direct_graph(
    program: &control::Program,
    modes: &HashMap<StateId, ControlCallMode>,
) -> HashMap<FunctionId, Vec<FunctionId>> {
    let mut graph: HashMap<FunctionId, Vec<FunctionId>> = HashMap::new();
    for function in &program.functions {
        for site in reachable_states(program, function.entry) {
            if let Some(ControlCallMode::Direct(callee)) = modes.get(&site) {
                graph.entry(function.id).or_default().push(*callee);
            }
        }
    }
    graph
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

fn creates_cycle(
    graph: &HashMap<FunctionId, Vec<FunctionId>>,
    caller: FunctionId,
    callee: FunctionId,
) -> bool {
    let mut pending = vec![callee];
    let mut seen = HashSet::new();
    while let Some(current) = pending.pop() {
        if current == caller {
            return true;
        }
        if seen.insert(current)
            && let Some(next) = graph.get(&current)
        {
            pending.extend(next);
        }
    }
    false
}

fn is_acyclic(
    graph: &HashMap<FunctionId, Vec<FunctionId>>,
    nodes: impl IntoIterator<Item = FunctionId>,
) -> bool {
    fn visit(
        graph: &HashMap<FunctionId, Vec<FunctionId>>,
        current: FunctionId,
        active: &mut HashSet<FunctionId>,
        finished: &mut HashSet<FunctionId>,
    ) -> bool {
        if finished.contains(&current) {
            return true;
        }
        if !active.insert(current) {
            return false;
        }
        if graph.get(&current).is_some_and(|next| {
            next.iter()
                .any(|target| !visit(graph, *target, active, finished))
        }) {
            return false;
        }
        active.remove(&current);
        finished.insert(current);
        true
    }

    let mut active = HashSet::new();
    let mut finished = HashSet::new();
    nodes
        .into_iter()
        .all(|node| visit(graph, node, &mut active, &mut finished))
}

fn reachable_states(program: &control::Program, entry: StateId) -> Vec<StateId> {
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
