use crate::closure::ast::{self as closure_ast, FunctionId};

mod application;
mod call;
mod closure;
mod continuation;
mod frame;
mod optimization;
pub(crate) mod ownership;
mod parameter;
mod pass_through;
mod region;
mod self_tail_parameter;

pub(crate) use application::ApplicationGraph;
pub(crate) use call::{ControlCallMode, ControlCallPlan};
pub(crate) use closure::ClosureUsePlan;
pub(crate) use continuation::ContinuationGraph;
pub(crate) use frame::{ControlFrame, ControlFramePlan, FrameResume};
pub(crate) use optimization::{OptimizationPlan, OptimizationSet};
pub(crate) use ownership::{Inputs as OwnershipInputs, Plan as OwnershipPlan};
pub(crate) use parameter::{ParameterDestination, ParameterPlan};
pub(crate) use region::{ControlRegionId, ControlRegionPlan};
pub(crate) use self_tail_parameter::SelfTailParameterPlan;

pub(crate) struct Program {
    pub(crate) lowered: closure_ast::Program,
    pub(crate) control: crate::control::ast::Program,
    pub(crate) applications: ApplicationGraph,
    pub(crate) parameters: ParameterPlan,
    pub(crate) control_calls: ControlCallPlan,
    pub(crate) control_regions: ControlRegionPlan,
    pub(crate) control_frames: ControlFramePlan,
    pub(crate) ownership: OwnershipPlan,
    pub(crate) self_tail_parameters: SelfTailParameterPlan,
}

pub(crate) fn lower(lowered: closure_ast::Program, enabled: OptimizationSet) -> Program {
    let closure_uses = ClosureUsePlan::new(&lowered);
    debug_assert!(closure_uses.is_valid(&lowered));
    let control = crate::control::lower(&lowered);
    let parameters = ParameterPlan::new(&control);
    debug_assert!(parameters.is_valid(&control));
    let applications = ApplicationGraph::new(&lowered, &control, &closure_uses);
    debug_assert!(applications.is_valid(&lowered, &control, &closure_uses));
    let optimizations = OptimizationPlan::new(&lowered, &control, &applications, enabled);
    debug_assert!(optimizations.is_valid(&lowered, &control, &applications, enabled));
    let continuations = ContinuationGraph::new(&applications, &optimizations);
    let control_regions = ControlRegionPlan::new(&control, &continuations);
    debug_assert!(control_regions.is_valid(&control, &continuations));
    let control_calls =
        ControlCallPlan::new(&control, &applications, &optimizations, &control_regions);
    debug_assert!(control_calls.is_valid(
        &control,
        &applications,
        &optimizations,
        &control_regions
    ));
    let control_frames =
        ControlFramePlan::new(&control, &control_regions, &control_calls, &optimizations);
    debug_assert!(control_frames.is_valid(
        &control,
        &control_regions,
        &control_calls,
        &optimizations
    ));
    let ownership = OwnershipPlan::new(OwnershipInputs::new(
        &control,
        &applications,
        &optimizations,
        &parameters,
        &control_calls,
        &control_regions,
        &control_frames,
    ));
    let self_tail_parameters =
        SelfTailParameterPlan::new(&control, &applications, &control_calls, &ownership);
    debug_assert!(self_tail_parameters.is_valid(
        &control,
        &applications,
        &control_calls,
        &ownership
    ));
    debug_assert!(ownership.is_valid(OwnershipInputs::new(
        &control,
        &applications,
        &optimizations,
        &parameters,
        &control_calls,
        &control_regions,
        &control_frames,
    )));
    Program {
        lowered,
        control,
        applications,
        parameters,
        control_calls,
        control_regions,
        control_frames,
        ownership,
        self_tail_parameters,
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
