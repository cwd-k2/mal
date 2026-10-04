//! Program-independent C ABI surface shared by every generated file header.
//!
//! It assembles typed C syntax for runtime declarations, lifecycle and Buffer helpers, bridge
//! conversions, and aggregate templates. File-specific interfaces and target-layout descriptors
//! remain in `header`.

use crate::backend::c::syntax::{
    Attribute, Declaration, Directive, FunctionSignature, FunctionSpecifier, Parameter,
    TranslationUnit, TypeName, c_expr, c_function, c_items,
};
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
    output.push(Directive::named_type_define());
    output.push(Directive::structural_type_define(
        "mal_product",
        "mal_detail_product_type",
    ));
    output.push(Directive::structural_type_define(
        "mal_sum",
        "mal_detail_sum_type",
    ));
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
        type MalControlArena = struct {
            storage: *mut uint8_t,
            capacity: size_t,
        };
        type MalContext = struct {
            control: MalControlArena,
            native_stack_limit: uintptr_t,
        };
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
    output.extend(c_items! {
        type MalType_Symbol = struct {
            owner: *mut void,
            data: *const uint8_t,
            length: size_t,
        };
        type MalType_Buffer = *mut void;
    });
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
        ("MalType_Symbol", "mal_Symbol_t"),
        ("MalType_Buffer", "mal_Buffer_t"),
        ("MalType_ByteSize", "mal_ByteSize_t"),
        ("MalType_USize", "mal_USize_t"),
    ] {
        output.extend(c_items! { type { alias } = { source }; });
    }
    output.push(Declaration::function_pointer_type_alias(
        TypeName::named("void"),
        "mal_detail_sum_key_bool_t",
        [
            Parameter::unnamed(TypeName::named("mal_Unit_t")),
            Parameter::unnamed(TypeName::named("mal_Unit_t")),
        ],
    ));
    output.push(
        FunctionSignature::new(
            TypeName::named("mal_Bool_t").pointer(),
            "mal_detail_sum_type",
            [Parameter::unnamed(TypeName::named(
                "mal_detail_sum_key_bool_t",
            ))],
        )
        .with_specifiers([FunctionSpecifier::Overloadable]),
    );
    output.extend(c_items! { type mal_call_t = MalContext; });
    output.push(Declaration::function_pointer_type_alias(
        TypeName::named("void"),
        "MalRuntimeRetain",
        [
            Parameter::unnamed(TypeName::named("MalContext").pointer()),
            Parameter::unnamed(TypeName::named("void").pointer()),
        ],
    ));
    output.push(Declaration::function_pointer_type_alias(
        TypeName::named("void"),
        "MalRuntimeRelease",
        [Parameter::unnamed(TypeName::named("void").pointer())],
    ));
    output.extend(c_items! {
        type mal_storage_descriptor_t = struct {
            size: size_t,
            alignment: size_t,
            share: MalRuntimeRetain,
            drop: MalRuntimeRelease,
        };
    });
    output.blank_line();
    output.extend(c_items! {
        define!(mal_false = UINT8_C(0) as mal_Bool_t);
        define!(mal_true = UINT8_C(1) as mal_Bool_t);
        define!(mal_unit = mal_Unit_t { unused: UINT8_C(0) });
    });
    output.blank_line();
    output.extend(c_items! {
        #[noreturn] fn mal_trap(
            context: *mut MalContext,
            message: *const char,
        ) -> void;
        fn mal_runtime_allocate(
            context: *mut MalContext,
            size: size_t,
        ) -> *mut void;
        fn mal_runtime_deallocate(allocation: *mut void) -> void;
        fn mal_runtime_owner_retain(
            context: *mut MalContext,
            owner: *mut void,
        ) -> *mut void;
        fn mal_runtime_owner_release(owner: *mut void) -> void;
        fn mal_runtime_owner_is_unique(owner: *const void) -> uint8_t;
        fn mal_runtime_bytes_data(owner: *const void) -> *const uint8_t;
        fn mal_runtime_bytes_read(
            context: *mut MalContext,
            source: *const void,
            length: size_t,
        ) -> *mut void;
        fn mal_runtime_bytes_retain(
            context: *mut MalContext,
            owner: *const void,
        ) -> *mut void;
        fn mal_runtime_bytes_release(owner: *const void) -> void;
        fn mal_runtime_buffer_make(
            context: *mut MalContext,
            stride: size_t,
            capacity: size_t,
        ) -> *mut void;
        fn mal_runtime_buffer_make_managed(
            context: *mut MalContext,
            stride: size_t,
            capacity: size_t,
            retain: MalRuntimeRetain,
            release: MalRuntimeRelease,
        ) -> *mut void;
        fn mal_runtime_buffer_data_slot(buffer: *const void) -> *const *mut void;
        fn mal_runtime_buffer_count(buffer: *const void) -> size_t;
        fn mal_runtime_buffer_new(
            context: *mut MalContext,
            buffer: *mut void,
            value: *const void,
            stride: size_t,
        ) -> size_t;
        fn mal_runtime_buffer_new_managed_move(
            context: *mut MalContext,
            buffer: *mut void,
            value: *const void,
            stride: size_t,
        ) -> size_t;
        fn mal_runtime_buffer_new_move(
            context: *mut MalContext,
            buffer: *mut void,
            value: *const void,
        ) -> size_t;
        fn mal_runtime_buffer_replace_move(
            context: *mut MalContext,
            buffer: *mut void,
            index: size_t,
            value: *const void,
        ) -> void;
        fn mal_runtime_buffer_fill_move(
            context: *mut MalContext,
            buffer: *mut void,
            offset: size_t,
            count: size_t,
            value: *const void,
        ) -> void;
        fn mal_runtime_buffer_copy_values(
            context: *mut MalContext,
            destination: *mut void,
            destination_offset: size_t,
            source: *const void,
            source_offset: size_t,
            count: size_t,
        ) -> void;
        fn mal_runtime_buffer_append_values(
            context: *mut MalContext,
            buffer: *mut void,
            source: *const void,
            count: size_t,
        ) -> void;
        fn mal_runtime_buffer_extend(
            context: *mut MalContext,
            buffer: *mut void,
            count: size_t,
        ) -> *mut void;
        fn mal_runtime_buffer_truncate(buffer: *mut void, count: size_t) -> void;
        fn mal_runtime_buffer_reserve_elements(
            context: *mut MalContext,
            buffer: *mut void,
            capacity: size_t,
        ) -> void;
        #[static] #[inline] #[noreturn] fn mal_call_trap(
            call: *mut mal_call_t,
            message: *const char,
        ) -> void {
            mal_trap(call, message);
        }
        #[static] #[inline] fn mal_detail_symbol(
            call: *mut mal_call_t,
            source: *const void,
            length: size_t,
        ) -> mal_Symbol_t {
            let owner: *mut void = mal_runtime_bytes_read(
                call,
                source,
                length,
            );
            return mal_Symbol_t {
                owner: owner,
                data: mal_runtime_bytes_data(owner),
                length: length,
            };
        }
        #[static] #[inline] fn mal_detail_buffer_make(
            call: *mut mal_call_t,
            stride: size_t,
            capacity: size_t,
        ) -> mal_Buffer_t {
            return mal_runtime_buffer_make(
                call,
                stride,
                capacity,
            );
        }
        #[static] #[inline] fn mal_detail_buffer_make_managed(
            call: *mut mal_call_t,
            stride: size_t,
            capacity: size_t,
            retain: MalRuntimeRetain,
            release: MalRuntimeRelease,
        ) -> mal_Buffer_t {
            return mal_runtime_buffer_make_managed(
                call,
                stride,
                capacity,
                retain,
                release,
            );
        }
        #[static] #[inline] fn mal_detail_buffer_data(value: mal_Buffer_t) -> *mut void {
            return *mal_runtime_buffer_data_slot(value);
        }
        #[static] #[inline] fn mal_detail_buffer_count(value: mal_Buffer_t) -> size_t {
            return mal_runtime_buffer_count(value);
        }
    });
    append_host_lifecycle(&mut output);
    append_generated_header_templates(&mut output);
    output.blank_line();
    c_items! {
        if !defined(MAL_H) {
            ..{ output }
        }
    }
    .render()
}

fn append_host_lifecycle(output: &mut TranslationUnit) {
    output.extend(c_items! {
        #[static] #[inline] #[overloadable] fn mal_detail_retain(
            #[maybe_unused] call: *mut mal_call_t,
            #[maybe_unused] value: *mut void,
        ) -> void {}
        #[static] #[inline] #[overloadable] fn mal_detail_release(
            #[maybe_unused] value: *mut void,
        ) -> void {}
        #[static] #[inline] #[overloadable] fn mal_detail_retain(
            call: *mut mal_call_t,
            value: *mut mal_Symbol_t,
        ) -> void {
            mal_runtime_bytes_retain(call, (*value).owner);
        }
        #[static] #[inline] #[overloadable] fn mal_detail_release(
            value: *mut mal_Symbol_t,
        ) -> void {
            mal_runtime_bytes_release((*value).owner);
        }
        #[static] #[inline] #[overloadable] fn mal_detail_retain(
            call: *mut mal_call_t,
            value: *mut mal_Buffer_t,
        ) -> void {
            mal_runtime_owner_retain(call, *value);
        }
        #[static] #[inline] #[overloadable] fn mal_detail_release(
            value: *mut mal_Buffer_t,
        ) -> void {
            mal_runtime_owner_release(*value);
        }
        #[static] #[inline] fn mal_detail_symbol_storage_share(
            context: *mut MalContext,
            carrier: *mut void,
        ) -> void {
            mal_detail_retain(context, carrier as *mut mal_Symbol_t);
        }
        #[static] #[inline] fn mal_detail_symbol_storage_drop(
            carrier: *mut void,
        ) -> void {
            mal_detail_release(carrier as *mut mal_Symbol_t);
        }
        #[static] #[inline] fn mal_detail_buffer_storage_share(
            context: *mut MalContext,
            carrier: *mut void,
        ) -> void {
            mal_detail_retain(context, carrier as *mut mal_Buffer_t);
        }
        #[static] #[inline] fn mal_detail_buffer_storage_drop(
            carrier: *mut void,
        ) -> void {
            mal_detail_release(carrier as *mut mal_Buffer_t);
        }
        #[static] #[inline] #[overloadable] fn mal_detail_storage(
            #[maybe_unused] type: *mut void,
            size: size_t,
            alignment: size_t,
        ) -> mal_storage_descriptor_t {
            return mal_storage_descriptor_t {
                size: size,
                alignment: alignment,
                share: NULL,
                drop: NULL,
            };
        }
        #[static] #[inline] #[overloadable] fn mal_detail_storage(
            #[maybe_unused] type: *mut mal_Symbol_t,
            size: size_t,
            alignment: size_t,
        ) -> mal_storage_descriptor_t {
            return mal_storage_descriptor_t {
                size: size,
                alignment: alignment,
                share: mal_detail_symbol_storage_share,
                drop: mal_detail_symbol_storage_drop,
            };
        }
        #[static] #[inline] #[overloadable] fn mal_detail_storage(
            #[maybe_unused] type: *mut mal_Buffer_t,
            size: size_t,
            alignment: size_t,
        ) -> mal_storage_descriptor_t {
            return mal_storage_descriptor_t {
                size: size,
                alignment: alignment,
                share: mal_detail_buffer_storage_share,
                drop: mal_detail_buffer_storage_drop,
            };
        }
        #[static] #[inline] fn mal_detail_buffer(
            call: *mut mal_call_t,
            storage: mal_storage_descriptor_t,
            capacity: size_t,
        ) -> mal_Buffer_t {
            if storage.alignment > _Alignof(max_align_t) {
                mal_call_trap(call, "unsupported Buffer element alignment");
            }
            if storage.share != NULL {
                return mal_detail_buffer_make_managed(
                    call,
                    storage.size,
                    capacity,
                    storage.share,
                    storage.drop,
                );
            }
            return mal_detail_buffer_make(call, storage.size, capacity);
        }
        #[static] #[inline] fn mal_detail_buffer_push(
            call: *mut mal_call_t,
            buffer: mal_Buffer_t,
            element: *const void,
        ) -> size_t {
            return mal_runtime_buffer_new_move(
                call,
                buffer,
                element,
            );
        }
        #[static] #[inline] fn mal_detail_buffer_snapshot(
            call: *mut mal_call_t,
            buffer: mal_Buffer_t,
        ) -> mal_Symbol_t {
            return mal_detail_symbol(
                call,
                mal_detail_buffer_data(buffer),
                mal_detail_buffer_count(buffer),
            );
        }
    });
    for name in [
        "Unit", "Bool", "Int8", "Int16", "Int32", "Int64", "UInt8", "UInt16", "UInt32", "UInt64",
        "Float32", "Float64", "Symbol", "Buffer", "ByteSize", "USize",
    ] {
        output.push(c_function! {
            #[static] #[inline] fn { format!("mal_detail_cleanup_{name}") }(
                value: *mut { format!("mal_{name}_t") },
            ) -> void {
                mal_detail_release(value);
                memset(value, 0, sizeof(*value));
            }
        });
    }
    output.push(Directive::host_lifecycle_defines());
    output.push(Directive::owned_type_define());
    output.extend(c_items! {
        define!(mal_symbol(call, source, length) = mal_detail_symbol(call, source, length));
        define!(mal_buffer(call, element_type, capacity) = mal_detail_buffer(call, mal_storage(element_type), capacity));
        define!(mal_data(buffer) = mal_detail_buffer_data(buffer));
        define!(mal_count(buffer) = mal_detail_buffer_count(buffer));
        define!(mal_from_bits(type, bits) = mal_detail_from_bits(0 as *mut type, bits));
        define!(mal_bits(value) = mal_detail_bits(value));
    });
    output.push(Directive::buffer_push_define());
    output.push(Directive::buffer_mutation_defines());
    output.extend(c_items! {
        define!(mal_copy(call, destination, destination_offset, source, source_offset, count) = mal_runtime_buffer_copy_values(call, destination, destination_offset, source, source_offset, count));
        define!(mal_append(call, buffer, source, count) = mal_runtime_buffer_append_values(call, buffer, source, count));
        define!(mal_extend(call, buffer, count) = mal_runtime_buffer_extend(call, buffer, count));
        define!(mal_truncate(buffer, count) = mal_runtime_buffer_truncate(buffer, count));
        define!(mal_reserve(call, buffer, capacity) = mal_runtime_buffer_reserve_elements(call, buffer, capacity));
        define!(mal_snapshot(call, buffer) = mal_detail_buffer_snapshot(call, buffer));
    });
    output.blank_line();
}
