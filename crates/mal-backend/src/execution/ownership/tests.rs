use std::collections::HashSet;

use super::super::ParameterDestination;
use super::destination::plan_borrowed_pattern;
use super::liveness::binding_id;
use super::use_plan::jump_value_effect;
use super::*;
use crate::closure::ast::Pattern;
use crate::control::ast::{Operation, StateId, Terminator};
use mal_frontend::check::ast::Type;
use mal_syntax::source::{FileId, SourceFile};

#[test]
fn validates_the_exact_binding_drop_facts() {
    let source = SourceFile::new(
        FileId::new(90),
        "execution-ownership-plan.mal",
        "main :: Unit -> Int32 := () -> { value := \"a\" + \"b\"; length := #value; length.i32; };"
            .into(),
    );
    let checked = mal_frontend::analysis::check(&source).expect("check ownership fixture");
    let core = crate::core::lower(
        &mal_frontend::check::specialize(checked).expect("specialize ownership fixture"),
    );
    let anf = crate::anf::lower(&core);
    let closure = crate::closure::convert(&anf);
    let mut execution =
        crate::execution::lower(closure, super::super::OptimizationSet::production());
    let optimizations = super::super::OptimizationPlan::new(
        &execution.lowered,
        &execution.control,
        &execution.applications,
        super::super::OptimizationSet::production(),
    );

    assert!(execution.ownership.is_valid(Inputs::new(
        &execution.control,
        &execution.applications,
        &optimizations,
        &execution.parameters,
        &execution.control_calls,
        &execution.control_regions,
        &execution.control_frames,
    )));
    let point = *execution
        .ownership
        .drops_after_binding
        .keys()
        .next()
        .expect("dead managed value fact");
    execution.ownership.drops_after_binding.remove(&point);
    assert!(!execution.ownership.is_valid(Inputs::new(
        &execution.control,
        &execution.applications,
        &optimizations,
        &execution.parameters,
        &execution.control_calls,
        &execution.control_regions,
        &execution.control_frames,
    )));
}

#[test]
fn shares_duplicate_owner_successors_before_consuming_the_source() {
    let source = SourceFile::new(
            FileId::new(91),
            "execution-ownership-duplicate.mal",
            "duplicate :: Unit -> (Symbol, Symbol) := () -> { value := \"a\" + \"b\"; (value, value); };\nmain :: Unit -> Int32 := () -> { pair := duplicate(); 0i32; };"
                .into(),
        );
    let checked =
        mal_frontend::analysis::check(&source).expect("check duplicate ownership fixture");
    let core = crate::core::lower(
        &mal_frontend::check::specialize(checked).expect("specialize duplicate ownership fixture"),
    );
    let anf = crate::anf::lower(&core);
    let closure = crate::closure::convert(&anf);
    let execution = crate::execution::lower(closure, super::super::OptimizationSet::production());
    let control = &execution.control;
    let plan = &execution.ownership;

    let (site, binding) = control
            .states
            .iter()
            .enumerate()
            .find_map(|(state_index, state)| {
                state
                    .bindings
                    .iter()
                    .enumerate()
                    .find_map(|(binding_index, binding)| {
                        matches!(&binding.operation, Operation::Product(elements) if elements.len() == 2 && binding_id(&elements[0]) == binding_id(&elements[1]))
                            .then_some((StateId(state_index), binding_index))
                    })
            })
            .expect("duplicate product binding");
    assert_eq!(
        plan.binding_use(site, binding, BindingOperand::ProductElement(0)),
        Some(UseEffect::Share)
    );
    assert_eq!(
        plan.binding_use(site, binding, BindingOperand::ProductElement(1)),
        Some(UseEffect::Consume)
    );
    let Operation::Product(elements) = &control.states[site.0].bindings[binding].operation else {
        unreachable!();
    };
    let source = binding_id(&elements[0]).expect("local duplicate source");
    assert!(!plan.drops_after_binding(site, binding).contains(&source));
}

#[test]
fn drops_an_unused_managed_binding_immediately() {
    let source = SourceFile::new(
        FileId::new(95),
        "execution-ownership-unused-result.mal",
        "main :: Unit -> Int32 := () -> { unused := \"a\" + \"b\"; 0i32; };".into(),
    );
    let checked = mal_frontend::analysis::check(&source).expect("check unused result fixture");
    let core = crate::core::lower(
        &mal_frontend::check::specialize(checked).expect("specialize unused result fixture"),
    );
    let anf = crate::anf::lower(&core);
    let closure = crate::closure::convert(&anf);
    let execution = crate::execution::lower(closure, super::super::OptimizationSet::production());

    let (site, binding) = execution
        .control
        .states
        .iter()
        .enumerate()
        .find_map(|(state_index, state)| {
            state
                .bindings
                .iter()
                .enumerate()
                .find_map(|(binding_index, binding)| match binding.pattern {
                    Pattern::Binding { ref ty, .. }
                        if is_managed(ty) && matches!(binding.operation, Operation::Atom(_)) =>
                    {
                        Some((StateId(state_index), binding_index))
                    }
                    _ => None,
                })
        })
        .expect("unused managed binding");
    assert_eq!(
        execution.ownership.binding_destination(site, binding),
        Some(&PatternDestination::Discard)
    );
    let Operation::Atom(atom) = &execution.control.states[site.0].bindings[binding].operation
    else {
        unreachable!();
    };
    assert_eq!(
        execution
            .ownership
            .binding_use(site, binding, BindingOperand::Atom),
        Some(UseEffect::Borrow)
    );
    if let Some(source) = binding_id(atom) {
        assert!(
            execution
                .ownership
                .drops_after_binding(site, binding)
                .contains(&source)
        );
    }
}

#[test]
fn distinguishes_borrowed_and_owned_parameter_entries() {
    let source = SourceFile::new(
            FileId::new(96),
            "execution-ownership-parameters.mal",
            "keep :: Symbol -> Symbol := (value) -> { value; };\nignore :: Symbol -> Int32 := (value) -> { 0i32; };\ndiscard :: Symbol -> Int32 := (_) -> { 0i32; };\nmain :: Unit -> Int32 := () -> { discard(keep(\"x\")) + ignore(\"y\"); };".into(),
        );
    let checked =
        mal_frontend::analysis::check(&source).expect("check parameter ownership fixture");
    let core = crate::core::lower(
        &mal_frontend::check::specialize(checked).expect("specialize parameter ownership fixture"),
    );
    let anf = crate::anf::lower(&core);
    let closure = crate::closure::convert(&anf);
    let execution = crate::execution::lower(closure, super::super::OptimizationSet::production());

    for function in execution
        .control
        .functions
        .iter()
        .filter(|function| is_managed(&function.parameter.ty))
    {
        match execution
            .parameters
            .destination(function.id)
            .expect("parameter destination")
        {
            ParameterDestination::Bind(binding)
                if execution.control.states[function.entry.0]
                    .live
                    .iter()
                    .any(|value| value.id == binding) =>
            {
                let borrowed = execution.control.states[function.entry.0].input.is_none();
                assert_eq!(
                    execution
                        .ownership
                        .parameter_effect(function.id, ParameterEntry::BorrowedAbi),
                    Some(if borrowed {
                        ParameterEffect::BorrowInto(binding)
                    } else {
                        ParameterEffect::ShareInto(binding)
                    })
                );
                assert_eq!(
                    execution
                        .ownership
                        .parameter_effect(function.id, ParameterEntry::OwnedHandoff),
                    Some(if borrowed {
                        ParameterEffect::BorrowInto(binding)
                    } else {
                        ParameterEffect::ConsumeInto(binding)
                    })
                );
            }
            ParameterDestination::Bind(_) | ParameterDestination::Discard => {
                assert_eq!(
                    execution
                        .ownership
                        .parameter_effect(function.id, ParameterEntry::BorrowedAbi),
                    None
                );
                assert_eq!(
                    execution
                        .ownership
                        .parameter_effect(function.id, ParameterEntry::OwnedHandoff),
                    execution.control.states[function.entry.0]
                        .input
                        .is_some()
                        .then_some(ParameterEffect::Drop)
                );
            }
        }
    }
}

#[test]
fn drops_branch_local_only_on_the_path_that_does_not_need_it() {
    let source = SourceFile::new(
            FileId::new(97),
            "execution-ownership-branch-edge.mal",
            "lengthOnOnePath :: Symbol -> USize := (value) -> { owned := value + \"x\"; if (#owned == 0usize) then { #owned } else { 0usize }; };\nmain :: Unit -> Int32 := () -> { lengthOnOnePath(\"x\").i32; };".into(),
        );
    let checked = mal_frontend::analysis::check(&source).expect("check branch edge fixture");
    let core = crate::core::lower(
        &mal_frontend::check::specialize(checked).expect("specialize branch edge fixture"),
    );
    let anf = crate::anf::lower(&core);
    let closure = crate::closure::convert(&anf);
    let execution = crate::execution::lower(closure, super::super::OptimizationSet::production());
    let managed_bindings = execution
        .control
        .states
        .iter()
        .flat_map(|state| state.bindings.iter())
        .filter_map(|binding| match &binding.pattern {
            Pattern::Binding { id, ty } if is_managed(ty) => Some(*id),
            _ => None,
        })
        .collect::<Vec<_>>();
    let differs_by_path = execution
        .control
        .states
        .iter()
        .enumerate()
        .find_map(|(state, value)| {
            let site = StateId(state);
            matches!(value.terminator, Terminator::PrimitiveBranch { .. }).then(|| {
                managed_bindings.iter().any(|binding| {
                    let otherwise = execution
                        .ownership
                        .drops_on_edge(site, ControlPath::BranchOtherwise)
                        .contains(binding);
                    let then = execution
                        .ownership
                        .drops_on_edge(site, ControlPath::BranchThen)
                        .contains(binding);
                    otherwise != then
                })
            })
        })
        .unwrap_or(false);
    assert!(differs_by_path);
}

#[test]
fn classifies_an_unused_managed_state_input_as_a_drop() {
    let id = ValueId::Temporary(17);
    let pattern = Pattern::Binding {
        id,
        ty: Type::Symbol,
    };
    assert_eq!(
        plan_borrowed_pattern(&pattern, &HashSet::new(), &HashSet::new()),
        PatternDestination::Discard
    );
    assert_eq!(
        jump_value_effect(&PatternDestination::Discard),
        UseEffect::Borrow
    );
    assert_eq!(
        jump_value_effect(&PatternDestination::Initialize(id)),
        UseEffect::Share
    );
}

#[test]
fn consumes_a_case_payload_when_the_sum_owner_ends_at_the_arm() {
    let source = SourceFile::new(
            FileId::new(99),
            "execution-ownership-case-payload.mal",
            "Choice :: [Symbol, Symbol];\nselect :: Symbol -> Symbol := (value) -> { owned := value + \"x\"; choice :: Choice := [first, second] => { first(owned) }; choice[(payload) -> { payload }, (payload) -> { payload }] };\nmain :: Unit -> Int32 := () -> { (#select(\"y\")).i32; };".into(),
        );
    let checked = mal_frontend::analysis::check(&source).expect("check case payload fixture");
    let core = crate::core::lower(
        &mal_frontend::check::specialize(checked).expect("specialize case payload fixture"),
    );
    let anf = crate::anf::lower(&core);
    let closure = crate::closure::convert(&anf);
    let execution = crate::execution::lower(closure, super::super::OptimizationSet::production());
    let (site, arm_count) = execution
        .control
        .states
        .iter()
        .enumerate()
        .find_map(|(state, value)| match &value.terminator {
            Terminator::Case { arms, .. } if arms.len() == 2 => Some((StateId(state), arms.len())),
            _ => None,
        })
        .expect("managed case");
    for arm in 0..arm_count {
        assert_eq!(
            execution.ownership.case_payload_use(site, arm),
            Some(UseEffect::Consume)
        );
    }
}

#[test]
fn borrows_a_final_observation_before_dropping_its_source() {
    let source = SourceFile::new(
            FileId::new(92),
            "execution-ownership-observation.mal",
            "observe :: Unit -> USize := () -> { value := \"a\" + \"b\"; #value; };\nmain :: Unit -> Int32 := () -> { observe().i32; };"
                .into(),
        );
    let checked = mal_frontend::analysis::check(&source).expect("check observation fixture");
    let core = crate::core::lower(
        &mal_frontend::check::specialize(checked).expect("specialize observation fixture"),
    );
    let anf = crate::anf::lower(&core);
    let closure = crate::closure::convert(&anf);
    let execution = crate::execution::lower(closure, super::super::OptimizationSet::production());
    let control = &execution.control;
    let plan = &execution.ownership;

    let (site, binding) = control
        .states
        .iter()
        .enumerate()
        .find_map(|(state_index, state)| {
            state
                .bindings
                .iter()
                .position(|binding| matches!(binding.operation, Operation::SymbolLength { .. }))
                .map(|binding| (StateId(state_index), binding))
        })
        .expect("Symbol length binding");
    assert_eq!(
        plan.binding_use(site, binding, BindingOperand::SymbolLength),
        Some(UseEffect::Borrow)
    );
    assert!(!plan.drops_after_binding(site, binding).is_empty());
}

#[test]
fn shares_a_frame_field_before_consuming_the_same_next_argument() {
    let source = SourceFile::new(
            FileId::new(94),
            "execution-ownership-frame-argument.mal",
            "extern choose :: Unit -> Bool;\nwalk :: Symbol -> Symbol := (value) -> {\n  if (choose()) then { value } else {\n    child := walk(value);\n    if (#value == 0usize) then { child } else { child };\n  };\n};\nmain :: Unit -> Int32 := () -> { result := walk(\"x\"); (#result).i32; };"
                .into(),
        );
    let checked = mal_frontend::analysis::check(&source).expect("check frame ownership fixture");
    let core = crate::core::lower(
        &mal_frontend::check::specialize(checked).expect("specialize frame ownership fixture"),
    );
    let anf = crate::anf::lower(&core);
    let closure = crate::closure::convert(&anf);
    let execution = crate::execution::lower(closure, super::super::OptimizationSet::production());
    let (site, field_index) = execution
        .control
        .states
        .iter()
        .enumerate()
        .find_map(|(state_index, state)| {
            let site = StateId(state_index);
            let frame = execution.control_frames.frame(site)?;
            let Terminator::Call { argument, .. } = &state.terminator else {
                return None;
            };
            let argument_id = binding_id(argument)?;
            frame
                .fields
                .iter()
                .position(|field| field.id == argument_id)
                .map(|field| (site, field))
        })
        .expect("frame field and argument share a source");

    assert_eq!(
        execution.ownership.frame_field_use(site, field_index),
        Some(UseEffect::Share)
    );
    assert_eq!(
        execution
            .ownership
            .terminator_use(site, TerminatorOperand::CallArgument),
        Some(UseEffect::Consume)
    );
}
