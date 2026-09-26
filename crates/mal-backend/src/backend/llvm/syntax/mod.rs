mod function;
mod instruction;
mod module;
mod ty;

#[cfg(test)]
pub(super) use function::BasicBlock;
pub(in crate::backend) use function::FunctionSignature;
pub(super) use function::{FunctionBuilder, FunctionDefinition, Terminator};
pub(super) use instruction::{Callee, Instruction, MetadataAttachment, TypedValue};
pub(in crate::backend) use module::FunctionDeclaration;
pub(super) use module::GlobalDefinition;
pub(super) use module::Module;
pub(in crate::backend::llvm) use ty::Type;
