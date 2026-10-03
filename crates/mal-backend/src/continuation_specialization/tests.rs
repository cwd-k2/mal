use mal_syntax::source::{FileId, SourceFile};

use super::Plan;
use super::plan::ProducerResult;
use crate::closure::ast::AtomKind;

fn analyze(text: &str) -> (crate::closure::ast::Program, Plan) {
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
    let program = crate::call_pattern::specialize(closure);
    let plan = Plan::new(&program);
    (program, plan)
}

fn plan(text: &str) -> Plan {
    analyze(text).1
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
    assert!(matches!(
        &plan.steps[0].result,
        ProducerResult::Closure { .. }
    ));
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
            .any(|step| matches!(&step.result, ProducerResult::Call { .. }))
    );
    assert!(
        plan.steps
            .iter()
            .any(|step| matches!(&step.result, ProducerResult::Closure { .. }))
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
            .filter(|step| matches!(&step.result, ProducerResult::Call { .. }))
            .count(),
        2
    );
    assert_eq!(
        plan.steps
            .iter()
            .filter(|step| matches!(&step.result, ProducerResult::Closure { .. }))
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
fn inventories_every_creator_and_call_site_for_slice_code() {
    let plan = plan(
        "create :: Int32 -> (Int32 -> Int32) := (captured) -> (value) -> captured + value;
         main :: Unit -> Int32 := () -> {
             first := create(39i32);
             left := first(1i32);
             second := create(40i32);
             left + second(2i32) - 82i32;
         };",
    );

    assert_eq!(plan.demands.len(), 2);
    assert_eq!(plan.creators.len(), 2, "{:#?}", plan.creators);
    assert_eq!(plan.call_sites.len(), 4, "{:#?}", plan.call_sites);
    assert!(plan.closed);
    assert!(plan.creators_complete);
}

#[test]
fn reports_a_slice_with_an_uncovered_call_site_as_open() {
    let plan = plan(
        "create :: Int32 -> (Int32 -> Int32) := (captured) -> (value) -> captured + value;
         main :: Unit -> Int32 := () -> {
             retained := create(1i32);
             kept := (retained, 0i32);
             (_, zero) := kept;
             create(40i32)(2i32) - 42i32 + zero;
         };",
    );

    assert_eq!(plan.demands.len(), 1);
    assert!(!plan.closed, "{plan:#?}");
}

#[test]
fn admits_a_closure_origin_packed_only_for_local_transport() {
    let (program, plan) = analyze(
        "increment :: Int32 -> Int32 := (value) -> value + 1i32;
         create :: (Int32 -> Int32) -> (Int32 -> Int32) := (callback) -> (value) -> {
             kept := (callback, value);
             (selected, argument) := kept;
             selected(argument);
         };
         main :: Unit -> Int32 := () -> create(increment)(41i32) - 42i32;",
    );
    assert_eq!(plan.demands.len(), 1);
    assert!(plan.closed, "{plan:#?}");
    assert!(plan.transport_closed, "{plan:#?}");
    assert!(plan.request(&program).is_some());
}

#[test]
fn reports_capture_by_a_closure_outside_the_slice_as_open_transport() {
    let (program, plan) = analyze(
        "increment :: Int32 -> Int32 := (value) -> value + 1i32;
         create :: (Int32 -> Int32) -> (Int32 -> Int32) := (callback) -> (value) -> {
             unused :: Unit -> (Int32 -> Int32) := () -> callback;
             callback(value);
         };
         main :: Unit -> Int32 := () -> create(increment)(41i32) - 42i32;",
    );

    assert_eq!(plan.demands.len(), 1);
    assert!(plan.closed, "{plan:#?}");
    assert!(!plan.transport_closed, "{plan:#?}");
    assert!(plan.request(&program).is_none());
}

#[test]
fn finds_the_final_consumer_of_a_recursive_state_chain() {
    let (program, plan) = analyze(
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
    assert!(
        plan.result_applications.iter().any(|application| {
            application.producer_site.0 == 97 && application.consumer.0 == 100
        }),
        "{plan:#?}"
    );
    assert_eq!(plan.steps.len(), 4, "{:#?}", plan.steps);
    assert_eq!(
        plan.steps
            .iter()
            .filter(|step| matches!(&step.result, ProducerResult::Call { .. }))
            .count(),
        2
    );
    assert!(!plan.applications.is_empty(), "{:#?}", plan.applications);
    assert!(!plan.creators.is_empty(), "{:#?}", plan.creators);
    assert!(!plan.call_sites.is_empty(), "{:#?}", plan.call_sites);
    assert!(plan.closed, "{plan:#?}");
    assert!(plan.creators_complete, "{plan:#?}");
    assert!(plan.transport_closed, "{plan:#?}");
    let mut request = plan
        .request(&program)
        .expect("closed State rewrite request");
    assert!(request.is_valid(&program, &plan));
    assert!(!request.workers.is_empty());
    assert!(
        request
            .workers
            .iter()
            .all(|worker| worker.original != worker.worker)
    );
    let copied = request.apply(&program).expect("fused State worker");
    assert_eq!(copied.functions.len(), program.functions.len() + 1);
    assert!(copied.functions.iter().any(|function| {
        request.workers.iter().any(|worker| {
            worker.original == request.demand.producer && function.id == worker.worker
        })
    }));
    assert!(crate::closure::rewrite::are_unique(&mut copied.clone()));
    request.workers.clear();
    assert!(!request.is_valid(&program, &plan));
}

#[test]
fn validates_the_exact_demand_set() {
    let source = SourceFile::new(
        FileId::new(100),
        "continuation-specialization-validation.mal",
        "increment :: Int32 -> Int32 := (value) -> value + 1i32;
         create :: (Int32 -> Int32) -> (Int32 -> Int32) := (callback) -> (value) -> callback(value);
         main :: Unit -> Int32 := () -> create(increment)(41i32) - 42i32;"
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
    plan = Plan::new(&closure);
    plan.result_applications.clear();
    assert!(!plan.is_valid(&closure));
    plan = Plan::new(&closure);
    plan.steps.clear();
    assert!(!plan.is_valid(&closure));
    plan = Plan::new(&closure);
    assert!(!plan.applications.is_empty());
    plan.applications.clear();
    assert!(!plan.is_valid(&closure));
    plan = Plan::new(&closure);
    assert!(!plan.creators.is_empty());
    plan.creators.clear();
    assert!(!plan.is_valid(&closure));
    plan = Plan::new(&closure);
    assert!(!plan.call_sites.is_empty());
    plan.call_sites.clear();
    assert!(!plan.is_valid(&closure));
    plan = Plan::new(&closure);
    plan.call_sites[0].sources.clear();
    assert!(!plan.is_valid(&closure));
    plan = Plan::new(&closure);
    plan.closed = !plan.closed;
    assert!(!plan.is_valid(&closure));
    plan = Plan::new(&closure);
    plan.creators_complete = !plan.creators_complete;
    assert!(!plan.is_valid(&closure));
    plan = Plan::new(&closure);
    plan.uses.clear();
    assert!(!plan.is_valid(&closure));
    plan = Plan::new(&closure);
    plan.transport_closed = !plan.transport_closed;
    assert!(!plan.is_valid(&closure));
}
