use crate::backend::c::syntax::{
    AggregateDefinition, AggregateField, Attribute, Block, Comment, Declaration, Directive, Expr,
    FunctionDefinition, FunctionSignature, FunctionSpecifier, Initializer, Parameter,
    PreprocessorExpr, Statement, TranslationUnit, TypeName,
};

pub(super) fn emit_prefix(index_bits: usize, memory_access: bool) -> TranslationUnit {
    let mut output = TranslationUnit::default();
    for directive in [
        Directive::Ifndef("MAL_PROGRAM_MAL_H".into()),
        Directive::define_empty("MAL_PROGRAM_MAL_H"),
    ] {
        output.push(directive);
    }
    output.blank_line();
    output.push(Directive::include_system("stddef.h"));
    output.push(Directive::include_system("stdint.h"));
    output.push(Directive::include_system("limits.h"));
    output.push(Directive::include_system("float.h"));
    if memory_access {
        output.push(Directive::include_system("string.h"));
    }
    output.blank_line();
    output.push(Directive::define_expr(
        "MAL_C_ABI_VERSION",
        Expr::number("0x000800u"),
    ));
    output.blank_line();
    output.push(Directive::If(PreprocessorExpr::defined("__clang__")));
    output.push(Directive::define_attribute(
        "MAL_DETAIL_MAYBE_UNUSED",
        Attribute::Unused,
    ));
    output.push(Directive::Else);
    output.push(Directive::define_empty("MAL_DETAIL_MAYBE_UNUSED"));
    output.push(Directive::Endif);
    output.blank_line();
    output.push(Comment::new("Runtime API"));
    output.blank_line();
    output.push(Declaration::type_alias(
        TypeName::structure("MalContext"),
        "MalContext",
    ));
    output.push(AggregateDefinition::typedef_structure(
        None,
        [AggregateField::variable("uint8_t", "unused")],
        "MalType_Unit",
    ));
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
        Expr::equal(
            Expr::multiply(
                Expr::sizeof_value(Expr::cast("size_t", Expr::number("0"))),
                Expr::identifier("CHAR_BIT"),
            ),
            Expr::number(index_bits.to_string()),
        ),
        "size_t does not match the mal target pointer index width",
    ));
    let width_of = |ty: &str, bits: u32| {
        Expr::equal(
            Expr::multiply(
                Expr::sizeof_value(Expr::cast(ty, Expr::number("0"))),
                Expr::identifier("CHAR_BIT"),
            ),
            Expr::number(bits.to_string()),
        )
    };
    let macro_equals =
        |name: &str, value: &str| Expr::equal(Expr::identifier(name), Expr::number(value));
    for (condition, message) in [
        (
            Expr::logical_and(width_of("float", 32), macro_equals("FLT_MANT_DIG", "24")),
            "float is not IEEE 754 binary32",
        ),
        (
            Expr::logical_and(width_of("double", 64), macro_equals("DBL_MANT_DIG", "53")),
            "double is not IEEE 754 binary64",
        ),
        (
            Expr::logical_and(
                macro_equals("FLT_HAS_SUBNORM", "1"),
                macro_equals("DBL_HAS_SUBNORM", "1"),
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
    output.push(AggregateDefinition::typedef_structure(
        None,
        [AggregateField::variable(
            TypeName::named("MalContext").pointer(),
            "mal_detail_context",
        )],
        "mal_call_t",
    ));
    output.blank_line();
    output.push(Directive::define_expr(
        "mal_false",
        Expr::cast(
            "mal_Bool_t",
            Expr::named_call("UINT8_C", [Expr::number("0")]),
        ),
    ));
    output.push(Directive::define_expr(
        "mal_true",
        Expr::cast(
            "mal_Bool_t",
            Expr::named_call("UINT8_C", [Expr::number("1")]),
        ),
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
    output.push(FunctionDefinition::from_signature(
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
        ]),
        Block::new([Statement::call(
            "mal_trap",
            [
                Expr::identifier("call").pointer_field("mal_detail_context"),
                Expr::identifier("message"),
            ],
        )]),
    ));
    append_builtin_returns(&mut output);
    output
}

fn append_builtin_returns(output: &mut TranslationUnit) {
    output.push(FunctionDefinition::from_signature(
        FunctionSignature::static_inline(
            "MalType_Unit",
            "mal_Unit_return",
            [Parameter::named(TypeName::named("mal_call_t").pointer(), "call").maybe_unused()],
        ),
        Block::new([Statement::return_value(Expr::compound_literal(
            "MalType_Unit",
            [Initializer::designated(
                "unused",
                Expr::named_call("UINT8_C", [Expr::number("0")]),
            )],
        ))]),
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
        output.push(FunctionDefinition::from_signature(
            FunctionSignature::static_inline(
                raw,
                format!("mal_{name}_return"),
                [
                    Parameter::named(TypeName::named("mal_call_t").pointer(), "call")
                        .maybe_unused(),
                    Parameter::named(host, "value"),
                ],
            ),
            Block::new([Statement::return_value(Expr::identifier("value"))]),
        ));
    }
    output.push(FunctionDefinition::from_signature(
        FunctionSignature::static_inline(
            "MalType_Address",
            "mal_Address_return",
            [
                Parameter::named(TypeName::named("mal_call_t").pointer(), "call"),
                Parameter::named("mal_Address_t", "value"),
            ],
        ),
        Block::new([
            Statement::if_then(
                Expr::equal(Expr::identifier("value"), Expr::number("0")),
                Block::new([Statement::call(
                    "mal_call_trap",
                    [
                        Expr::identifier("call"),
                        Expr::string("invalid Address result"),
                    ],
                )]),
            ),
            Statement::return_value(Expr::identifier("value")),
        ]),
    ));
    output.push(FunctionDefinition::from_signature(
        FunctionSignature::static_inline(
            "MalType_Bool",
            "mal_Bool_return",
            [
                Parameter::named(TypeName::named("mal_call_t").pointer(), "call"),
                Parameter::named("mal_Bool_t", "value"),
            ],
        ),
        Block::new([
            Statement::if_then(
                Expr::logical_and(
                    Expr::not_equal(Expr::identifier("value"), Expr::identifier("mal_false")),
                    Expr::not_equal(Expr::identifier("value"), Expr::identifier("mal_true")),
                ),
                Block::new([Statement::call(
                    "mal_call_trap",
                    [
                        Expr::identifier("call"),
                        Expr::string("invalid Bool result"),
                    ],
                )]),
            ),
            Statement::return_value(Expr::identifier("value")),
        ]),
    ));
}
