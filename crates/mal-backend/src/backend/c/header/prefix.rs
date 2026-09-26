use crate::backend::c::syntax::{
    Comment, Declaration, Directive, FunctionSignature, FunctionSpecifier, Parameter,
    TranslationUnit, TypeName, c_aggregate, c_declaration, c_directive, c_expr, c_function,
    c_signature,
};

pub(super) fn emit_prefix(index_bits: usize, memory_access: bool) -> TranslationUnit {
    let mut output = TranslationUnit::default();
    for directive in [
        c_directive!(ifndef "MAL_PROGRAM_MAL_H"),
        c_directive!(define "MAL_PROGRAM_MAL_H"),
    ] {
        output.push(directive);
    }
    output.blank_line();
    output.push(c_directive!(include(system "stddef.h")));
    output.push(c_directive!(include(system "stdint.h")));
    output.push(c_directive!(include(system "limits.h")));
    output.push(c_directive!(include(system "float.h")));
    if memory_access {
        output.push(c_directive!(include(system "string.h")));
    }
    output.blank_line();
    output.push(c_directive!(define "MAL_C_ABI_VERSION" = (number "0x000800u")));
    output.blank_line();
    output.push(c_directive!(if (defined("__clang__"))));
    output.push(c_directive!(define "MAL_DETAIL_MAYBE_UNUSED" = unused));
    output.push(c_directive!(else));
    output.push(c_directive!(define "MAL_DETAIL_MAYBE_UNUSED"));
    output.push(c_directive!(endif));
    output.blank_line();
    output.push(Comment::new("Runtime API"));
    output.blank_line();
    output.push(c_declaration!(type "MalContext" = struct("MalContext")));
    output.push(c_aggregate!(typedef struct => "MalType_Unit"; [
        ("unused": named("uint8_t")),
    ]));
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
        output.push(Declaration::type_alias(source, alias));
    }
    output.push(Declaration::type_alias(
        TypeName::named("void").pointer(),
        "MalType_Address",
    ));
    output.push(Declaration::static_assert(
        c_expr!(equal
            (multiply (sizeof (cast "size_t"; (number 0))); (id "CHAR_BIT"));
            (number index_bits)
        ),
        "size_t does not match the mal target pointer index width",
    ));
    let width_of = |ty: &str, bits: u32| {
        c_expr!(equal
            (multiply (sizeof (cast ty; (number 0))); (id "CHAR_BIT"));
            (number bits)
        )
    };
    let macro_equals = |name: &str, value: &str| c_expr!(equal (id name); (number value));
    for (condition, message) in [
        (
            c_expr!(logical_and
                { width_of("float", 32) };
                { macro_equals("FLT_MANT_DIG", "24") }
            ),
            "float is not IEEE 754 binary32",
        ),
        (
            c_expr!(logical_and
                { width_of("double", 64) };
                { macro_equals("DBL_MANT_DIG", "53") }
            ),
            "double is not IEEE 754 binary64",
        ),
        (
            c_expr!(logical_and
                { macro_equals("FLT_HAS_SUBNORM", "1") };
                { macro_equals("DBL_HAS_SUBNORM", "1") }
            ),
            "the target does not preserve subnormal floating-point values",
        ),
        (
            macro_equals("FLT_EVAL_METHOD", "0"),
            "floating-point expressions are evaluated with extra precision",
        ),
    ] {
        output.push(Declaration::static_assert(condition, message));
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
        output.push(Declaration::type_alias(source, alias));
    }
    output.push(c_aggregate!(typedef struct => "mal_call_t"; [
        ("mal_detail_context": ptr(named("MalContext"))),
    ]));
    output.blank_line();
    output.push(Directive::define_expr(
        "mal_false",
        c_expr!(cast "mal_Bool_t"; (call "UINT8_C"; (number 0))),
    ));
    output.push(Directive::define_expr(
        "mal_true",
        c_expr!(cast "mal_Bool_t"; (call "UINT8_C"; (number 1))),
    ));
    output.blank_line();
    output.push(Declaration::function(FunctionSignature::no_return(
        "void",
        "mal_trap",
        [
            Parameter::named(TypeName::named("MalContext").pointer(), "context"),
            Parameter::named(TypeName::const_named("char").pointer(), "message"),
        ],
    )));
    output.push(c_function!(signature {
        FunctionSignature::new(
            "void",
            "mal_call_trap",
            [
                Parameter::named(TypeName::named("mal_call_t").pointer(), "call"),
                Parameter::named(TypeName::const_named("char").pointer(), "message"),
            ],
        )
        .with_specifiers([
            FunctionSpecifier::Static,
            FunctionSpecifier::Inline,
            FunctionSpecifier::NoReturn,
        ])
    };
        block [(call "mal_trap";
                (pointer_field (id "call"); "mal_detail_context"),
                (id "message"),
        )]
    ));
    append_builtin_returns(&mut output);
    output
}

fn append_builtin_returns(output: &mut TranslationUnit) {
    output.push(c_function!(signature {
        c_signature!(static inline fn "mal_Unit_return"(
            "call": ptr(named("mal_call_t")) [maybe_unused],
        ) -> named("MalType_Unit"))
    };
        block [(return (compound "MalType_Unit";
            (field "unused"; (call "UINT8_C"; (number 0))),
        ))]
    ));
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
        output.push(c_function!(signature {
            FunctionSignature::static_inline(
                raw,
                format!("mal_{name}_return"),
                [
                    Parameter::named(TypeName::named("mal_call_t").pointer(), "call")
                        .maybe_unused(),
                    Parameter::named(host, "value"),
                ],
            )
        };
            block [(return (id "value"))]
        ));
    }
    output.push(c_function!(signature {
        FunctionSignature::static_inline(
            "MalType_Address",
            "mal_Address_return",
            [
                Parameter::named(TypeName::named("mal_call_t").pointer(), "call"),
                Parameter::named("mal_Address_t", "value"),
            ],
        )
    };
        block [
            (if (equal (id "value"); (number 0)); [
                (call "mal_call_trap";
                    (id "call"),
                    (string "invalid Address result"),
                ),
            ]),
            (return (id "value")),
        ]
    ));
    output.push(c_function!(signature {
        FunctionSignature::static_inline(
            "MalType_Bool",
            "mal_Bool_return",
            [
                Parameter::named(TypeName::named("mal_call_t").pointer(), "call"),
                Parameter::named("mal_Bool_t", "value"),
            ],
        )
    };
        block [
            (if (logical_and
                    (not_equal (id "value"); (id "mal_false"));
                    (not_equal (id "value"); (id "mal_true"))
                ); [
                (call "mal_call_trap";
                    (id "call"),
                    (string "invalid Bool result"),
                ),
            ]),
            (return (id "value")),
        ]
    ));
}
