use super::super::body;
use super::super::function_name;
use super::super::syntax::{FunctionBuilder, FunctionDefinition, Instruction, Terminator};
use crate::backend::abi::Function as AbiFunction;

pub(super) fn definition(
    body: &body::Output,
    types: &body::types::Types,
) -> Option<FunctionDefinition> {
    let mut function =
        FunctionBuilder::new(AbiFunction::program_entry().llvm_definition_signature());
    function.start_block("entry").then_some(())?;
    let control_top = if body.uses_control {
        function
            .structured_instruction(Instruction::alloca(
                "%mal_control_top",
                types.index_llvm_type(),
                types.index_alignment(),
            )?)
            .then_some(())?;
        function
            .instruction(format!(
                "store {} 0, ptr %mal_control_top, align {}",
                types.index_integer(),
                types.index_alignment()
            ))
            .then_some(())?;
        function
            .instruction("call void @mal_native_stack_begin(ptr %mal_context)")
            .then_some(())?;
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
            function
                .instruction(format!(
                    "%mal_entry_argument = load {}, ptr %mal_argument, align {}",
                    value.llvm, value.alignment
                ))
                .then_some(())?;
            format!(
                "call i32 @{}(ptr %mal_context, ptr {control_top}, ptr null, {} %mal_entry_argument)",
                function_name(body.main),
                value.llvm
            )
        }
    };
    function
        .instruction(format!("%mal_entry_result = {call}"))
        .then_some(())?;
    function
        .instruction("store i32 %mal_entry_result, ptr %mal_result, align 4")
        .then_some(())?;
    function
        .terminate(Terminator::return_void())
        .then_some(())?;
    function.finish()
}
