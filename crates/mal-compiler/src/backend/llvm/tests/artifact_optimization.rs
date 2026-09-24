use super::super::*;
use mal_syntax::source::{FileId, SourceFile};

#[test]
fn selects_symbol_storage_reuse_only_when_enabled() {
    let source = SourceFile::new(
        FileId::new(87),
        "symbol-concat-optimization.mal",
        "main :: Unit -> Int32 := () -> { prefix := \"a\" + \"b\"; text := prefix + \"c\"; (#text).i32; };"
            .into(),
    );
    let checked = mal_frontend::analysis::check(&source).expect("check Symbol concat fixture");
    let core = crate::core::lower(
        &mal_frontend::check::specialize(checked).expect("specialize checked program"),
    );
    let anf = crate::anf::lower(&core);
    let closure = crate::closure::convert(&anf);
    let execution =
        crate::execution::lower(closure, crate::execution::OptimizationSet::production());
    let target = || Target {
        triple: "x86_64-unknown-linux-gnu",
        data_layout: "e-p:64:64",
    };
    let baseline = generate(&execution, target(), OptimizationSet::none())
        .expect("baseline Symbol concat is supported");
    let optimized = generate(&execution, target(), OptimizationSet::production())
        .expect("optimized Symbol concat is supported");

    assert!(
        !baseline
            .module
            .contains("call void @mal_runtime_symbol_concatenate_consuming_left")
    );
    assert!(
        baseline
            .module
            .contains("call void @mal_runtime_symbol_concatenate(")
    );
    assert!(
        optimized
            .module
            .contains("call void @mal_runtime_symbol_concatenate_consuming_left")
    );
}

#[test]
fn scalarizes_preserved_self_tail_parameter_fields_only_when_enabled() {
    let source = SourceFile::new(
        FileId::new(99),
        "self-tail-parameter-optimization.mal",
        "walk :: (Buffer<Int32>, Int32) -> Int32 := (values, left) -> {
           if (left == 0i32) then { values.get(0usize) } else { walk(values, left - 1i32) };
         };
         main :: Unit -> Int32 := () -> {
           values := make<Int32>(1usize);
           values.new(7i32);
           walk(values, 2i32) - 7i32;
         };"
        .into(),
    );
    let checked = mal_frontend::analysis::check(&source).expect("check self-tail fixture");
    let core = crate::core::lower(
        &mal_frontend::check::specialize(checked).expect("specialize self-tail fixture"),
    );
    let anf = crate::anf::lower(&core);
    let closure = crate::closure::convert(&anf);
    let execution =
        crate::execution::lower(closure, crate::execution::OptimizationSet::production());
    let target = || Target {
        triple: "x86_64-unknown-linux-gnu",
        data_layout: "e-p:64:64",
    };
    let baseline = generate(&execution, target(), OptimizationSet::none())
        .expect("baseline self-tail fixture is supported");
    let optimized = generate(&execution, target(), OptimizationSet::production())
        .expect("optimized self-tail fixture is supported");

    assert!(!baseline.module.contains("mal_self_tail_entry_"));
    assert!(optimized.module.contains("mal_self_tail_entry_"));
}
