use super::{add_declaration, declaration, strings};
use crate::backend::llvm::body;
use crate::backend::llvm::syntax::Module;

pub(super) fn add(module: &mut Module<'_>, types: &body::types::Types) {
    let index = types.index_integer();
    module.declare(
        declaration("ptr", "mal_runtime_bytes_data", strings(["ptr"])).with_attributes(strings([
            "nofree",
            "nounwind",
            "willreturn",
            "memory(argmem: read)",
        ])),
    );
    add_declaration(
        module,
        "ptr",
        "mal_runtime_bytes_read",
        vec!["ptr".into(), "ptr".into(), index.clone()],
    );
    add_declaration(
        module,
        "ptr",
        "mal_runtime_bytes_retain",
        strings(["ptr", "ptr"]),
    );
    add_declaration(
        module,
        "void",
        "mal_runtime_bytes_release",
        strings(["ptr"]),
    );
    add_declaration(
        module,
        "void",
        "mal_runtime_bytes_write",
        vec!["ptr".into(), "ptr".into(), index.clone(), index.clone()],
    );
    add_declaration(
        module,
        "ptr",
        "mal_runtime_buffer_make",
        vec!["ptr".into(), index.clone(), index.clone()],
    );
    add_declaration(
        module,
        "ptr",
        "mal_runtime_buffer_make_managed",
        vec![
            "ptr".into(),
            index.clone(),
            index.clone(),
            "ptr".into(),
            "ptr".into(),
        ],
    );
    add_declaration(
        module,
        index.clone(),
        "mal_runtime_buffer_new_managed",
        vec!["ptr".into(), "ptr".into(), "ptr".into(), index.clone()],
    );
    add_declaration(
        module,
        "void",
        "mal_runtime_buffer_fill_managed",
        vec![
            "ptr".into(),
            "ptr".into(),
            index.clone(),
            index.clone(),
            "ptr".into(),
            index.clone(),
        ],
    );
    add_declaration(
        module,
        "void",
        "mal_runtime_buffer_copy_managed",
        vec![
            "ptr".into(),
            "ptr".into(),
            index.clone(),
            "ptr".into(),
            index.clone(),
            index.clone(),
            index.clone(),
        ],
    );
    add_declaration(
        module,
        index.clone(),
        "mal_runtime_buffer_new",
        vec!["ptr".into(), "ptr".into(), "ptr".into(), index.clone()],
    );
    add_declaration(
        module,
        "void",
        "mal_runtime_buffer_fill",
        vec![
            "ptr".into(),
            "ptr".into(),
            index.clone(),
            index.clone(),
            "ptr".into(),
            index.clone(),
        ],
    );
    add_declaration(
        module,
        "void",
        "mal_runtime_buffer_copy",
        vec![
            "ptr".into(),
            "ptr".into(),
            index.clone(),
            "ptr".into(),
            index.clone(),
            index.clone(),
            index.clone(),
        ],
    );
    module.declare(
        declaration("ptr", "mal_runtime_buffer_data_slot", strings(["ptr"])).with_attributes(
            strings(["nofree", "nounwind", "willreturn", "memory(none)"]),
        ),
    );
    module.declare(
        declaration(index.clone(), "mal_runtime_buffer_count", strings(["ptr"])).with_attributes(
            strings(["nofree", "nounwind", "willreturn", "memory(argmem: read)"]),
        ),
    );
    add_declaration(
        module,
        "ptr",
        "mal_runtime_buffer_from",
        vec![
            "ptr".into(),
            "ptr".into(),
            index.clone(),
            index.clone(),
            index.clone(),
        ],
    );
    add_declaration(
        module,
        "void",
        "mal_runtime_buffer_into",
        vec![
            "ptr".into(),
            "ptr".into(),
            "ptr".into(),
            index.clone(),
            index.clone(),
            index.clone(),
        ],
    );
    add_declaration(
        module,
        "i8",
        "mal_runtime_symbol_at",
        vec!["ptr".into(), index.clone()],
    );
    for name in [
        "mal_runtime_symbol_concatenate",
        "mal_runtime_symbol_concatenate_consuming_left",
        "mal_runtime_symbol_concatenate_consuming_right",
    ] {
        add_declaration(
            module,
            "void",
            name,
            vec![
                "ptr".into(),
                "ptr".into(),
                "ptr".into(),
                "ptr".into(),
                index.clone(),
                "ptr".into(),
                "ptr".into(),
                index.clone(),
            ],
        );
    }
    add_declaration(
        module,
        "i8",
        "mal_runtime_symbol_equal",
        vec!["ptr".into(), index, "ptr".into(), types.index_integer()],
    );
}
