use mal_syntax::source::{FileId, SourceFile};

use super::Plan;
use super::plan::ProducerResult;
use crate::closure::ast::AtomKind;

fn plan(text: &str) -> Plan {
    let source = SourceFile::new(
        FileId::new(99),
        "continuation-specialization.mal",
        text.into(),
    );
    let checked = mal_frontend::analysis::check(&source).expect("check continuation fixture");
    let core = crate::core::lower(
        &mal_frontend::check::specialize(checked).expect("specialize continuation fixture"),
    );
    let anf = crate::anf::lower(&core);
    let closure = crate::closure::convert(&anf);
    Plan::new(&crate::call_pattern::specialize(closure))
}

#[test]
fn admits_one_application_of_a_function_valued_call_result() {
    let plan = plan(
        "create :: Int32 -> (Int32 -> Int32) := (captured) -> (value) -> captured + value;
         main :: Unit -> Int32 := () -> {
             function := create(40i32);
             alias := function;
             alias(2i32) - 42i32;
         };",
    );

    assert_eq!(plan.demands.len(), 1);
    assert_eq!(plan.steps.len(), 1);
    assert!(matches!(plan.steps[0].result, ProducerResult::Closure(_)));
}

#[test]
fn follows_a_direct_call_result_to_its_closure_creator() {
    let plan = plan(
        "create :: Int32 -> (Int32 -> Int32) := (captured) -> (value) -> captured + value;
         forward :: Int32 -> (Int32 -> Int32) := (captured) -> create(captured);
         main :: Unit -> Int32 := () -> forward(40i32)(2i32) - 42i32;",
    );

    assert_eq!(plan.demands.len(), 1);
    assert_eq!(plan.steps.len(), 2);
    assert!(
        plan.steps
            .iter()
            .any(|step| matches!(step.result, ProducerResult::Call(_)))
    );
    assert!(
        plan.steps
            .iter()
            .any(|step| matches!(step.result, ProducerResult::Closure(_)))
    );
}

#[test]
fn follows_every_result_join_predecessor() {
    let plan = plan(
        "first :: Int32 -> (Int32 -> Int32) := (captured) -> (value) -> captured + value;
         second :: Int32 -> (Int32 -> Int32) := (captured) -> (value) -> captured - value;
         choose :: (Bool, Int32) -> (Int32 -> Int32) := (condition, captured) ->
             if (condition) then first(captured) else second(captured);
         main :: Unit -> Int32 := () -> choose(true, 40i32)(2i32) - 42i32;",
    );

    assert_eq!(plan.demands.len(), 1);
    assert_eq!(
        plan.steps
            .iter()
            .filter(|step| matches!(step.result, ProducerResult::Call(_)))
            .count(),
        2
    );
    assert_eq!(
        plan.steps
            .iter()
            .filter(|step| matches!(step.result, ProducerResult::Closure(_)))
            .count(),
        2
    );
}

#[test]
fn rejects_a_result_applied_more_than_once() {
    let plan = plan(
        "create :: Int32 -> (Int32 -> Int32) := (captured) -> (value) -> captured + value;
         main :: Unit -> Int32 := () -> {
             function := create(40i32);
             function(1i32) + function(2i32) - 83i32;
         };",
    );

    assert!(plan.demands.is_empty());
}

#[test]
fn rejects_a_result_that_escapes_into_an_aggregate() {
    let plan = plan(
        "create :: Int32 -> (Int32 -> Int32) := (captured) -> (value) -> captured + value;
         main :: Unit -> Int32 := () -> {
             function := create(40i32);
             kept := (function, 2i32);
             (selected, value) := kept;
             selected(value) - 42i32;
         };",
    );

    assert!(plan.demands.is_empty());
}

#[test]
fn rejects_an_application_argument_computed_after_the_producer() {
    let plan = plan(
        "create :: Int32 -> (Int32 -> Int32) := (captured) -> (value) -> captured + value;
         main :: Unit -> Int32 := () -> {
             function := create(40i32);
             argument := 1i32 + 1i32;
             function(argument) - 42i32;
         };",
    );

    assert!(plan.demands.is_empty());
}

#[test]
fn rejects_an_operation_between_the_producer_and_consumer() {
    let plan = plan(
        "create :: Int32 -> (Int32 -> Int32) := (captured) -> (value) -> captured + value;
         main :: Unit -> Int32 := () -> {
             values := make<Int32>(1usize);
             function := create(40i32);
             values.new(1i32);
             function(2i32) - 42i32;
         };",
    );

    assert!(plan.demands.is_empty());
}

#[test]
fn finds_the_final_consumer_of_a_recursive_state_chain() {
    let plan = plan(
        "opaque State<S, A> :: S -> (S, A);
         pureState<S, A> :: A -> State<S, A> := (value) -> (state) -> (state, value);
         bindState<S, A, B> :: (State<S, A>, A -> State<S, B>) -> State<S, B> :=
             (action, following) -> (state) -> {
                 (middle, value) := action(state);
                 next := following(value);
                 next(middle);
             };
         fresh :: State<USize, USize> := (state) -> (state + 1usize, state);
         step :: (USize, USize) -> State<USize, USize> := (sum, value) ->
             bindState<USize, USize, USize>(fresh, (label) -> pureState<USize, USize>(sum + value + label));
         fold :: (USize, USize, USize) -> State<USize, USize> := (index, count, sum) -> [return] => {
             when (index == count) return(pureState<USize, USize>(sum));
             return(bindState<USize, USize, USize>(step(sum, 1usize), (next) -> fold(index + 1usize, count, next)));
         };
         main :: Unit -> Int32 := () -> {
             action := fold(0usize, 3usize, 0usize);
             (_, checksum) := action(0usize);
             checksum.u8.i32;
         };",
    );

    assert!(
        plan.demands
            .iter()
            .any(|demand| matches!(demand.argument.kind, AtomKind::Integer(0)))
    );
    assert_eq!(plan.steps.len(), 4, "{:#?}", plan.steps);
    assert_eq!(
        plan.steps
            .iter()
            .filter(|step| matches!(step.result, ProducerResult::Call(_)))
            .count(),
        2
    );
}

#[test]
fn validates_the_exact_demand_set() {
    let source = SourceFile::new(
        FileId::new(100),
        "continuation-specialization-validation.mal",
        "create :: Int32 -> (Int32 -> Int32) := (captured) -> (value) -> captured + value;
         main :: Unit -> Int32 := () -> create(40i32)(2i32) - 42i32;"
            .into(),
    );
    let checked = mal_frontend::analysis::check(&source).expect("check validation fixture");
    let core = crate::core::lower(
        &mal_frontend::check::specialize(checked).expect("specialize validation fixture"),
    );
    let closure =
        crate::call_pattern::specialize(crate::closure::convert(&crate::anf::lower(&core)));
    let mut plan = Plan::new(&closure);

    assert!(plan.is_valid(&closure));
    plan.demands.clear();
    assert!(!plan.is_valid(&closure));
}
