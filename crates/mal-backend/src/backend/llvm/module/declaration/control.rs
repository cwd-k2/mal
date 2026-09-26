use super::declaration;
use crate::backend::llvm::body;
use crate::backend::llvm::syntax::{Module, llvm_function_attributes, llvm_type};

pub(super) fn add(module: &mut Module<'_>, types: &body::types::Types) {
    let index = types.index_llvm_type();
    module.declare(declaration(
        llvm_type!(ptr),
        "mal_control_reserve_frame",
        [llvm_type!(ptr), index.clone(), index.clone()],
    ));
    module.declare(declaration(
        llvm_type!(ptr),
        "mal_control_storage",
        [llvm_type!(ptr)],
    ));
    module.declare(declaration(
        index,
        "mal_control_capacity",
        [llvm_type!(ptr)],
    ));
    module.declare(declaration(
        llvm_type!(void),
        "mal_native_stack_begin",
        [llvm_type!(ptr)],
    ));
    module.declare(
        declaration(
            llvm_type!(int(8_u16)),
            "mal_native_stack_is_deep",
            [llvm_type!(ptr)],
        )
        .with_attributes(llvm_function_attributes!(
            nofree,
            nounwind,
            willreturn,
            memory_argmem_read
        )),
    );
    module.declare(declaration(
        llvm_type!(int(1_u16)),
        "llvm.expect.i1",
        [llvm_type!(int(1_u16)), llvm_type!(int(1_u16))],
    ));
}
