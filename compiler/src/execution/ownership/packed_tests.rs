use super::{BindingOperand, operand::binding_operands};
use crate::check::ast::Type;
use crate::control::ast::{Operation, StateId};
use crate::source::{FileId, SourceFile};

#[test]
fn identifies_capability_builders_without_capture_inference() {
    let source = SourceFile::new(
        FileId::new(102),
        "execution-ownership-packed-builder.mal",
        "fill :: ((Int64 -> USize), (USize -> Int64), ((USize, Int64) -> Unit)) -> Unit := (new, _, _) -> { new(1i64); (); };\nmain :: Unit -> Int32 := () -> { packed := pack<Int64>(fill); (packed # 0usize).i32; };"
            .into(),
    );
    let checked = crate::pipeline::check(&source).expect("check Packed builder fixture");
    let core = crate::core::lower(
        &crate::check::specialize(checked).expect("specialize Packed builder fixture"),
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
                .position(|binding| {
                    matches!(binding.operation, Operation::MakePackedCapability { .. })
                })
                .map(|binding| (StateId(state_index), binding))
        })
        .expect("Packed capability construction");
    let operation = &execution.control.states[site.0].bindings[binding].operation;
    let operands = binding_operands(operation);
    assert!(matches!(
        operands.as_slice(),
        [(BindingOperand::PackedBuilder, builder, false)] if builder.ty == Type::Address
    ));
    assert_eq!(
        execution
            .ownership
            .binding_use(site, binding, BindingOperand::Capture(0)),
        None
    );
}
