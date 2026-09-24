use super::*;
use mal_syntax::source::{FileId, SourceFile};

#[test]
fn validates_the_exact_symbol_concat_decisions() {
    let source = SourceFile::new(
        FileId::new(88),
        "llvm-optimization-plan.mal",
        "main :: Unit -> Int32 := () -> { left := \"a\" + \"b\"; result := left + \"c\"; (#result).i32; };"
            .into(),
    );
    let checked = mal_frontend::analysis::check(&source).expect("check LLVM optimization fixture");
    let core = crate::core::lower(
        &mal_frontend::check::specialize(checked).expect("specialize checked program"),
    );
    let anf = crate::anf::lower(&core);
    let closure = crate::closure::convert(&anf);
    let execution =
        crate::execution::lower(closure, crate::execution::OptimizationSet::production());
    let enabled = OptimizationSet::none().with(Technique::SymbolConcatReuse);
    let mut plan = OptimizationPlan::new(&execution, enabled);

    assert!(plan.is_valid(&execution, enabled));
    let decision = *plan
        .symbol_concatenations
        .keys()
        .next()
        .expect("consuming concat decision");
    plan.symbol_concatenations.remove(&decision);
    assert!(!plan.is_valid(&execution, enabled));
    assert!(
        OptimizationPlan::new(&execution, OptimizationSet::none())
            .symbol_concatenations
            .is_empty()
    );
    assert!(
        OptimizationPlan::new(&execution, OptimizationSet::none())
            .local_control_storage_functions
            .is_empty()
    );
    assert!(
        OptimizationPlan::new(&execution, OptimizationSet::none())
            .local_control_top_functions
            .is_empty()
    );
    assert!(
        OptimizationPlan::new(&execution, OptimizationSet::none())
            .self_tail_parameters
            .is_empty()
    );
}

#[test]
fn selects_recursive_functions_with_control_frames_for_local_top_storage() {
    let source = SourceFile::new(
        FileId::new(89),
        "llvm-local-control-top.mal",
        "sum :: Int32 -> Int32 := (value) -> { if (value == 0i32) then { 0i32 } else { rest := sum(value - 1i32); value + rest; }; }; main :: Unit -> Int32 := () -> { sum(4i32) - 10i32; };"
            .into(),
    );
    let checked = mal_frontend::analysis::check(&source).expect("check local control top fixture");
    let core = crate::core::lower(
        &mal_frontend::check::specialize(checked).expect("specialize checked program"),
    );
    let anf = crate::anf::lower(&core);
    let closure = crate::closure::convert(&anf);
    let execution =
        crate::execution::lower(closure, crate::execution::OptimizationSet::production());
    let enabled = OptimizationSet::none().with(Technique::LocalControlTop);
    let mut plan = OptimizationPlan::new(&execution, enabled);

    assert_eq!(plan.local_control_top_functions.len(), 1);
    assert!(plan.is_valid(&execution, enabled));
    assert!(
        OptimizationPlan::new(&execution, OptimizationSet::none())
            .local_control_top_functions
            .is_empty()
    );
    plan.local_control_top_functions.clear();
    assert!(!plan.is_valid(&execution, enabled));

    let enabled = OptimizationSet::none().with(Technique::LocalControlStorage);
    let mut plan = OptimizationPlan::new(&execution, enabled);
    assert_eq!(plan.local_control_storage_functions.len(), 1);
    assert!(plan.local_control_top_functions.is_empty());
    assert!(plan.is_valid(&execution, enabled));
    plan.local_control_storage_functions.clear();
    assert!(!plan.is_valid(&execution, enabled));
}
