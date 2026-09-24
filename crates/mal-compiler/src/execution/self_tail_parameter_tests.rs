use super::*;
use crate::check::ast::Type;
use crate::control::ast::StateId;
use crate::execution::ownership::PatternDestination;
use mal_syntax::source::{FileId, SourceFile};

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
        "walk :: (Buffer<Int32>, Int32) -> Int32 := (values, left) -> {
           if (left == 0i32) then { values.get(0usize) } else { walk(values, left - 1i32) };
         };
         main :: Unit -> Int32 := () -> {
           values := make<Int32>(1usize);
           values.new(7i32);
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
        "walk :: (Buffer<Int32>, Buffer<Int32>, Int32) -> Int32 :=
           (left, right, count) -> {
             if (count == 0i32) then { left.get(0usize) }
             else { walk(right, left, count - 1i32) };
           };
         main :: Unit -> Int32 := () -> {
           values := make<Int32>(1usize);
           values.new(7i32);
           walk(values, values, 1i32) - 7i32;
         };",
    );

    assert!(execution.self_tail_parameters.entries.is_empty());
}

#[test]
fn lends_nested_fields_from_a_preserved_managed_parameter() {
    let execution = execution(
        "Heap :: (Buffer<Int32>, Buffer<Int32>);
         walk :: (Heap, Int32) -> Int32 := (heap, left) -> {
           (nodes, distances) := heap;
           if (left == 0i32)
           then { nodes.get(0usize) + distances.get(0usize) }
           else { walk(heap, left - 1i32) };
         };
         main :: Unit -> Int32 := () -> {
           nodes := make<Int32>(1usize);
           distances := make<Int32>(1usize);
           nodes.new(3i32);
           distances.new(4i32);
           walk((nodes, distances), 1i32) - 7i32;
         };",
    );

    let destination = execution
        .control
        .states
        .iter()
        .enumerate()
        .find_map(|(state_index, state)| {
            state
                .bindings
                .iter()
                .enumerate()
                .find_map(|(binding_index, binding)| {
                    let Pattern::Product { elements, .. } = &binding.pattern else {
                        return None;
                    };
                    (elements.len() == 2
                        && elements.iter().all(|element| {
                            matches!(
                                element,
                                Pattern::Binding {
                                    ty: Type::Buffer(_),
                                    ..
                                }
                            )
                        }))
                    .then(|| {
                        execution
                            .ownership
                            .binding_destination(StateId(state_index), binding_index)
                    })
                })
        })
        .flatten()
        .expect("nested managed destructure destination");
    let PatternDestination::Product(elements) = destination else {
        panic!("nested managed destructure must have product destination");
    };
    assert!(
        elements
            .iter()
            .all(|element| { matches!(element, PatternDestination::Borrow(_)) })
    );
}

#[test]
fn rejects_a_preserved_managed_field_without_borrowed_parameter_authority() {
    let execution = execution(
        "walk :: (Buffer<Int32>, Int32) -> Buffer<Int32> := (values, left) -> {
           if (left == 0i32) then { values } else { walk(values, left - 1i32) };
         };
         main :: Unit -> Int32 := () -> {
           values := make<Int32>(1usize);
           values.new(7i32);
           result := walk(values, 1i32);
           result.get(0usize) - 7i32;
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
