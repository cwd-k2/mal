use super::super::body;
use super::super::function_name;
use super::super::syntax::{
    FunctionBuilder, FunctionDefinition, llvm_instruction, llvm_terminator, llvm_type,
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
                { types.index_llvm_type() },
                { types.index_alignment() },
            )?)
            .then_some(())?;
        function
            .structured_instruction(llvm_instruction!(store
                { types.index_llvm_type() },
                "0",
                "%mal_control_top",
                { types.index_alignment() },
                [],
            )?)
            .then_some(())?;
        function
            .structured_instruction(llvm_instruction!(
                call None,
                false,
                (void),
                direct "mal_native_stack_begin";
                [
                    (typed (ptr) => "%mal_context"),
                ]
            )?)
            .then_some(())?;
        "%mal_control_top"
    } else {
        "null"
    };
    let mut arguments = vec![
        (llvm_type!(ptr), "%mal_context".into()),
        (llvm_type!(ptr), control_top.into()),
        (llvm_type!(ptr), "null".into()),
    ];
    match &body.main_parameter {
        mal_frontend::check::ast::Type::Unit => {}
        ty => {
            let value = types.value(ty)?;
            function
                .structured_instruction(llvm_instruction!(load
                    "%mal_entry_argument",
                    { value.llvm.clone() },
                    "%mal_argument",
                    { value.alignment },
                    [],
                )?)
                .then_some(())?;
            arguments.push((value.llvm, "%mal_entry_argument".into()));
        }
    }
    function
        .structured_instruction(llvm_instruction!(
            call { Some("%mal_entry_result".into()) },
            false,
            (int(32_u16)),
            direct { function_name(body.main) },
            {{ arguments }}
        )?)
        .then_some(())?;
    function
        .structured_instruction(llvm_instruction!(
            store(int(32_u16)),
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
