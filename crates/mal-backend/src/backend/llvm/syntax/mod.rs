mod constant;
mod function;
mod instruction;
mod macros;
mod module;
mod ty;

pub(in crate::backend) use macros::{
    llvm_constant, llvm_constant_child, llvm_declaration, llvm_declaration_build,
    llvm_declaration_result, llvm_function_attributes, llvm_function_attributes_items, llvm_global,
    llvm_instruction, llvm_instruction_atom, llvm_instruction_callee, llvm_instruction_indices,
    llvm_instruction_pairs, llvm_instruction_sequence, llvm_instruction_type, llvm_metadata,
    llvm_metadata_operand, llvm_metadata_operands, llvm_metadata_operands_items, llvm_parameter,
    llvm_parameters, llvm_parameters_items, llvm_scalar, llvm_signature, llvm_signature_build,
    llvm_signature_result, llvm_switch_cases, llvm_switch_cases_item, llvm_terminator, llvm_type,
    llvm_typed_constant, llvm_typed_constant_child, llvm_typed_constants,
    llvm_typed_constants_item, llvm_types, llvm_types_items, llvm_value, llvm_values,
    llvm_values_item, llvm_values_items,
};

pub(super) use constant::{Constant, TypedConstant};
#[cfg(test)]
pub(super) use function::BasicBlock;
pub(in crate::backend) use function::{
    FunctionAttribute, FunctionSignature, Linkage, Parameter, ParameterAttribute,
};
pub(super) use function::{FunctionBuilder, FunctionDefinition, Terminator};
pub(super) use instruction::MetadataAttachment;
pub(super) use instruction::{
    BinaryOperator, CastOperator, ComparisonKind, ComparisonPredicate, UnaryOperator,
};
pub(in crate::backend) use instruction::{Callee, Instruction, TypedValue};
pub(in crate::backend) use module::FunctionDeclaration;
pub(super) use module::Module;
pub(super) use module::{GlobalDefinition, MetadataDefinition, MetadataOperand};
pub(in crate::backend) use ty::Type;
