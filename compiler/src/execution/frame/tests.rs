use super::*;
use crate::execution::{ApplicationGraph, ClosureUsePlan, ContinuationGraph, TailCallPlan};
use crate::source::{FileId, SourceFile};
use crate::{anf, check, closure, control, core, parser, resolve};

#[test]
fn validates_exact_frame_sites_and_payloads() {
    let source = SourceFile::new(
        FileId::new(72),
        "control-frame-validation.mal",
        "walk :: (Int32, Symbol) -> Symbol := \\(depth, prefix) {\n\
           if (depth == 0i32)\n\
           then { prefix }\n\
           else {\n\
             append :: Symbol -> Symbol := \\(suffix) { prefix + suffix; };\n\
             child := walk(depth - 1i32, prefix);\n\
             append(child);\n\
           };\n\
         };\n\
         main :: Unit -> Int32 := \\() { Int32(#walk(2i32, \"x\")) - 3i32; };"
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
    let tail_calls = TailCallPlan::new(&closure, &control, &applications);
    let continuations = ContinuationGraph::new(&control, &applications, &tail_calls);
    let regions = ControlRegionPlan::new(&control, &continuations);
    let calls = ControlCallPlan::new(&control, &applications, &tail_calls, &regions);
    let mut plan = ControlFramePlan::new(&control, &regions, &calls);

    assert!(plan.is_valid(&control, &regions, &calls));
    assert!(!plan.frames.is_empty());

    let frame_site = *plan.frames.keys().next().expect("frame site");
    let frame = plan.frames.remove(&frame_site).expect("frame");
    assert!(!plan.is_valid(&control, &regions, &calls));
    plan.frames.insert(frame_site, frame);
    assert!(plan.is_valid(&control, &regions, &calls));
}
