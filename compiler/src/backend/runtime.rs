use super::artifact::RuntimeSource;

pub(crate) fn control() -> [RuntimeSource; 2] {
    [
        RuntimeSource {
            name: "runtime.h",
            contents: include_str!("../../runtime/c11/runtime.h"),
        },
        RuntimeSource {
            name: "control.c",
            contents: include_str!("../../runtime/c11/control.c"),
        },
    ]
}
