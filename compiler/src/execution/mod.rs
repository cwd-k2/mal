use crate::closure::ast::{self as closure_ast, FunctionId};

mod application;
mod call;
mod closure;
mod continuation;
mod region;
mod tail;

pub(crate) use application::ApplicationGraph;
pub(crate) use call::{ControlCallMode, ControlCallPlan};
pub(crate) use closure::ClosureUsePlan;
pub(crate) use continuation::ContinuationGraph;
pub(crate) use region::{ControlRegionId, ControlRegionPlan};
pub(crate) use tail::TailCallPlan;

pub(crate) struct Program {
    pub(crate) control: crate::control::ast::Program,
    pub(crate) closure_uses: ClosureUsePlan,
    pub(crate) applications: ApplicationGraph,
    pub(crate) tail_calls: TailCallPlan,
    pub(crate) control_calls: ControlCallPlan,
    pub(crate) control_regions: ControlRegionPlan,
}

pub(crate) fn lower(program: &closure_ast::Program) -> Program {
    let closure_uses = ClosureUsePlan::new(program);
    debug_assert!(closure_uses.is_valid(program));
    let control = crate::control::lower(program);
    let applications = ApplicationGraph::new(program, &control, &closure_uses);
    let tail_calls = TailCallPlan::new(program, &control, &applications);
    let continuations = ContinuationGraph::new(&control, &applications, &tail_calls);
    let control_regions = ControlRegionPlan::new(&control, &continuations);
    debug_assert!(control_regions.is_valid(&control, &continuations));
    let control_calls =
        ControlCallPlan::new(&control, &applications, &tail_calls, &control_regions);
    debug_assert!(control.states.iter().enumerate().all(|(index, _)| {
        let site = crate::control::ast::StateId(index);
        control_calls.mode(site) != Some(ControlCallMode::Dispatch)
            || applications.targets(site).is_some()
    }));
    debug_assert!(control.states.iter().enumerate().all(|(index, state)| {
        !matches!(
            state.terminator,
            crate::control::ast::Terminator::Call { .. }
                | crate::control::ast::Terminator::TailCall { .. }
        ) || control_calls
            .mode(crate::control::ast::StateId(index))
            .is_some()
    }));
    Program {
        control,
        closure_uses,
        applications,
        tail_calls,
        control_calls,
        control_regions,
    }
}

pub(crate) fn direct_function_id(
    closure_uses: &ClosureUsePlan,
    callee: &closure_ast::Atom,
) -> Option<FunctionId> {
    match callee.kind {
        closure_ast::AtomKind::Reference(closure_ast::Reference::SelfClosure(function)) => {
            Some(function)
        }
        closure_ast::AtomKind::Reference(closure_ast::Reference::Binding(id)) => closure_uses
            .direct_closure(id)
            .map(|target| target.function),
        _ => None,
    }
}
