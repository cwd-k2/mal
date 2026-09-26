use crate::backend::llvm::body;
use crate::backend::llvm::syntax::{FunctionDeclaration, Module};

mod byte;
mod control;
mod environment;

pub(super) fn add_byte_runtime(module: &mut Module<'_>, types: &body::types::Types) {
    byte::add(module, types);
}

pub(super) fn add_control(module: &mut Module<'_>, types: &body::types::Types) {
    control::add(module, types);
}

pub(super) fn add_environment(module: &mut Module<'_>, types: &body::types::Types) {
    environment::add(module, types);
}

fn declaration(
    result: impl Into<String>,
    name: impl Into<String>,
    parameters: impl IntoIterator<Item = String>,
) -> FunctionDeclaration {
    FunctionDeclaration::new(result, name, parameters)
}

fn add_declaration(
    module: &mut Module<'_>,
    result: impl Into<String>,
    name: impl Into<String>,
    parameters: impl IntoIterator<Item = String>,
) {
    module.declare(declaration(result, name, parameters));
}

fn strings<const N: usize>(values: [&str; N]) -> Vec<String> {
    values.into_iter().map(Into::into).collect()
}
