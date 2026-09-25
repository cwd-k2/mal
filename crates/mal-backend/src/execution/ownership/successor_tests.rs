use super::*;
use crate::control::ast::{Operation, StateId};
use mal_frontend::check::ast::Type;
use mal_syntax::source::{FileId, SourceFile};

fn lower(source: &str) -> crate::execution::Program {
    let source = SourceFile::new(FileId::new(87), "owner-successors.mal", source.into());
    let checked = mal_frontend::analysis::check(&source).expect("check owner successor fixture");
    let core = crate::core::lower(
        &mal_frontend::check::specialize(checked).expect("specialize owner successor fixture"),
    );
    let anf = crate::anf::lower(&core);
    let closure = crate::closure::convert(&anf);
    crate::execution::lower(closure, super::super::OptimizationSet::production())
}

#[test]
fn shares_borrowed_parameter_leaves_when_returning_a_product() {
    let execution = lower(
        "rebuild :: (Symbol, Symbol) -> (Symbol, Symbol) := ((left, right)) -> { (left, right) };\nmain :: Unit -> Int32 := () -> { result := rebuild((\"a\" + \"b\", \"c\" + \"d\")); 0i32; };",
    );
    let (site, binding) = execution
        .control
        .states
        .iter()
        .enumerate()
        .find_map(|(state, value)| {
            value
                .bindings
                .iter()
                .enumerate()
                .find(|(_, binding)| {
                    matches!(
                        &binding.operation,
                        Operation::Product(elements)
                            if elements.len() == 2
                                && elements.iter().all(|element| element.ty == Type::Symbol)
                    )
                })
                .map(|(binding, _)| (StateId(state), binding))
        })
        .expect("Symbol product reconstruction");
    assert_eq!(
        execution
            .ownership
            .binding_use(site, binding, BindingOperand::ProductElement(0)),
        Some(UseEffect::Share)
    );
    assert_eq!(
        execution
            .ownership
            .binding_use(site, binding, BindingOperand::ProductElement(1)),
        Some(UseEffect::Share)
    );
}

#[test]
fn shares_a_borrowed_leaf_only_when_a_closure_environment_escapes() {
    let execution = lower(
        "create :: (Symbol, Symbol) -> (Unit -> Symbol) := (pair) -> { (left, _) := pair; closure :: Unit -> Symbol := () -> { left }; (againLeft, _) := pair; length := #againLeft; closure; };\nmain :: Unit -> Int32 := () -> { closure := create((\"a\" + \"b\", \"c\" + \"d\")); result := closure(); 0i32; };",
    );
    let (site, binding) = execution
        .control
        .states
        .iter()
        .enumerate()
        .find_map(|(state, value)| {
            value
                .bindings
                .iter()
                .enumerate()
                .find(|(_, binding)| {
                    matches!(
                        &binding.operation,
                        Operation::MakeClosure { captures, .. }
                            if matches!(captures.as_slice(), [capture] if capture.ty == Type::Symbol)
                    )
                })
                .map(|(binding, _)| (StateId(state), binding))
        })
        .expect("capturing closure construction");
    assert_eq!(
        execution
            .ownership
            .binding_use(site, binding, BindingOperand::Capture(0)),
        Some(UseEffect::Share)
    );
}

#[test]
fn owns_a_join_input_when_a_jumping_value_is_an_owner() {
    let execution = lower(
        "len :: Buffer<UInt8> -> Int32 := (b) -> (#b).i32;\nmain :: Unit -> Int32 := () -> { b := if (true) then make<UInt8>(4usize) else make<UInt8>(2usize); len(b) };",
    );
    let jumps = execution
        .control
        .states
        .iter()
        .enumerate()
        .filter_map(|(index, state)| match &state.terminator {
            crate::control::ast::Terminator::Jump { target, value }
                if value.ty == Type::Buffer(Box::new(Type::UInt8).into()) =>
            {
                Some((StateId(index), *target))
            }
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(jumps.len(), 2);
    for (site, target) in jumps {
        assert!(matches!(
            execution.ownership.input_destination(target),
            Some(PatternDestination::Initialize(_))
        ));
        assert_eq!(
            execution
                .ownership
                .terminator_use(site, TerminatorOperand::JumpValue),
            Some(UseEffect::Consume)
        );
    }
}
