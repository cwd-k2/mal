mod function;
mod instruction;
mod module;
mod ty;

pub(in crate::backend) use function::FunctionSignature;
pub(super) use function::{BasicBlock, FunctionBuilder, FunctionDefinition, Terminator};
pub(super) use instruction::Instruction;
pub(in crate::backend) use module::FunctionDeclaration;
pub(super) use module::GlobalDefinition;
pub(super) use module::Module;
pub(in crate::backend::llvm) use ty::Type;
