use super::declaration;
use crate::backend::llvm::body;
use crate::backend::llvm::syntax::{Module, llvm_declaration, llvm_function_attributes, llvm_type};

pub(super) fn add(module: &mut Module<'_>, types: &body::types::Types) {
    let index = types.index_llvm_type();
    module.declare(declaration(
        llvm_type!(ptr),
        "mal_runtime_owner_allocate",
        [llvm_type!(ptr), index.clone(), llvm_type!(ptr)],
    ));
    module.declare(declaration(
        llvm_type!(ptr),
        "mal_runtime_owner_retain",
        [llvm_type!(ptr), llvm_type!(ptr)],
    ));
    module.declare(declaration(
        llvm_type!(void),
        "mal_runtime_owner_release",
        [llvm_type!(ptr)],
    ));
    module.declare(
        declaration(
            llvm_type!(int(8_u16)),
            "mal_runtime_owner_is_unique",
            [llvm_type!(ptr)],
        )
        .with_attributes(llvm_function_attributes! {
            nofree,
            nounwind,
            willreturn,
            memory_argmem_read
        }),
    );
    module.declare(declaration(
        llvm_type!(ptr),
        "llvm.invariant.start.p0",
        [llvm_type!(int(64_u16)), llvm_type!(ptr)],
    ));
    module.declare(declaration(
        llvm_type!(ptr),
        format!("llvm.ptrmask.p0.i{}", types.pointer_size() * 8),
        [llvm_type!(ptr), types.pointer_representation_llvm_type()],
    ));
    module.declare(llvm_declaration! {
        fn #{ format!("llvm.memcpy.p0.p0.i{}", types.index_size() * 8) }(
            _ : ptr,
            _ : ptr,
            _ : #{ index },
            #[immarg] _ : int(1),
        ) -> void;
    });
}
