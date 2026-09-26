mod function;
mod module;

pub(super) use function::{BasicBlock, FunctionBuilder, FunctionDefinition};
pub(in crate::backend) use module::FunctionDeclaration;
pub(super) use module::Module;
