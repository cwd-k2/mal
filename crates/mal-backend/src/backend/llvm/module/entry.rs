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
            .structured_instruction(llvm_instruction! {
                let "%mal_control_top" = alloca {
                    ty: #{ types.index_llvm_type() },
                    alignment: #{ types.index_alignment() },
                };
            }?)
            .then_some(())?;
        function
            .structured_instruction(llvm_instruction! {
                store {
                    value: typed(#{ types.index_llvm_type() }, "0"),
                    pointer: "%mal_control_top",
                    alignment: #{ types.index_alignment() },
                    metadata: [],
                };
            }?)
            .then_some(())?;
        function
            .structured_instruction(llvm_instruction! {
                call {
                    tail: false,
                    result_type: (void),
                    callee: direct("mal_native_stack_begin"),
                    arguments: [typed((ptr), "%mal_context")],
                };
            }?)
            .then_some(())?;
        "%mal_control_top"
    } else {
        "null"
    };
    let mut arguments = vec![
        (llvm_type!(ptr), "%mal_context".to_owned()),
        (llvm_type!(ptr), control_top.to_owned()),
        (llvm_type!(ptr), "null".to_owned()),
    ];
    match &body.main_parameter {
        mal_frontend::check::ast::Type::Unit => {}
        ty => {
            let value = types.value(ty)?;
            function
                .structured_instruction(llvm_instruction! {
                    let "%mal_entry_argument" = load {
                        ty: #{ value.llvm.clone() },
                        pointer: "%mal_argument",
                        alignment: #{ value.alignment },
                        metadata: [],
                    };
                }?)
                .then_some(())?;
            arguments.push((value.llvm, "%mal_entry_argument".into()));
        }
    }
    let arguments = super::super::syntax::TypedValue::from_pairs(arguments)?;
    function
        .structured_instruction(llvm_instruction! {
            let "%mal_entry_result" = call {
                tail: false,
                result_type: (int(32_u16)),
                callee: direct(#{ function_name(body.main) }),
                arguments: [...#{ arguments }],
            };
        }?)
        .then_some(())?;
    function
        .structured_instruction(llvm_instruction! {
            store {
                value: typed((int(32_u16)), "%mal_entry_result"),
                pointer: "%mal_result",
                alignment: 4,
                metadata: [],
            };
        }?)
        .then_some(())?;
    function
        .terminate(llvm_terminator!(return;)?)
        .then_some(())?;
    function.finish()
}
