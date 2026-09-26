use super::declaration;
use crate::backend::llvm::body;
use crate::backend::llvm::syntax::{FunctionAttribute, Module, Type};

pub(super) fn add(module: &mut Module<'_>, types: &body::types::Types) {
    let index = types.index_llvm_type();
    module.declare(declaration(
        Type::Pointer,
        "mal_control_reserve_frame",
        [Type::Pointer, index.clone(), index.clone()],
    ));
    module.declare(declaration(
        Type::Pointer,
        "mal_control_storage",
        [Type::Pointer],
    ));
    module.declare(declaration(index, "mal_control_capacity", [Type::Pointer]));
    module.declare(declaration(
        Type::Void,
        "mal_native_stack_begin",
        [Type::Pointer],
    ));
    module.declare(
        declaration(
            Type::integer(8_u16),
            "mal_native_stack_is_deep",
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
        Type::integer(1_u16),
        "llvm.expect.i1",
        [Type::integer(1_u16), Type::integer(1_u16)],
    ));
}
