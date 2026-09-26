use super::declaration;
use crate::backend::llvm::body;
use crate::backend::llvm::syntax::{
    FunctionAttribute, FunctionDeclaration, Module, Parameter, Type, llvm_parameter,
};

pub(super) fn add(module: &mut Module<'_>, types: &body::types::Types) {
    let index = types.index_llvm_type();
    module.declare(declaration(
        Type::Pointer,
        "mal_runtime_environment_allocate",
        [Type::Pointer, index.clone(), Type::Pointer],
    ));
    module.declare(declaration(
        Type::Pointer,
        "mal_runtime_environment_retain",
        [Type::Pointer, Type::Pointer],
    ));
    module.declare(declaration(
        Type::Void,
        "mal_runtime_environment_release",
        [Type::Pointer],
    ));
    module.declare(
        declaration(
            Type::integer(8_u16),
            "mal_runtime_environment_is_unique",
            [Type::Pointer],
        )
        .with_attributes([
            FunctionAttribute::NoFree,
            FunctionAttribute::NoUnwind,
            FunctionAttribute::WillReturn,
            FunctionAttribute::MemoryArgMemRead,
        ]),
    );
    module.declare(declaration(
        Type::Pointer,
        "llvm.invariant.start.p0",
        [Type::integer(64_u16), Type::Pointer],
    ));
    module.declare(declaration(
        Type::Pointer,
        format!("llvm.ptrmask.p0.i{}", types.pointer_size() * 8),
        [Type::Pointer, types.pointer_representation_llvm_type()],
    ));
    module.declare(FunctionDeclaration::new(
        Type::Void,
        format!("llvm.memcpy.p0.p0.i{}", types.index_size() * 8),
        [
            Parameter::unnamed(Type::Pointer),
            Parameter::unnamed(Type::Pointer),
            Parameter::unnamed(index),
            llvm_parameter!(_ : int(1) [immarg]),
        ],
    ));
}
