use super::artifact::RuntimeSource;

pub(crate) fn for_program(uses_byte_runtime: bool) -> Vec<RuntimeSource> {
    let mut sources = vec![
        RuntimeSource {
            name: "runtime.h",
            contents: include_str!("../../runtime/c11/runtime.h"),
        },
        RuntimeSource {
            name: "control.c",
            contents: include_str!("../../runtime/c11/control.c"),
        },
        RuntimeSource {
            name: "core.c",
            contents: include_str!("../../runtime/c11/core.c"),
        },
    ];
    if uses_byte_runtime {
        sources.extend([
            RuntimeSource {
                name: "bytes.c",
                contents: include_str!("../../runtime/c11/bytes.c"),
            },
            RuntimeSource {
                name: "bytes_internal.h",
                contents: include_str!("../../runtime/c11/bytes_internal.h"),
            },
            RuntimeSource {
                name: "symbol.c",
                contents: include_str!("../../runtime/c11/symbol.c"),
            },
        ]);
    }
    sources
}
