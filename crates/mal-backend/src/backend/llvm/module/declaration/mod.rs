use crate::backend::llvm::body;
use crate::backend::llvm::syntax::{FunctionDeclaration, Module, Parameter, Type};

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
    result: Type,
    name: impl Into<String>,
    parameters: impl IntoIterator<Item = Type>,
) -> FunctionDeclaration {
    FunctionDeclaration::new(result, name, parameters.into_iter().map(Parameter::unnamed))
}

fn add_declaration(
    module: &mut Module<'_>,
    result: Type,
    name: impl Into<String>,
    parameters: impl IntoIterator<Item = Type>,
) {
    module.declare(declaration(result, name, parameters));
}
