mod constant;
mod function;
mod instruction;
mod macros;
mod module;
mod ty;

pub(super) use macros::{
    llvm_constant, llvm_constant_child, llvm_function_attributes, llvm_function_attributes_items,
    llvm_instruction, llvm_parameter, llvm_parameter_attributes, llvm_parameters,
    llvm_parameters_items, llvm_signature, llvm_signature_build, llvm_switch_cases,
    llvm_switch_cases_item, llvm_terminator, llvm_type, llvm_typed_constant,
    llvm_typed_constant_child, llvm_typed_constants, llvm_typed_constants_item, llvm_types,
    llvm_types_items, llvm_value, llvm_values, llvm_values_item,
};

pub(super) use constant::{Constant, TypedConstant};
#[cfg(test)]
pub(super) use function::BasicBlock;
pub(super) use function::{
    FunctionAttribute, FunctionBuilder, FunctionDefinition, Linkage, ParameterAttribute, Terminator,
};
pub(in crate::backend) use function::{FunctionSignature, Parameter};
pub(super) use instruction::MetadataAttachment;
pub(super) use instruction::{
    BinaryOperator, CastOperator, ComparisonKind, ComparisonPredicate, UnaryOperator,
};
pub(in crate::backend) use instruction::{Callee, Instruction, TypedValue};
pub(in crate::backend) use module::FunctionDeclaration;
pub(super) use module::Module;
pub(super) use module::{GlobalDefinition, MetadataDefinition, MetadataOperand};
pub(in crate::backend) use ty::Type;
