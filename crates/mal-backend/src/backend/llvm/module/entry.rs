use super::super::body;
use super::super::function_name;
use super::super::syntax::{
    FunctionBuilder, FunctionDefinition, Instruction, Type, llvm_instruction, llvm_terminator,
};
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
            .structured_instruction(llvm_instruction!(alloca
                "%mal_control_top",
                types.index_llvm_type(),
                types.index_alignment(),
            )?)
            .then_some(())?;
        function
            .structured_instruction(llvm_instruction!(store
                types.index_llvm_type(),
                "0",
                "%mal_control_top",
                types.index_alignment(),
                [],
            )?)
            .then_some(())?;
        function
            .structured_instruction(direct_call(
                None,
                Type::Void,
                "mal_native_stack_begin",
                [(Type::Pointer, "%mal_context".into())],
            )?)
            .then_some(())?;
        "%mal_control_top"
    } else {
        "null"
    };
    let mut arguments = vec![
        (Type::Pointer, "%mal_context".into()),
        (Type::Pointer, control_top.into()),
        (Type::Pointer, "null".into()),
    ];
    match &body.main_parameter {
        mal_frontend::check::ast::Type::Unit => {}
        ty => {
            let value = types.value(ty)?;
            function
                .structured_instruction(llvm_instruction!(load
                    "%mal_entry_argument",
                    value.llvm.clone(),
                    "%mal_argument",
                    value.alignment,
                    [],
                )?)
                .then_some(())?;
            arguments.push((value.llvm, "%mal_entry_argument".into()));
        }
    }
    function
        .structured_instruction(direct_call(
            Some("%mal_entry_result".into()),
            Type::integer(32_u16),
            function_name(body.main),
            arguments,
        )?)
        .then_some(())?;
    function
        .structured_instruction(llvm_instruction!(store
            Type::integer(32_u16),
            "%mal_entry_result",
            "%mal_result",
            4,
            [],
        )?)
        .then_some(())?;
    function
        .terminate(llvm_terminator!(return_void)?)
        .then_some(())?;
    function.finish()
}

fn direct_call(
    result: Option<String>,
    result_type: Type,
    callee: impl Into<String>,
    arguments: impl IntoIterator<Item = (Type, String)>,
) -> Option<Instruction> {
    llvm_instruction!(
        call result, false, result_type, direct callee, arguments
    )
}
