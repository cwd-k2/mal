mod constant;
mod declaration;
mod instruction;
mod terminator;
mod value;

pub(in crate::backend::llvm) use constant::{
    llvm_constant, llvm_typed_constant, llvm_typed_constants, llvm_typed_constants_item,
};
pub(in crate::backend::llvm) use declaration::{
    llvm_function_attributes, llvm_function_attributes_items, llvm_parameter,
    llvm_parameter_attributes, llvm_parameters, llvm_parameters_items, llvm_signature,
    llvm_signature_build, llvm_type, llvm_types, llvm_types_items,
};
pub(in crate::backend::llvm) use instruction::llvm_instruction;
pub(in crate::backend::llvm) use terminator::{
    llvm_switch_cases, llvm_switch_cases_item, llvm_terminator,
};
pub(in crate::backend::llvm) use value::{llvm_value, llvm_values, llvm_values_item};

#[cfg(test)]
mod tests;
