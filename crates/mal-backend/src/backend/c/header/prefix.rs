use crate::backend::c::syntax::{TranslationUnit, c_items};

use super::C_ABI_VERSION_LITERAL;

pub(super) fn emit_prefix(
    target: crate::backend::llvm::TargetLayout,
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
    let (symbol_size, symbol_offsets) = crate::backend::llvm::symbol_carrier_layout(target);
    output.extend(c_items! {
        assert!(MAL_C_ABI_VERSION == { abi_version }, "generated header requires mal C ABI 0x000b00");
        assert!(sizeof(0 as size_t) == { target.index_size }, "size_t does not match the mal target index width");
        assert!(sizeof(0 as *mut void) == { target.pointer_size }, "C pointer size does not match the mal target");
        assert!(sizeof(*(0 as *mut mal_Symbol_t)) == { symbol_size }, "Symbol carrier size does not match the mal target");
        assert!(offsetof(mal_Symbol_t, owner) == { symbol_offsets[0] }, "Symbol owner offset does not match the mal target");
        assert!(offsetof(mal_Symbol_t, data) == { symbol_offsets[1] }, "Symbol data offset does not match the mal target");
        assert!(offsetof(mal_Symbol_t, length) == { symbol_offsets[2] }, "Symbol length offset does not match the mal target");
    });
    output
}
