use super::*;
use crate::execution::{
    ApplicationGraph, ClosureUsePlan, ContinuationGraph, OptimizationPlan, OptimizationSet,
};
use crate::{anf, check, closure, control, core, resolve};
use mal_syntax::parser;
use mal_syntax::source::{FileId, SourceFile};

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
    let core = core::lower(&check::admit_monomorphic(checked).expect("specialize checked program"));
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
    let mut plan = ControlFramePlan::new(&control, &regions, &calls, &optimizations);

    assert!(plan.is_valid(&control, &regions, &calls, &optimizations));
    assert!(!plan.frames.is_empty());
    assert!(!plan.resumes.pairs.is_empty());

    let frame_site = *plan.frames.keys().next().expect("frame site");
    let mut frame = plan.frames.remove(&frame_site).expect("frame");
    assert!(!plan.is_valid(&control, &regions, &calls, &optimizations));
    let resume = frame.resume;
    frame.resume = control.functions[0].entry;
    plan.frames.insert(frame_site, frame.clone());
    assert!(!plan.is_valid(&control, &regions, &calls, &optimizations));
    frame.resume = resume;
    plan.frames.insert(frame_site, frame);
    assert!(plan.is_valid(&control, &regions, &calls, &optimizations));

    let relation = *plan
        .resumes
        .compatible
        .iter()
        .next()
        .expect("compatible frame resume");
    plan.resumes.compatible.remove(&relation);
    assert!(!plan.is_valid(&control, &regions, &calls, &optimizations));
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
    let core = core::lower(&check::admit_monomorphic(checked).expect("specialize checked program"));
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
    let plan = ControlFramePlan::new(&control, &regions, &calls, &optimizations);

    assert!(plan.is_valid(&control, &regions, &calls, &optimizations));
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

#[test]
fn replaces_a_retired_frame_only_on_a_must_resume_path() {
    let source = SourceFile::new(
        FileId::new(91),
        "retired-frame-replacement.mal",
        "walk :: (Int32, Int32) -> Int32 := (depth, value) -> {\n\
           if (depth == 0i32) then { value } else {\n\
             first := walk(depth - 1i32, value);\n\
             second := walk(depth - 1i32, first);\n\
             first + second;\n\
           };\n\
         };\n\
         main :: Unit -> Int32 := () -> { walk(4i32, 1i32); };"
            .into(),
    );
    let parsed = parser::parse(&source).expect("parse replacement fixture");
    let resolved = resolve::resolve(&parsed).expect("resolve replacement fixture");
    let checked = check::check(&resolved).expect("check replacement fixture");
    let core = core::lower(&check::admit_monomorphic(checked).expect("specialize fixture"));
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
    let mut plan = ControlFramePlan::new(&control, &regions, &calls, &optimizations);

    assert!(plan.is_valid(&control, &regions, &calls, &optimizations));
    assert_eq!(plan.replacements.replacements.len(), 1);
    let (&site, &retired) = plan
        .replacements
        .replacements
        .iter()
        .next()
        .expect("second recursive call replaces the first frame");
    assert!(plan.frames.contains_key(&site));
    assert!(plan.frames.contains_key(&retired));
    assert_ne!(site, retired);
    plan.replacements.replacements.remove(&site);
    assert!(!plan.is_valid(&control, &regions, &calls, &optimizations));
}

#[test]
fn rejects_a_replacement_reached_from_both_entry_and_resume() {
    let plan = frame_plan(
        "walk :: Int32 -> Int32 := (depth) -> {\n\
           if (depth == 0i32) then { 0i32 } else {\n\
             first := if (depth == 1i32) then { walk(depth - 1i32) } else { 0i32 };\n\
             second := walk(depth - 1i32);\n\
             first + second;\n\
           };\n\
         };\n\
         main :: Unit -> Int32 := () -> { walk(3i32); };",
    );

    assert!(plan.replacements.replacements.is_empty());
}

#[test]
fn rejects_a_replacement_after_distinct_retired_frames_merge() {
    let plan = frame_plan(
        "walk :: Int32 -> Int32 := (depth) -> {\n\
           if (depth == 0i32) then { 0i32 } else {\n\
             first := if (depth == 1i32)\n\
               then { walk(depth - 1i32) }\n\
               else { walk(depth - 1i32) };\n\
             second := walk(depth - 1i32);\n\
             first + second;\n\
           };\n\
         };\n\
         main :: Unit -> Int32 := () -> { walk(3i32); };",
    );

    assert!(plan.replacements.replacements.is_empty());
}

#[test]
fn a_frame_call_barrier_starts_the_next_retired_relation() {
    let plan = frame_plan(
        "walk :: Int32 -> Int32 := (depth) -> {\n\
           if (depth == 0i32) then { 1i32 } else {\n\
             first := walk(depth - 1i32);\n\
             second := walk(depth - 1i32);\n\
             third := walk(depth - 1i32);\n\
             first + second + third;\n\
           };\n\
         };\n\
         main :: Unit -> Int32 := () -> { walk(3i32); };",
    );

    assert_eq!(plan.replacements.replacements.len(), 2);
    let (&third, &second) = plan
        .replacements
        .replacements
        .iter()
        .find(|(_, retired)| plan.replacements.replacements.contains_key(retired))
        .expect("third frame replaces the retired second frame");
    let first = plan.replacements.replacements[&second];
    assert_eq!(plan.replacement(third), Some(second));
    assert_ne!(plan.replacement(third), Some(first));
}

fn frame_plan(source: &str) -> ControlFramePlan {
    let source = SourceFile::new(FileId::new(92), "frame-replacement.mal", source.into());
    let parsed = parser::parse(&source).expect("parse replacement fixture");
    let resolved = resolve::resolve(&parsed).expect("resolve replacement fixture");
    let checked = check::check(&resolved).expect("check replacement fixture");
    let core = core::lower(&check::admit_monomorphic(checked).expect("specialize fixture"));
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
    ControlFramePlan::new(&control, &regions, &calls, &optimizations)
}
