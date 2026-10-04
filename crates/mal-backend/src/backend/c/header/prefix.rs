use crate::backend::c::syntax::{TranslationUnit, c_items};

use super::C_ABI_VERSION_LITERAL;

pub(super) fn emit_prefix(
    index_bits: usize,
    _memory_access: bool,
    dependencies: &[String],
    umbrella: bool,
) -> TranslationUnit {
    let mut output = c_items! { include_system!("mal.h"); };
    for dependency in dependencies {
        output.extend(c_items! { include_quoted!({ dependency }); });
    }
    if umbrella {
        output.extend(c_items! {
            define!(MAL_BUILD_UMBRELLA);
            define!(MAL_PROGRAM_MAL_H);
        });
    }
    output.blank_line();
    let abi_version = crate::backend::c::syntax::Expr::number(C_ABI_VERSION_LITERAL);
    output.extend(c_items! {
        assert!(MAL_C_ABI_VERSION == { abi_version }, "generated header requires mal C ABI 0x000900");
        assert!(sizeof(0 as size_t) * CHAR_BIT == { index_bits }, "size_t does not match the mal target pointer index width");
    });
    output
}
