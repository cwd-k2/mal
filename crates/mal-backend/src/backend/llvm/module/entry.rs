use super::super::body;
use super::super::function_name;
use super::super::syntax::{BasicBlock, FunctionDefinition};
use crate::backend::abi::Function as AbiFunction;

pub(super) fn definition(
    body: &body::Output,
    types: &body::types::Types,
) -> Option<FunctionDefinition> {
    let mut instructions = Vec::new();
    let control_top = if body.uses_control {
        instructions.extend([
            format!(
                "%mal_control_top = alloca {}, align {}",
                types.index_integer(),
                types.index_alignment()
            ),
            format!(
                "store {} 0, ptr %mal_control_top, align {}",
                types.index_integer(),
                types.index_alignment()
            ),
            "call void @mal_native_stack_begin(ptr %mal_context)".into(),
        ]);
        "%mal_control_top"
    } else {
        "null"
    };
    let call = match &body.main_parameter {
        mal_frontend::check::ast::Type::Unit => format!(
            "call i32 @{}(ptr %mal_context, ptr {control_top}, ptr null)",
            function_name(body.main)
        ),
        ty => {
            let value = types.value(ty)?;
            instructions.push(format!(
                "%mal_entry_argument = load {}, ptr %mal_argument, align {}",
                value.llvm, value.alignment
            ));
            format!(
                "call i32 @{}(ptr %mal_context, ptr {control_top}, ptr null, {} %mal_entry_argument)",
                function_name(body.main),
                value.llvm
            )
        }
    };
    instructions.extend([
        format!("%mal_entry_result = {call}"),
        "store i32 %mal_entry_result, ptr %mal_result, align 4".into(),
        "ret void".into(),
    ]);
    FunctionDefinition::new(
        AbiFunction::program_entry().llvm_definition_signature(),
        vec![BasicBlock::new("entry", instructions)?],
    )
}
