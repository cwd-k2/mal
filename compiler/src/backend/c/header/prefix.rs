use crate::backend::c::syntax::{
    AggregateDefinition, AggregateField, Attribute, Block, Comment, Declaration, Directive, Expr,
    FunctionDefinition, FunctionSignature, FunctionSpecifier, Initializer, Parameter,
    PreprocessorExpr, Statement, TranslationUnit, TypeName,
};

pub(super) fn emit_prefix() -> TranslationUnit {
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
    output.blank_line();
    output.push(Directive::define_expr(
        "MAL_C_ABI_VERSION",
        Expr::number("0x000600u"),
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
    ] {
        output.push(Declaration::type_alias(source, alias));
    }
    output.push(AggregateDefinition::typedef_structure(
        None,
        [
            AggregateField::variable(TypeName::const_named("uint8_t").pointer(), "data"),
            AggregateField::variable("uint64_t", "length"),
            AggregateField::variable(TypeName::named("void").pointer(), "ownership"),
        ],
        "MalType_Symbol",
    ));
    output.push(AggregateDefinition::typedef_structure(
        None,
        [AggregateField::variable(
            TypeName::named("uint8_t").pointer(),
            "address",
        )],
        "MalType_Ptr",
    ));
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
    ] {
        output.push(Declaration::type_alias(source, alias));
    }
    output.push(Declaration::type_alias(
        TypeName::named("void").pointer(),
        "mal_Ptr_t",
    ));
    output.push(AggregateDefinition::typedef_structure(
        None,
        [AggregateField::variable(
            TypeName::named("MalContext").pointer(),
            "mal_detail_context",
        )],
        "mal_call_t",
    ));
    output.push(AggregateDefinition::typedef_structure(
        None,
        [
            AggregateField::variable(TypeName::const_named("uint8_t").pointer(), "data"),
            AggregateField::variable("uint64_t", "length"),
        ],
        "mal_span_t",
    ));
    output.push(AggregateDefinition::typedef_structure(
        None,
        [
            AggregateField::variable("MalType_Symbol", "mal_detail_raw"),
            AggregateField::variable("mal_span_t", "mal_detail_bytes"),
            AggregateField::variable("uint8_t", "mal_detail_source"),
        ],
        "mal_Symbol_t",
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
    for signature in [
        FunctionSignature::new(
            "MalType_Symbol",
            "mal_symbol_materialize",
            [
                Parameter::named(TypeName::named("MalContext").pointer(), "context"),
                Parameter::named("MalType_Symbol", "value"),
            ],
        ),
        FunctionSignature::new(
            "MalType_Symbol",
            "mal_symbol_copy_from_bytes",
            [
                Parameter::named(TypeName::named("MalContext").pointer(), "context"),
                Parameter::named(TypeName::const_named("uint8_t").pointer(), "data"),
                Parameter::named("uint64_t", "length"),
            ],
        ),
        FunctionSignature::new(
            "MalType_Symbol",
            "mal_symbol_retain",
            [
                Parameter::named(TypeName::named("MalContext").pointer(), "context"),
                Parameter::named("MalType_Symbol", "value"),
            ],
        ),
    ] {
        output.push(Declaration::function(signature));
    }
    output.push(FunctionDefinition::from_signature(
        FunctionSignature::static_inline(
            "mal_Symbol_t",
            "mal_Symbol_from_bytes",
            [Parameter::named("mal_span_t", "bytes")],
        ),
        Block::new([Statement::return_value(Expr::compound_literal(
            "mal_Symbol_t",
            [
                Initializer::designated(
                    "mal_detail_raw",
                    Expr::compound_literal(
                        "MalType_Symbol",
                        [Initializer::positional(Expr::number("0"))],
                    ),
                ),
                Initializer::designated("mal_detail_bytes", Expr::identifier("bytes")),
                Initializer::designated(
                    "mal_detail_source",
                    Expr::named_call("UINT8_C", [Expr::number("1")]),
                ),
            ],
        ))]),
    ));
    output.push(FunctionDefinition::from_signature(
        FunctionSignature::static_inline(
            "mal_span_t",
            "mal_Symbol_to_bytes",
            [
                Parameter::named(TypeName::named("mal_call_t").pointer(), "call"),
                Parameter::named("mal_Symbol_t", "value"),
            ],
        ),
        Block::new([
            Statement::if_then(
                Expr::not_equal(
                    Expr::identifier("value").field("mal_detail_source"),
                    Expr::named_call("UINT8_C", [Expr::number("0")]),
                ),
                Block::new([Statement::return_value(
                    Expr::identifier("value").field("mal_detail_bytes"),
                )]),
            ),
            Statement::variable(
                "MalType_Symbol",
                "raw",
                Some(Expr::named_call(
                    "mal_symbol_materialize",
                    [
                        Expr::identifier("call").pointer_field("mal_detail_context"),
                        Expr::identifier("value").field("mal_detail_raw"),
                    ],
                )),
            ),
            Statement::return_value(Expr::compound_literal(
                "mal_span_t",
                [
                    Initializer::designated("data", Expr::identifier("raw").field("data")),
                    Initializer::designated("length", Expr::identifier("raw").field("length")),
                ],
            )),
        ]),
    ));
    output.push(FunctionDefinition::from_signature(
        FunctionSignature::static_inline(
            "MalType_Symbol",
            "mal_detail_Symbol_return",
            [
                Parameter::named(TypeName::named("mal_call_t").pointer(), "call"),
                Parameter::named("mal_Symbol_t", "value"),
            ],
        ),
        Block::new([
            Statement::if_then(
                Expr::not_equal(
                    Expr::identifier("value").field("mal_detail_source"),
                    Expr::named_call("UINT8_C", [Expr::number("0")]),
                ),
                Block::new([
                    Statement::if_then(
                        Expr::logical_and(
                            Expr::not_equal(
                                Expr::identifier("value")
                                    .field("mal_detail_bytes")
                                    .field("length"),
                                Expr::named_call("UINT64_C", [Expr::number("0")]),
                            ),
                            Expr::equal(
                                Expr::identifier("value")
                                    .field("mal_detail_bytes")
                                    .field("data"),
                                Expr::identifier("NULL"),
                            ),
                        ),
                        Block::new([Statement::call(
                            "mal_call_trap",
                            [Expr::identifier("call"), Expr::string("null Symbol data")],
                        )]),
                    ),
                    Statement::return_value(Expr::named_call(
                        "mal_symbol_copy_from_bytes",
                        [
                            Expr::identifier("call").pointer_field("mal_detail_context"),
                            Expr::identifier("value")
                                .field("mal_detail_bytes")
                                .field("data"),
                            Expr::identifier("value")
                                .field("mal_detail_bytes")
                                .field("length"),
                        ],
                    )),
                ]),
            ),
            Statement::return_value(Expr::named_call(
                "mal_symbol_retain",
                [
                    Expr::identifier("call").pointer_field("mal_detail_context"),
                    Expr::identifier("value").field("mal_detail_raw"),
                ],
            )),
        ]),
    ));
    output.push(FunctionDefinition::from_signature(
        FunctionSignature::static_inline(
            "MalType_Symbol",
            "mal_Symbol_return",
            [
                Parameter::named(TypeName::named("mal_call_t").pointer(), "call"),
                Parameter::named("mal_Symbol_t", "value"),
            ],
        ),
        Block::new([Statement::return_value(Expr::named_call(
            "mal_detail_Symbol_return",
            [Expr::identifier("call"), Expr::identifier("value")],
        ))]),
    ));
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
    output.push(FunctionDefinition::from_signature(
        FunctionSignature::static_inline(
            "MalType_Ptr",
            "mal_Ptr_return",
            [
                Parameter::named(TypeName::named("mal_call_t").pointer(), "call").maybe_unused(),
                Parameter::named("mal_Ptr_t", "value"),
            ],
        ),
        Block::new([Statement::return_value(Expr::compound_literal(
            "MalType_Ptr",
            [Initializer::designated(
                "address",
                Expr::cast(
                    TypeName::named("uint8_t").pointer(),
                    Expr::identifier("value"),
                ),
            )],
        ))]),
    ));
}
