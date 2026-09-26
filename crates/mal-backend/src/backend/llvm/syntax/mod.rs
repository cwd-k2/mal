mod function;
mod instruction;
mod module;
mod ty;

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
