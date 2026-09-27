mod constant;
mod declaration;
mod instruction;
mod module;
mod terminator;
mod value;

pub(in crate::backend) use constant::{
    llvm_constant, llvm_typed_constant, llvm_typed_constants, llvm_typed_constants_item,
    llvm_typed_constants_items,
};
pub(in crate::backend) use declaration::{
    llvm_function_attributes, llvm_function_attributes_items, llvm_parameter, llvm_parameters,
    llvm_parameters_items, llvm_scalar, llvm_signature, llvm_signature_build,
    llvm_signature_result, llvm_type, llvm_types, llvm_types_items,
};
pub(in crate::backend) use instruction::{
    llvm_instruction, llvm_instruction_atom, llvm_instruction_callee, llvm_instruction_indices,
    llvm_instruction_sequence, llvm_instruction_type,
};
pub(in crate::backend) use module::{
    llvm_declaration, llvm_declaration_build, llvm_declaration_result, llvm_global, llvm_metadata,
    llvm_metadata_operand, llvm_metadata_operands, llvm_metadata_operands_items,
};
pub(in crate::backend) use terminator::{
    llvm_switch_cases, llvm_switch_cases_item, llvm_switch_cases_items, llvm_terminator,
};
pub(in crate::backend) use value::{llvm_value, llvm_values, llvm_values_item, llvm_values_items};

#[cfg(test)]
mod tests;
