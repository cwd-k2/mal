use super::super::direct_function_id;
use super::super::{ClosureUsePlan, ContinuationGraph, OptimizationSet};
use super::*;
use crate::source::{FileId, SourceFile};
use crate::{anf, check, closure, control, core, parser, resolve};

#[test]
fn preserves_direct_edges_without_admitting_recursive_c_call_cycles() {
    let source = SourceFile::new(
        FileId::new(69),
        "control-call-plan.mal",
        "helper :: Int32 -> Int32 := (x) -> { x + 1i32; };\n\
         recursive :: Int32 -> Int32 := (n) -> {\n\
           if (n == 0i32)\n\
           then { helper(n) }\n\
           else {\n\
               child := recursive(n - 1i32);\n\
               helper(child);\n\
             };\n\
         };\n\
         tail :: Int32 -> Int32 := (n) -> {\n\
           if (n == 0i32)\n\
           then { n }\n\
           else { tail(n - 1i32) };\n\
         };"
        .into(),
    );
    let parsed = parser::parse(&source).expect("parse plan fixture");
    let resolved = resolve::resolve(&parsed).expect("resolve plan fixture");
    let checked = check::check(&resolved).expect("check plan fixture");
    let core = core::lower(&check::admit_monomorphic(checked).expect("specialize checked program"));
    let anf = anf::lower(&core);
    let closure = closure::convert(&anf);
    let control = control::lower(&closure);
    let uses = ClosureUsePlan::new(&closure);
    let applications = ApplicationGraph::new(&closure, &control, &uses);
    let optimizations = OptimizationPlan::new(
        &closure,
        &control,
        &applications,
        OptimizationSet::production(),
    );
    let continuations = ContinuationGraph::new(&applications, &optimizations);
    let regions = ControlRegionPlan::new(&control, &continuations);
    let mut plan = ControlCallPlan::new(&control, &applications, &optimizations, &regions);

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
    assert!(recursive_modes.contains(&ControlCallMode::DirectRegion(recursive.id)));
    assert!(recursive_modes.contains(&ControlCallMode::Direct(helper)));
    let recursive_region = regions
        .function_region(recursive.id)
        .expect("recursive function has a control region");
    assert!(!plan.requires_common_control(recursive_region));

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

    assert!(plan.is_valid(&control, &applications, &optimizations, &regions));
    let site = *plan.modes.keys().next().expect("application mode");
    let mode = plan.modes.remove(&site).expect("application mode");
    assert!(!plan.is_valid(&control, &applications, &optimizations, &regions));
    plan.modes.insert(site, mode);
    assert!(plan.is_valid(&control, &applications, &optimizations, &regions));
}

#[test]
fn closes_known_edges_over_type_compatible_indirect_targets() {
    let source = SourceFile::new(
        FileId::new(70),
        "indirect-control-cycle.mal",
        "apply :: ((Int32 -> Int32), Int32) -> Int32 := (function, value) -> {\n\
           function(value);\n\
         };\n\
         identity :: Int32 -> Int32 := (value) -> { value; };\n\
         main :: Unit -> Int32 := () -> {\n\
           recurse :: Int32 -> Int32 := (value) -> {\n\
             if (value == 0i32)\n\
             then { 0i32 }\n\
             else {\n\
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
    let core = core::lower(&check::admit_monomorphic(checked).expect("specialize checked program"));
    let anf = anf::lower(&core);
    let closure = closure::convert(&anf);
    let control = control::lower(&closure);
    let uses = ClosureUsePlan::new(&closure);
    let applications = ApplicationGraph::new(&closure, &control, &uses);
    let optimizations = OptimizationPlan::new(
        &closure,
        &control,
        &applications,
        OptimizationSet::production(),
    );
    let continuations = ContinuationGraph::new(&applications, &optimizations);
    let regions = ControlRegionPlan::new(&control, &continuations);
    let plan = ControlCallPlan::new(&control, &applications, &optimizations, &regions);
    let apply = top_level_function_id(&closure, "apply");
    let identity = top_level_function_id(&closure, "identity");

    assert!(
        regions
            .ids()
            .any(|region| plan.requires_common_control(region))
    );

    assert!(control.states.iter().enumerate().any(|(index, state)| {
        let (Terminator::TailCall { callee, .. } | Terminator::Call { callee, .. }) =
            &state.terminator
        else {
            return false;
        };
        direct_function_id(&uses, callee) == Some(apply)
            && plan.mode(StateId(index)) == Some(ControlCallMode::DirectRegion(apply))
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
            && applications
                .targets(StateId(index))
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
        "apply :: ((Int32 -> Int32), Int32) -> Int32 := (function, value) -> {\n\
           function(value);\n\
         };\n\
         main :: Unit -> Int32 := () -> {\n\
           recurse :: Int32 -> Int32 := (value) -> {\n\
             if (value == 0i32)\n\
             then { 0i32 }\n\
             else { apply(recurse, value - 1i32) };\n\
           };\n\
           recurse(2i32);\n\
         };"
        .into(),
    );
    let parsed = parser::parse(&source).expect("parse tail forwarder fixture");
    let resolved = resolve::resolve(&parsed).expect("resolve tail forwarder fixture");
    let checked = check::check(&resolved).expect("check tail forwarder fixture");
    let core = core::lower(&check::admit_monomorphic(checked).expect("specialize checked program"));
    let anf = anf::lower(&core);
    let closure = closure::convert(&anf);
    let control = control::lower(&closure);
    let uses = ClosureUsePlan::new(&closure);
    let applications = ApplicationGraph::new(&closure, &control, &uses);
    let optimizations = OptimizationPlan::new(
        &closure,
        &control,
        &applications,
        OptimizationSet::production(),
    );
    let continuations = ContinuationGraph::new(&applications, &optimizations);
    let regions = ControlRegionPlan::new(&control, &continuations);
    let plan = ControlCallPlan::new(&control, &applications, &optimizations, &regions);
    let apply = top_level_function_id(&closure, "apply");

    assert!(regions.ids().next().is_none());

    assert!(control.states.iter().enumerate().any(|(index, state)| {
        let Terminator::TailCall { callee, .. } = &state.terminator else {
            return false;
        };
        let site = StateId(index);
        direct_function_id(&uses, callee) == Some(apply)
            && applications.direct_target(site) == Some(apply)
            && plan.mode(site) == Some(ControlCallMode::DirectSelfTail)
            && plan.forwarded_self_argument(site).is_some()
    }));
    assert!(control.states.iter().enumerate().any(|(index, state)| {
        let (Terminator::TailCall { callee, .. } | Terminator::Call { callee, .. }) =
            &state.terminator
        else {
            return false;
        };
        let site = StateId(index);
        let Some([target]) = applications.targets(site) else {
            return false;
        };
        direct_function_id(&uses, callee).is_none()
            && regions.site_region(site).is_none()
            && plan.mode(site) == Some(ControlCallMode::Direct(*target))
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
