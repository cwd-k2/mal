//! Procedural entry macros for typed LLVM syntax nodes.

pub(in crate::backend) use mal_backend_macros::{
    emit_instruction, emit_terminator, llvm_constant, llvm_declaration, llvm_function_attributes,
    llvm_global, llvm_instruction, llvm_metadata, llvm_parameter, llvm_parameters, llvm_signature,
    llvm_terminator, llvm_type, llvm_typed_constant,
};

#[cfg(test)]
pub(in crate::backend) use mal_backend_macros::llvm_metadata_operand;

#[cfg(test)]
mod tests;
