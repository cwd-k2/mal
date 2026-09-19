use super::*;
use crate::check::ast::Type;
use crate::closure::ast::Pattern;
use crate::control::ast::{Operation, StateId};
use crate::source::{FileId, SourceFile};

fn lower(source: &str) -> crate::execution::Program {
    let source = SourceFile::new(FileId::new(86), "construction-ownership.mal", source.into());
    let checked = crate::pipeline::check(&source).expect("check construction fixture");
    let core = crate::core::lower(
        &crate::check::specialize(checked).expect("specialize construction fixture"),
    );
    let anf = crate::anf::lower(&core);
    let closure = crate::closure::convert(&anf);
    crate::execution::lower(closure, super::super::OptimizationSet::production())
}

#[test]
fn borrows_pure_aggregate_inputs_when_the_result_has_no_owner_successor() {
    let execution = lower(
        "Choice :: [Symbol, Unit];\ndiscard :: (Symbol, Symbol) -> Unit := ((left, right)) -> { _ := (left, right); choice :: Choice := [some, none] => { some(left) }; () };\nmain :: Unit -> Int32 := () -> { discard((\"a\" + \"b\", \"c\" + \"d\")); 0i32; };",
    );
    let mut effects = Vec::new();
    let mut borrowed_results = 0;
    for (state_index, state) in execution.control.states.iter().enumerate() {
        let site = StateId(state_index);
        for (binding_index, binding) in state.bindings.iter().enumerate() {
            let PatternDestination::Borrow(result) = execution
                .ownership
                .binding_destination(site, binding_index)
                .expect("binding destination")
            else {
                continue;
            };
            match &binding.operation {
                Operation::Product(elements) => {
                    borrowed_results += 1;
                    assert!(execution.ownership.binding_is_borrowed(*result));
                    for (index, element) in elements.iter().enumerate() {
                        if element.ty == Type::Symbol {
                            effects.push(execution.ownership.binding_use(
                                site,
                                binding_index,
                                BindingOperand::ProductElement(index),
                            ));
                        }
                    }
                }
                Operation::SumInjection { value, .. } if value.ty == Type::Symbol => {
                    borrowed_results += 1;
                    assert!(execution.ownership.binding_is_borrowed(*result));
                    effects.push(execution.ownership.binding_use(
                        site,
                        binding_index,
                        BindingOperand::SumValue,
                    ));
                }
                _ => {}
            }
        }
    }
    assert_eq!(borrowed_results, 3);
    assert_eq!(effects.len(), 5);
    assert!(
        effects
            .iter()
            .all(|effect| *effect == Some(UseEffect::Borrow))
    );
}

#[test]
fn propagates_borrowed_authority_through_nested_construction() {
    let execution = lower(
        "Problem :: (Symbol, Int64, Int64);\nChoice :: [(Symbol, Symbol), Unit];\nwalk :: (Problem, Int64) -> USize := (problem, remaining) -> { (text, first, second) := problem; if (remaining == 0i64) then { #text } else { (text, first, second).walk(remaining - 1i64) } };\nchoiceLength :: Choice -> USize := (choice) -> { choice[(pair) -> { (left, _) := pair; #left }, () -> { 0usize }] };\nmain :: Unit -> Int32 := () -> { problem :: Problem := (\"a\" + \"b\", 1i64, 2i64); _ := problem.walk(3i64); choice :: Choice := [some, none] => { some((\"c\" + \"d\", \"e\" + \"f\")) }; _ := choiceLength(choice); 0i32; };",
    );
    let construction_results = execution
        .control
        .states
        .iter()
        .flat_map(|state| &state.bindings)
        .filter_map(|binding| {
            matches!(
                binding.operation,
                Operation::Product(_) | Operation::SumInjection { .. }
            )
            .then(|| match binding.pattern {
                Pattern::Binding { id, .. } => Some(id),
                _ => None,
            })
            .flatten()
        })
        .collect::<HashSet<_>>();
    let mut nested_product = false;
    let mut nested_sum = false;
    for (state_index, state) in execution.control.states.iter().enumerate() {
        let site = StateId(state_index);
        for (binding_index, binding) in state.bindings.iter().enumerate() {
            match &binding.operation {
                Operation::Product(elements) => {
                    nested_product |= elements.iter().enumerate().any(|(index, element)| {
                        managed_binding_id(element)
                            .is_some_and(|source| construction_results.contains(&source))
                            && execution.ownership.binding_use(
                                site,
                                binding_index,
                                BindingOperand::ProductElement(index),
                            ) == Some(UseEffect::Borrow)
                    });
                }
                Operation::SumInjection { value, .. } => {
                    nested_sum |= managed_binding_id(value)
                        .is_some_and(|source| construction_results.contains(&source))
                        && execution.ownership.binding_use(
                            site,
                            binding_index,
                            BindingOperand::SumValue,
                        ) == Some(UseEffect::Borrow);
                }
                _ => {}
            }
        }
    }

    assert!(nested_product);
    assert!(nested_sum);
}

#[test]
fn consumes_payloads_into_live_product_and_sum_results() {
    let execution = lower(
        "Choice :: [Symbol, Unit];\npair :: (Symbol, Symbol) -> (Symbol, Symbol) := ((left, right)) -> { ownedLeft := left + \"x\"; ownedRight := right + \"y\"; (ownedLeft, ownedRight) };\nchoice :: Symbol -> Choice := (value) -> { owned := value + \"z\"; [some, none] => { some(owned) } };\nmain :: Unit -> Int32 := () -> { resultPair := pair((\"a\", \"b\")); resultChoice := choice(\"c\"); 0i32; };",
    );
    let mut product_consumes = 0;
    let mut sum_consumes = 0;
    for (state_index, state) in execution.control.states.iter().enumerate() {
        let site = StateId(state_index);
        for (binding_index, binding) in state.bindings.iter().enumerate() {
            match &binding.operation {
                Operation::Product(elements)
                    if elements.iter().all(|element| element.ty == Type::Symbol) =>
                {
                    product_consumes += elements
                        .iter()
                        .enumerate()
                        .filter(|(index, _)| {
                            execution.ownership.binding_use(
                                site,
                                binding_index,
                                BindingOperand::ProductElement(*index),
                            ) == Some(UseEffect::Consume)
                        })
                        .count();
                }
                Operation::SumInjection { value, .. }
                    if value.ty == Type::Symbol
                        && execution.ownership.binding_use(
                            site,
                            binding_index,
                            BindingOperand::SumValue,
                        ) == Some(UseEffect::Consume) =>
                {
                    sum_consumes += 1;
                }
                _ => {}
            }
        }
    }
    assert!(product_consumes >= 2);
    assert_eq!(sum_consumes, 1);
}

#[test]
fn consumes_an_owned_capture_into_a_new_closure_environment() {
    let execution = lower(
        "create :: Symbol -> (Unit -> Symbol) := (value) -> { owned := value + \"x\"; closure :: Unit -> Symbol := () -> { owned }; closure };\nmain :: Unit -> Int32 := () -> { closure := create(\"a\"); result := closure(); 0i32; };",
    );
    let effect = execution
        .control
        .states
        .iter()
        .enumerate()
        .find_map(|(state, value)| {
            value
                .bindings
                .iter()
                .enumerate()
                .find_map(|(binding, value)| {
                    matches!(
                    &value.operation,
                    Operation::MakeClosure { captures, .. }
                        if matches!(captures.as_slice(), [capture] if capture.ty == Type::Symbol)
                )
                .then(|| {
                    execution.ownership.binding_use(
                        StateId(state),
                        binding,
                        BindingOperand::Capture(0),
                    )
                })
                .flatten()
                })
        });
    assert_eq!(effect, Some(UseEffect::Consume));
}
