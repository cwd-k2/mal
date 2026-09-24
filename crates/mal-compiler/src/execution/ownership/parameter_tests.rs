use super::*;
use mal_frontend::check::ast::Type;
use mal_syntax::source::{FileId, SourceFile};

#[test]
fn keeps_region_parameters_owned_when_the_region_creates_managed_authority() {
    let source = SourceFile::new(
        FileId::new(104),
        "execution-owned-region-parameter.mal",
        "walk :: (Symbol, Int32) -> Int32 := (text, remaining) -> { next := text + \"x\"; if (remaining == 0i32) then { (#next).i32 } else { walk((next, remaining - 1i32)) }; }; main :: Unit -> Int32 := () -> { walk((\"x\", 4i32)); };"
            .into(),
    );
    let checked = mal_frontend::analysis::check(&source).expect("check owned region fixture");
    let core = crate::core::lower(
        &mal_frontend::check::specialize(checked).expect("specialize owned region fixture"),
    );
    let anf = crate::anf::lower(&core);
    let closure = crate::closure::convert(&anf);
    let execution = crate::execution::lower(closure, super::super::OptimizationSet::production());
    let function = execution
        .control
        .functions
        .iter()
        .find(|function| matches!(function.parameter.ty, Type::Product(_)))
        .expect("recursive product function");
    let binding = function.parameter.binding.expect("bound region parameter");

    assert_eq!(
        execution
            .ownership
            .parameter_effect(function.id, ParameterEntry::BorrowedAbi),
        Some(ParameterEffect::ShareInto(binding))
    );
    assert_eq!(
        execution
            .ownership
            .parameter_effect(function.id, ParameterEntry::OwnedHandoff),
        Some(ParameterEffect::ConsumeInto(binding))
    );
    assert!(!execution.ownership.binding_is_borrowed(binding));
}
