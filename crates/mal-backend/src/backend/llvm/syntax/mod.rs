mod function;
mod module;

pub(in crate::backend) use function::FunctionSignature;
pub(super) use function::{BasicBlock, FunctionBuilder, FunctionDefinition};
pub(in crate::backend) use module::FunctionDeclaration;
pub(super) use module::GlobalDefinition;
pub(super) use module::Module;
