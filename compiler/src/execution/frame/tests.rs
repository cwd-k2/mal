use super::*;
use crate::execution::{
    ApplicationGraph, ClosureUsePlan, ContinuationGraph, OptimizationPlan, OptimizationSet,
};
use crate::source::{FileId, SourceFile};
use crate::{anf, check, closure, control, core, parser, resolve};

#[test]
fn validates_exact_frame_sites_and_payloads() {
    let source = SourceFile::new(
        FileId::new(72),
        "control-frame-validation.mal",
        "walk :: (Int32, Symbol) -> Symbol := (depth, prefix) -> {\n\
           if (depth == 0i32)\n\
           then { prefix }\n\
           else {\n\
             append :: Symbol -> Symbol := (suffix) -> { prefix + suffix; };\n\
             child := walk(depth - 1i32, prefix);\n\
             append(child);\n\
           };\n\
         };\n\
         main :: Unit -> Int32 := () -> { (#walk(2i32, \"x\")).i32 - 3i32; };"
            .into(),
    );
    let parsed = parser::parse(&source).expect("parse control frame fixture");
    let resolved = resolve::resolve(&parsed).expect("resolve control frame fixture");
    let checked = check::check(&resolved).expect("check control frame fixture");
    let core = core::lower(&checked);
    let anf = anf::lower(&core);
    let closure = closure::convert(&anf);
    let control = control::lower(&closure);
    let closure_uses = ClosureUsePlan::new(&closure);
    let applications = ApplicationGraph::new(&closure, &control, &closure_uses);
    let optimizations = OptimizationPlan::new(
        &closure,
        &control,
        &applications,
        OptimizationSet::production(),
    );
    let continuations = ContinuationGraph::new(&applications, &optimizations);
    let regions = ControlRegionPlan::new(&control, &continuations);
    let calls = ControlCallPlan::new(&control, &applications, &optimizations, &regions);
    let mut plan = ControlFramePlan::new(&control, &regions, &calls);

    assert!(plan.is_valid(&control, &regions, &calls));
    assert!(!plan.frames.is_empty());
    assert!(!plan.resumes.pairs.is_empty());

    let frame_site = *plan.frames.keys().next().expect("frame site");
    let mut frame = plan.frames.remove(&frame_site).expect("frame");
    assert!(!plan.is_valid(&control, &regions, &calls));
    let resume = frame.resume;
    frame.resume = control.functions[0].entry;
    plan.frames.insert(frame_site, frame.clone());
    assert!(!plan.is_valid(&control, &regions, &calls));
    frame.resume = resume;
    plan.frames.insert(frame_site, frame);
    assert!(plan.is_valid(&control, &regions, &calls));

    let relation = *plan
        .resumes
        .compatible
        .iter()
        .next()
        .expect("compatible frame resume");
    plan.resumes.compatible.remove(&relation);
    assert!(!plan.is_valid(&control, &regions, &calls));
}

#[test]
fn distinguishes_resumable_and_unreachable_heterogeneous_frame_pairs() {
    let source = SourceFile::new(
        FileId::new(83),
        "heterogeneous-frame-resume.mal",
        "Answer :: Int32;\n\
         Continuation :: Int32 -> Answer;\n\
         Computation :: Continuation -> Answer;\n\
         Next :: Int32 -> Computation;\n\
         Mapper :: Int32 -> Int32;\n\
         pure :: Int32 -> Computation := (value) -> {\n\
           (continuation) -> { value[continuation] };\n\
         };\n\
         bind :: (Computation, Next) -> Computation := (computation, next) -> {\n\
           (continuation) -> {\n\
             resume :: Continuation := (value) -> { continuation[value[next]]; };\n\
             resume[computation];\n\
           };\n\
         };\n\
         map :: (Computation, Mapper) -> Computation := (computation, mapper) -> {\n\
           next :: Next := (value) -> { value[mapper][pure]; };\n\
           (computation, next)[bind];\n\
         };\n\
         main :: Unit -> Int32 := () -> {\n\
           mapped := (10[pure], (value) -> { value * 2 })[map];\n\
           (value) -> { value - 20 }[mapped];\n\
         };"
        .into(),
    );
    let parsed = parser::parse(&source).expect("parse heterogeneous frame fixture");
    let resolved = resolve::resolve(&parsed).expect("resolve heterogeneous frame fixture");
    let checked = check::check(&resolved).expect("check heterogeneous frame fixture");
    let core = core::lower(&checked);
    let anf = anf::lower(&core);
    let closure = closure::convert(&anf);
    let control = control::lower(&closure);
    let closure_uses = ClosureUsePlan::new(&closure);
    let applications = ApplicationGraph::new(&closure, &control, &closure_uses);
    let optimizations = OptimizationPlan::new(
        &closure,
        &control,
        &applications,
        OptimizationSet::production(),
    );
    let continuations = ContinuationGraph::new(&applications, &optimizations);
    let regions = ControlRegionPlan::new(&control, &continuations);
    let calls = ControlCallPlan::new(&control, &applications, &optimizations, &regions);
    let plan = ControlFramePlan::new(&control, &regions, &calls);

    assert!(plan.is_valid(&control, &regions, &calls));
    assert!(
        plan.resumes
            .pairs
            .iter()
            .any(|(exit, frame)| { plan.resume(*exit, *frame) == Some(FrameResume::Resume) })
    );
    assert!(
        plan.resumes
            .pairs
            .iter()
            .any(|(exit, frame)| { plan.resume(*exit, *frame) == Some(FrameResume::Unreachable) })
    );
}
