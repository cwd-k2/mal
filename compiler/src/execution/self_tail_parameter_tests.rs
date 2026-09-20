use super::*;
use crate::source::{FileId, SourceFile};

fn execution(source: &str) -> crate::execution::Program {
    let source = SourceFile::new(
        FileId::new(95),
        "self-tail-parameter-plan.mal",
        source.into(),
    );
    let checked = crate::pipeline::check(&source).expect("check self-tail fixture");
    let core = crate::core::lower(
        &crate::check::specialize(checked).expect("specialize self-tail fixture"),
    );
    let anf = crate::anf::lower(&core);
    let closure = crate::closure::convert(&anf);
    crate::execution::lower(closure, crate::execution::OptimizationSet::production())
}

#[test]
fn selects_preserved_managed_fields() {
    let execution = execution(
        "walk :: (Packed<Int32>, Int32) -> Int32 := (values, left) -> {
           if (left == 0i32) then { values # 0usize } else { walk(values, left - 1i32) };
         };
         main :: Unit -> Int32 := () -> {
           values := make<Int32>(1usize, (buffer) -> { _ := buffer.new(7i32); (); });
           walk(values, 1i32) - 7i32;
         };",
    );

    assert_eq!(execution.self_tail_parameters.entries.len(), 1);
    assert!(execution.self_tail_parameters.is_valid(
        &execution.control,
        &execution.applications,
        &execution.control_calls,
        &execution.ownership,
    ));
    let mut changed = SelfTailParameterPlan::new(
        &execution.control,
        &execution.applications,
        &execution.control_calls,
        &execution.ownership,
    );
    let function = changed.entries.keys().copied().next().unwrap();
    changed.entries.remove(&function);
    assert!(!changed.is_valid(
        &execution.control,
        &execution.applications,
        &execution.control_calls,
        &execution.ownership,
    ));
}

#[test]
fn rejects_changed_managed_fields() {
    let execution = execution(
        "walk :: (Packed<Int32>, Packed<Int32>, Int32) -> Int32 :=
           (left, right, count) -> {
             if (count == 0i32) then { left # 0usize }
             else { walk(right, left, count - 1i32) };
           };
         main :: Unit -> Int32 := () -> {
           values := make<Int32>(1usize, (buffer) -> { _ := buffer.new(7i32); (); });
           walk(values, values, 1i32) - 7i32;
         };",
    );

    assert!(execution.self_tail_parameters.entries.is_empty());
}

#[test]
fn rejects_an_expanded_aggregate_used_by_the_body() {
    let execution = execution(
        "walk :: (Int32, (Int32, Int32)) -> (Int32, Int32) :=
           (left, pair) -> [return] => {
             (first, second) := pair;
             when (left == 0i32) { return(pair); };
             return(walk(left - 1i32, (first, second + 1i32)));
           };
         main :: Unit -> Int32 := () -> {
           (_, result) := walk(1i32, (2i32, 3i32));
           result - 4i32;
         };",
    );

    assert!(execution.self_tail_parameters.entries.is_empty());
}
