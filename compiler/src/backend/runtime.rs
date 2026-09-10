use super::artifact::RuntimeSource;

pub(crate) fn control() -> [RuntimeSource; 4] {
    [
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
        RuntimeSource {
            name: "symbol.c",
            contents: include_str!("../../runtime/c11/symbol.c"),
        },
    ]
}
