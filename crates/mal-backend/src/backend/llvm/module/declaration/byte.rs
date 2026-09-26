use super::{add_declaration, declaration};
use crate::backend::llvm::body;
use crate::backend::llvm::syntax::{Module, llvm_function_attributes, llvm_type};

pub(super) fn add(module: &mut Module<'_>, types: &body::types::Types) {
    let index = types.index_llvm_type();
    module.declare(
        declaration(llvm_type!(ptr), "mal_runtime_bytes_data", [llvm_type!(ptr)]).with_attributes(
            llvm_function_attributes!(nofree, nounwind, willreturn, memory_argmem_read),
        ),
    );
    add_declaration(
        module,
        llvm_type!(ptr),
        "mal_runtime_bytes_read",
        [llvm_type!(ptr), llvm_type!(ptr), index.clone()],
    );
    add_declaration(
        module,
        llvm_type!(ptr),
        "mal_runtime_bytes_retain",
        [llvm_type!(ptr), llvm_type!(ptr)],
    );
    add_declaration(
        module,
        llvm_type!(void),
        "mal_runtime_bytes_release",
        [llvm_type!(ptr)],
    );
    add_declaration(
        module,
        llvm_type!(void),
        "mal_runtime_bytes_write",
        [
            llvm_type!(ptr),
            llvm_type!(ptr),
            index.clone(),
            index.clone(),
        ],
    );
    add_declaration(
        module,
        llvm_type!(ptr),
        "mal_runtime_buffer_make",
        [llvm_type!(ptr), index.clone(), index.clone()],
    );
    add_declaration(
        module,
        llvm_type!(ptr),
        "mal_runtime_buffer_make_managed",
        [
            llvm_type!(ptr),
            index.clone(),
            index.clone(),
            llvm_type!(ptr),
            llvm_type!(ptr),
        ],
    );
    for name in ["mal_runtime_buffer_new_managed", "mal_runtime_buffer_new"] {
        add_declaration(
            module,
            index.clone(),
            name,
            [
                llvm_type!(ptr),
                llvm_type!(ptr),
                llvm_type!(ptr),
                index.clone(),
            ],
        );
    }
    for name in ["mal_runtime_buffer_fill_managed", "mal_runtime_buffer_fill"] {
        add_declaration(
            module,
            llvm_type!(void),
            name,
            [
                llvm_type!(ptr),
                llvm_type!(ptr),
                index.clone(),
                index.clone(),
                llvm_type!(ptr),
                index.clone(),
            ],
        );
    }
    for name in ["mal_runtime_buffer_copy_managed", "mal_runtime_buffer_copy"] {
        add_declaration(
            module,
            llvm_type!(void),
            name,
            [
                llvm_type!(ptr),
                llvm_type!(ptr),
                index.clone(),
                llvm_type!(ptr),
                index.clone(),
                index.clone(),
                index.clone(),
            ],
        );
    }
    module.declare(
        declaration(
            llvm_type!(ptr),
            "mal_runtime_buffer_data_slot",
            [llvm_type!(ptr)],
        )
        .with_attributes(llvm_function_attributes!(
            nofree,
            nounwind,
            willreturn,
            memory_none
        )),
    );
    module.declare(
        declaration(index.clone(), "mal_runtime_buffer_count", [llvm_type!(ptr)]).with_attributes(
            llvm_function_attributes!(nofree, nounwind, willreturn, memory_argmem_read),
        ),
    );
    add_declaration(
        module,
        llvm_type!(ptr),
        "mal_runtime_buffer_from",
        [
            llvm_type!(ptr),
            llvm_type!(ptr),
            index.clone(),
            index.clone(),
            index.clone(),
        ],
    );
    add_declaration(
        module,
        llvm_type!(void),
        "mal_runtime_buffer_into",
        [
            llvm_type!(ptr),
            llvm_type!(ptr),
            llvm_type!(ptr),
            index.clone(),
            index.clone(),
            index.clone(),
        ],
    );
    add_declaration(
        module,
        llvm_type!(int(8_u16)),
        "mal_runtime_symbol_at",
        [llvm_type!(ptr), index.clone()],
    );
    for name in [
        "mal_runtime_symbol_concatenate",
        "mal_runtime_symbol_concatenate_consuming_left",
        "mal_runtime_symbol_concatenate_consuming_right",
    ] {
        add_declaration(
            module,
            llvm_type!(void),
            name,
            [
                llvm_type!(ptr),
                llvm_type!(ptr),
                llvm_type!(ptr),
                llvm_type!(ptr),
                index.clone(),
                llvm_type!(ptr),
                llvm_type!(ptr),
                index.clone(),
            ],
        );
    }
    add_declaration(
        module,
        llvm_type!(int(8_u16)),
        "mal_runtime_symbol_equal",
        [llvm_type!(ptr), index.clone(), llvm_type!(ptr), index],
    );
}
