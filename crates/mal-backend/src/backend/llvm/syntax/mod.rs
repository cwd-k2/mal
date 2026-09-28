//! Restricted typed LLVM syntax model; rendering is its only conversion to text.

mod constant;
mod function;
mod instruction;
mod macros;
mod module;
mod ty;

pub(in crate::backend) use macros::*;

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
