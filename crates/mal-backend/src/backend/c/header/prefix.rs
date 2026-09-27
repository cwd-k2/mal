use crate::backend::c::syntax::{TranslationUnit, c_declaration, c_directive};

pub(super) fn emit_prefix(index_bits: usize, memory_access: bool) -> TranslationUnit {
    let mut output = TranslationUnit::default();
    for directive in [
        c_directive!(ifndef "MAL_PROGRAM_MAL_H"),
        c_directive!(define "MAL_PROGRAM_MAL_H";),
    ] {
        output.push(directive);
    }
    output.blank_line();
    output.push(c_directive!(include(system "mal.h")));
    if memory_access {
        output.push(c_directive!(include(system "string.h")));
    }
    output.blank_line();
    output.push(c_declaration! {
        static_assert(
            (equal((id("MAL_C_ABI_VERSION")), (number("0x000900u")))),
            "generated header requires mal C ABI 0x000900"
        );
    });
    output.push(c_declaration! {
        static_assert(
            (equal(
                (multiply((sizeof((cast((named("size_t")), (number(0)))))), (id("CHAR_BIT")))),
                (number(#{ index_bits }))
            )),
            "size_t does not match the mal target pointer index width"
        );
    });
    output
}
