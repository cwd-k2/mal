use super::*;
use crate::c_emit::types::TypeRegistry;
use crate::execution::{ApplicationGraph, ContinuationGraph, TailCallPlan};
use crate::source::{FileId, SourceFile};
use crate::{anf, check, closure, control, core, parser, resolve};

#[test]
fn validates_exact_frame_closure_and_arena_sets() {
    let source = SourceFile::new(
        FileId::new(72),
        "control-frame-validation.mal",
        "walk :: (Int32, Symbol) -> Symbol := \\(depth, prefix) {\n\
           if (depth == 0i32) then { prefix } else {\n\
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
    let mut types = TypeRegistry::default();
    types.collect_program_body(&closure);
    let calls = ControlCallPlan::new(&control, &applications, &tail_calls, &regions);
    let mut plan = ControlFramePlan::new(&control, &regions, &calls, &types, &closure_uses);

    assert!(plan.is_valid(&control, &regions, &calls, &types, &closure_uses));
    assert!(!plan.frames.is_empty());
    assert!(!plan.closures_crossing_suspension.is_empty());
    assert!(!plan.region_arenas.is_empty());
    assert_eq!(plan.homogeneous_regions.len(), 1);
    assert!(plan.has_homogeneous_arenas());
    assert!(!plan.has_heterogeneous_arenas());

    let frame_site = *plan.frames.keys().next().expect("frame site");
    let frame = plan.frames.remove(&frame_site).expect("frame");
    assert!(!plan.is_valid(&control, &regions, &calls, &types, &closure_uses));
    plan.frames.insert(frame_site, frame);

    let closure_id = *plan
        .closures_crossing_suspension
        .iter()
        .next()
        .expect("crossing closure");
    plan.closures_crossing_suspension.remove(&closure_id);
    assert!(!plan.is_valid(&control, &regions, &calls, &types, &closure_uses));
    plan.closures_crossing_suspension.insert(closure_id);

    let region = *plan.region_arenas.keys().next().expect("arena region");
    let arena = plan.region_arenas.remove(&region).expect("arena");
    assert!(!plan.is_valid(&control, &regions, &calls, &types, &closure_uses));
    plan.region_arenas.insert(region, arena);

    let homogeneous = plan
        .homogeneous_regions
        .remove(&region)
        .expect("homogeneous region");
    assert!(!plan.is_valid(&control, &regions, &calls, &types, &closure_uses));
    plan.homogeneous_regions.insert(region, homogeneous);

    assert!(plan.is_valid(&control, &regions, &calls, &types, &closure_uses));
}
