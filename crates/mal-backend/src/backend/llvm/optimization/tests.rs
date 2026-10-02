use super::*;
use mal_syntax::source::{FileId, SourceFile};

/// Lowers one source text to the execution plan that optimization planning reads.
fn execution(id: u32, name: &str, text: &str) -> crate::execution::Program {
    let source = SourceFile::new(FileId::new(id), name, text.into());
    let checked = mal_frontend::analysis::check(&source).expect("check optimization fixture");
    let core = crate::core::lower(
        &mal_frontend::check::specialize(checked).expect("specialize checked program"),
    );
    let anf = crate::anf::lower(&core);
    let closure = crate::closure::convert(&anf);
    crate::execution::lower(closure, crate::execution::OptimizationSet::production())
}

#[test]
fn validates_the_exact_symbol_concat_decisions() {
    let execution = execution(
        88,
        "llvm-optimization-plan.mal",
        "main :: Unit -> Int32 := () -> { left := \"a\" + \"b\"; result := left + \"c\"; (#result).i32; };",
    );
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
    let execution = execution(
        89,
        "llvm-local-control-top.mal",
        "sum :: Int32 -> Int32 := (value) -> { if (value == 0i32) then { 0i32 } else { rest := sum(value - 1i32); value + rest; }; }; main :: Unit -> Int32 := () -> { sum(4i32) - 10i32; };",
    );
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

#[test]
fn transfers_only_byte_conversions_whose_operand_dies_there() {
    let execution = execution(
        90,
        "llvm-byte-conversion.mal",
        "main :: Unit -> Int32 := () -> { buffer := *\"ab\"; kept := *buffer; text := *buffer; (#text + #kept).i32 - 4i32; };",
    );
    let enabled = OptimizationSet::none().with(Technique::ByteConversionTransfer);
    let mut plan = OptimizationPlan::new(&execution, enabled);

    // `*"ab"` has no binding operand and the first `*buffer` is followed by another use; only the last moves.
    assert_eq!(plan.byte_conversions.len(), 1);
    assert!(plan.is_valid(&execution, enabled));
    assert!(
        OptimizationPlan::new(&execution, OptimizationSet::none())
            .byte_conversions
            .is_empty()
    );
    plan.byte_conversions.clear();
    assert!(!plan.is_valid(&execution, enabled));
}
