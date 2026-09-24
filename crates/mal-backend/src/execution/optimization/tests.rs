use super::*;
use crate::execution::ClosureUsePlan;
use crate::{anf, core};
use mal_frontend::{check, resolve};
use mal_syntax::parser;
use mal_syntax::source::{FileId, SourceFile};

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
    let core = core::lower(&check::admit_monomorphic(checked).expect("specialize checked program"));
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
    let core = core::lower(&check::admit_monomorphic(checked).expect("specialize checked program"));
    let anf = anf::lower(&core);
    let closure = crate::closure::convert(&anf);
    let control = crate::control::lower(&closure);
    let uses = ClosureUsePlan::new(&closure);
    let applications = ApplicationGraph::new(&closure, &control, &uses);
    let plan = OptimizationPlan::new(&closure, &control, &applications, OptimizationSet::none());

    assert!(plan.fused_sites.is_empty());
    assert!(plan.forwarded_self_arguments.is_empty());
    assert!(plan.direct_targets.is_empty());
    assert!(plan.unique_captures.is_empty());
    assert!(plan.frame_pass_through.is_empty());
    assert!(plan.is_valid(&closure, &control, &applications, OptimizationSet::none()));
}

#[test]
fn selects_only_parameter_fields_preserved_by_every_self_recursive_edge() {
    let source = SourceFile::new(
        FileId::new(93),
        "frame-pass-through-plan.mal",
        "walk :: (Int32, Int32) -> Int32 := (fixed, depth) -> {
           if (depth == 0i32) then { fixed } else {
             child := walk(fixed, depth - 1i32);
             child + fixed;
           };
         };
         changed :: (Int32, Int32) -> Int32 := (fixed, depth) -> {
           if (depth == 0i32) then { fixed } else {
             child := changed(fixed + 1i32, depth - 1i32);
             child + fixed;
           };
         };
         main :: Unit -> Int32 := () -> { walk(1i32, 2i32) + changed(1i32, 2i32) - 8i32; };"
            .into(),
    );
    let checked = mal_frontend::analysis::check(&source).expect("check frame pass-through fixture");
    let core = crate::core::lower(
        &mal_frontend::check::specialize(checked).expect("specialize frame pass-through fixture"),
    );
    let anf = crate::anf::lower(&core);
    let closure = crate::closure::convert(&anf);
    let control = crate::control::lower(&closure);
    let uses = ClosureUsePlan::new(&closure);
    let applications = ApplicationGraph::new(&closure, &control, &uses);
    let enabled = OptimizationSet::none().with(Technique::FramePassThrough);
    let mut plan = OptimizationPlan::new(&closure, &control, &applications, enabled);

    assert_eq!(plan.frame_pass_through.len(), 1);
    assert_eq!(plan.frame_pass_through.values().next().unwrap().len(), 1);
    assert!(plan.is_valid(&closure, &control, &applications, enabled));
    assert!(
        OptimizationPlan::new(&closure, &control, &applications, OptimizationSet::none())
            .frame_pass_through
            .is_empty()
    );
    plan.frame_pass_through.clear();
    assert!(!plan.is_valid(&closure, &control, &applications, enabled));
}

#[test]
fn includes_tail_edges_when_selecting_frame_pass_through_fields() {
    let source = SourceFile::new(
        FileId::new(94),
        "frame-pass-through-tail-edge.mal",
        "walk :: (Int32, Int32) -> Int32 := (fixed, depth) -> {
           if (depth == 0i32) then { fixed }
           else { if (depth == 1i32) then { walk(fixed + 1i32, 0i32) } else {
             child := walk(fixed, depth - 1i32);
             child + fixed;
           }; };
         };
         main :: Unit -> Int32 := () -> { walk(1i32, 2i32) - 3i32; };"
            .into(),
    );
    let checked = mal_frontend::analysis::check(&source).expect("check tail edge fixture");
    let core = crate::core::lower(
        &mal_frontend::check::specialize(checked).expect("specialize tail edge fixture"),
    );
    let anf = crate::anf::lower(&core);
    let closure = crate::closure::convert(&anf);
    let control = crate::control::lower(&closure);
    let uses = ClosureUsePlan::new(&closure);
    let applications = ApplicationGraph::new(&closure, &control, &uses);
    let enabled = OptimizationSet::none().with(Technique::FramePassThrough);
    let plan = OptimizationPlan::new(&closure, &control, &applications, enabled);

    assert!(plan.frame_pass_through.is_empty());
    assert!(plan.is_valid(&closure, &control, &applications, enabled));
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
           callback :: Unit -> Unit := () -> { #value; (); };
           repeat(callback, 2i32);
           0;
         };"
        .into(),
    );
    let checked = mal_frontend::analysis::check(&source).expect("check repeated capture fixture");
    let core = crate::core::lower(
        &mal_frontend::check::specialize(checked).expect("specialize repeated capture fixture"),
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
    let core = core::lower(&check::admit_monomorphic(checked).expect("specialize checked program"));
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
        "create :: Int32 -> (Int32 -> Int32) := (captured) -> { (value) -> { captured + value; }; };\n\
         apply :: ((Int32 -> Int32), Int32) -> Int32 := (function, value) -> { function(value); };\n\
         main :: Unit -> Int32 := () -> { apply(create(40i32), 2i32); };"
            .into(),
    );
    let parsed = parser::parse(&source).expect("parse singleton target fixture");
    let resolved = resolve::resolve(&parsed).expect("resolve singleton target fixture");
    let checked = check::check(&resolved).expect("check singleton target fixture");
    let core = core::lower(&check::admit_monomorphic(checked).expect("specialize checked program"));
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
