use super::{add_declaration, declaration};
use crate::backend::llvm::body;
use crate::backend::llvm::syntax::{FunctionAttribute, Module, Type};

pub(super) fn add(module: &mut Module<'_>, types: &body::types::Types) {
    let index = types.index_llvm_type();
    let read_only = [
        FunctionAttribute::NoFree,
        FunctionAttribute::NoUnwind,
        FunctionAttribute::WillReturn,
        FunctionAttribute::MemoryArgMemRead,
    ];
    module.declare(
        declaration(Type::Pointer, "mal_runtime_bytes_data", [Type::Pointer])
            .with_attributes(read_only),
    );
    add_declaration(
        module,
        Type::Pointer,
        "mal_runtime_bytes_read",
        [Type::Pointer, Type::Pointer, index.clone()],
    );
    add_declaration(
        module,
        Type::Pointer,
        "mal_runtime_bytes_retain",
        [Type::Pointer, Type::Pointer],
    );
    add_declaration(
        module,
        Type::Void,
        "mal_runtime_bytes_release",
        [Type::Pointer],
    );
    add_declaration(
        module,
        Type::Void,
        "mal_runtime_bytes_write",
        [Type::Pointer, Type::Pointer, index.clone(), index.clone()],
    );
    add_declaration(
        module,
        Type::Pointer,
        "mal_runtime_buffer_make",
        [Type::Pointer, index.clone(), index.clone()],
    );
    add_declaration(
        module,
        Type::Pointer,
        "mal_runtime_buffer_make_managed",
        [
            Type::Pointer,
            index.clone(),
            index.clone(),
            Type::Pointer,
            Type::Pointer,
        ],
    );
    for name in ["mal_runtime_buffer_new_managed", "mal_runtime_buffer_new"] {
        add_declaration(
            module,
            index.clone(),
            name,
            [Type::Pointer, Type::Pointer, Type::Pointer, index.clone()],
        );
    }
    for name in ["mal_runtime_buffer_fill_managed", "mal_runtime_buffer_fill"] {
        add_declaration(
            module,
            Type::Void,
            name,
            [
                Type::Pointer,
                Type::Pointer,
                index.clone(),
                index.clone(),
                Type::Pointer,
                index.clone(),
            ],
        );
    }
    for name in ["mal_runtime_buffer_copy_managed", "mal_runtime_buffer_copy"] {
        add_declaration(
            module,
            Type::Void,
            name,
            [
                Type::Pointer,
                Type::Pointer,
                index.clone(),
                Type::Pointer,
                index.clone(),
                index.clone(),
                index.clone(),
            ],
        );
    }
    module.declare(
        declaration(
            Type::Pointer,
            "mal_runtime_buffer_data_slot",
            [Type::Pointer],
        )
        .with_attributes([
            FunctionAttribute::NoFree,
            FunctionAttribute::NoUnwind,
            FunctionAttribute::WillReturn,
            FunctionAttribute::MemoryNone,
        ]),
    );
    module.declare(
        declaration(index.clone(), "mal_runtime_buffer_count", [Type::Pointer])
            .with_attributes(read_only),
    );
    add_declaration(
        module,
        Type::Pointer,
        "mal_runtime_buffer_from",
        [
            Type::Pointer,
            Type::Pointer,
            index.clone(),
            index.clone(),
            index.clone(),
        ],
    );
    add_declaration(
        module,
        Type::Void,
        "mal_runtime_buffer_into",
        [
            Type::Pointer,
            Type::Pointer,
            Type::Pointer,
            index.clone(),
            index.clone(),
            index.clone(),
        ],
    );
    add_declaration(
        module,
        Type::integer(8_u16),
        "mal_runtime_symbol_at",
        [Type::Pointer, index.clone()],
    );
    for name in [
        "mal_runtime_symbol_concatenate",
        "mal_runtime_symbol_concatenate_consuming_left",
        "mal_runtime_symbol_concatenate_consuming_right",
    ] {
        add_declaration(
            module,
            Type::Void,
            name,
            [
                Type::Pointer,
                Type::Pointer,
                Type::Pointer,
                Type::Pointer,
                index.clone(),
                Type::Pointer,
                Type::Pointer,
                index.clone(),
            ],
        );
    }
    add_declaration(
        module,
        Type::integer(8_u16),
        "mal_runtime_symbol_equal",
        [Type::Pointer, index.clone(), Type::Pointer, index],
    );
}
