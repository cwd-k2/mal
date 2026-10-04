//! Program-independent C ABI surface shared by every generated file header.
//!
//! It assembles typed C syntax for runtime declarations, scalar helpers, built-in result validation,
//! and aggregate templates. File-specific interfaces and target-layout descriptors remain in `header`.

use crate::backend::c::syntax::{
    Attribute, Directive, TranslationUnit, c_expr, c_function, c_items,
};
use crate::backend::c::types::TypeRegistry;

mod templates;

use self::templates::append_generated_header_templates;
use super::C_ABI_VERSION_LITERAL;

pub(super) fn emit() -> String {
    let mut output = TranslationUnit::default();
    output.extend(c_items! { define!(MAL_H); });
    output.blank_line();
    output.extend(c_items! {
        include_system!("stddef.h");
        include_system!("stdint.h");
        include_system!("limits.h");
        include_system!("float.h");
        include_system!("string.h");
    });
    output.blank_line();
    let abi_version = crate::backend::c::syntax::Expr::number(C_ABI_VERSION_LITERAL);
    output.extend(c_items! { define!(MAL_C_ABI_VERSION = { abi_version }); });
    output.blank_line();
    let clang_attribute = Directive::define_attribute("MAL_DETAIL_MAYBE_UNUSED", Attribute::Unused);
    output.extend(c_items! {
        if defined(__clang__) {
            { clang_attribute };
        } else {
            define!(MAL_DETAIL_MAYBE_UNUSED);
        }
    });
    output.blank_line();
    output.extend(c_items! { comment!("Runtime API"); });
    output.blank_line();
    output.extend(c_items! {
        type MalContext = Struct<MalContext>;
        type MalType_Unit = struct {
            unused: uint8_t,
        };
    });
    for (source, alias) in [
        ("uint8_t", "MalType_Bool"),
        ("int8_t", "MalType_Int8"),
        ("int16_t", "MalType_Int16"),
        ("int32_t", "MalType_Int32"),
        ("int64_t", "MalType_Int64"),
        ("uint8_t", "MalType_UInt8"),
        ("uint16_t", "MalType_UInt16"),
        ("uint32_t", "MalType_UInt32"),
        ("uint64_t", "MalType_UInt64"),
        ("float", "MalType_Float32"),
        ("double", "MalType_Float64"),
        ("size_t", "MalType_ByteSize"),
        ("size_t", "MalType_USize"),
    ] {
        output.extend(c_items! { type { alias } = { source }; });
    }
    output.extend(c_items! { type MalType_Address = *mut void; });
    let width_of = |ty: &str, bits: u32| c_expr!(sizeof(0 as { ty }) * CHAR_BIT == { bits });
    let macro_equals = |name: &str, value: &str| {
        let value = crate::backend::c::syntax::Expr::number(value);
        c_expr!({ name } == { value })
    };
    for (condition, message) in [
        (
            c_expr!({ width_of("float", 32) } && { macro_equals("FLT_MANT_DIG", "24") }),
            "float is not IEEE 754 binary32",
        ),
        (
            c_expr!({ width_of("double", 64) } && { macro_equals("DBL_MANT_DIG", "53") }),
            "double is not IEEE 754 binary64",
        ),
        (
            c_expr!(
                { macro_equals("FLT_HAS_SUBNORM", "1") } && {
                    macro_equals("DBL_HAS_SUBNORM", "1")
                }
            ),
            "the target does not preserve subnormal floating-point values",
        ),
        (
            macro_equals("FLT_EVAL_METHOD", "0"),
            "floating-point expressions are evaluated with extra precision",
        ),
    ] {
        output.extend(c_items! { assert!({ condition }, { message }); });
    }
    for (source, alias) in [
        ("MalType_Unit", "mal_Unit_t"),
        ("MalType_Bool", "mal_Bool_t"),
        ("MalType_Int8", "mal_Int8_t"),
        ("MalType_Int16", "mal_Int16_t"),
        ("MalType_Int32", "mal_Int32_t"),
        ("MalType_Int64", "mal_Int64_t"),
        ("MalType_UInt8", "mal_UInt8_t"),
        ("MalType_UInt16", "mal_UInt16_t"),
        ("MalType_UInt32", "mal_UInt32_t"),
        ("MalType_UInt64", "mal_UInt64_t"),
        ("MalType_Float32", "mal_Float32_t"),
        ("MalType_Float64", "mal_Float64_t"),
        ("MalType_Address", "mal_Address_t"),
        ("MalType_ByteSize", "mal_ByteSize_t"),
        ("MalType_USize", "mal_USize_t"),
    ] {
        output.extend(c_items! { type { alias } = { source }; });
    }
    output.extend(c_items! {
        type mal_call_t = struct {
            mal_detail_context: *mut MalContext,
        };
    });
    output.blank_line();
    output.extend(c_items! {
        define!(mal_false = UINT8_C(0) as mal_Bool_t);
        define!(mal_true = UINT8_C(1) as mal_Bool_t);
    });
    output.blank_line();
    output.extend(c_items! {
        #[noreturn] fn mal_trap(
            context: *mut MalContext,
            message: *const char,
        ) -> void;
        #[static] #[inline] #[noreturn] fn mal_call_trap(
            call: *mut mal_call_t,
            message: *const char,
        ) -> void {
            mal_trap((*call).mal_detail_context, message);
        }
    });
    append_builtin_returns(&mut output);
    output.push(c_function! {
        #[static] #[inline] fn mal_detail_convert_Unit(
            #[maybe_unused] call: *mut mal_call_t,
            #[maybe_unused] value: mal_Unit_t,
        ) -> MalType_Unit {
            return MalType_Unit { _0: 0 };
        }
    });
    output.blank_line();
    output.extend(c_items! { comment!("Canonical scalar memory access"); });
    output.blank_line();
    output.push(c_function! {
        #[static] #[inline] fn mal_detail_memory_read_Unit(
            #[maybe_unused] call: *mut mal_call_t,
            #[maybe_unused] source: *const uint8_t,
        ) -> mal_Unit_t {
            return mal_Unit_t { _0: 0 };
        }
    });
    output.push(c_function! {
        #[static] #[inline] fn mal_detail_memory_write_Unit(
            #[maybe_unused] call: *mut mal_call_t,
            #[maybe_unused] destination: *mut uint8_t,
            value: mal_Unit_t,
        ) -> void {
            value as void;
        }
    });
    output.blank_line();
    output.extend(TypeRegistry::default().common_scalar_memory_helpers());
    output.blank_line();
    append_generated_header_templates(&mut output);
    output.blank_line();
    c_items! {
        if !defined(MAL_H) {
            ..{ output }
        }
    }
    .render()
}

fn append_builtin_returns(output: &mut TranslationUnit) {
    output.push(c_function! {
        #[static] #[inline] fn mal_Unit_return(
            #[maybe_unused] call: *mut mal_call_t,
        ) -> MalType_Unit {
            return MalType_Unit { unused: UINT8_C(0) };
        }
    });
    for (raw, host, name) in [
        ("MalType_Int8", "mal_Int8_t", "Int8"),
        ("MalType_Int16", "mal_Int16_t", "Int16"),
        ("MalType_Int32", "mal_Int32_t", "Int32"),
        ("MalType_Int64", "mal_Int64_t", "Int64"),
        ("MalType_UInt8", "mal_UInt8_t", "UInt8"),
        ("MalType_UInt16", "mal_UInt16_t", "UInt16"),
        ("MalType_UInt32", "mal_UInt32_t", "UInt32"),
        ("MalType_UInt64", "mal_UInt64_t", "UInt64"),
        ("MalType_Float32", "mal_Float32_t", "Float32"),
        ("MalType_Float64", "mal_Float64_t", "Float64"),
        ("MalType_ByteSize", "mal_ByteSize_t", "ByteSize"),
        ("MalType_USize", "mal_USize_t", "USize"),
    ] {
        output.push(c_function! {
            #[static] #[inline] fn { format!("mal_{name}_return") }(
                #[maybe_unused] call: *mut mal_call_t,
                value: { host },
            ) -> { raw } {
                return value;
            }
        });
    }
    output.push(c_function! {
        #[static] #[inline] fn mal_Address_return(
            call: *mut mal_call_t,
            value: mal_Address_t,
        ) -> MalType_Address {
            if value == 0 {
                mal_call_trap(call, "invalid Address result");
            }
            return value;
        }
    });
    output.push(c_function! {
        #[static] #[inline] fn mal_Bool_return(
            call: *mut mal_call_t,
            value: mal_Bool_t,
        ) -> MalType_Bool {
            if value != mal_false && value != mal_true {
                mal_call_trap(call, "invalid Bool result");
            }
            return value;
        }
    });
}
