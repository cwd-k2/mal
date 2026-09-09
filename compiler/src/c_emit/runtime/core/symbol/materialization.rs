use super::*;

pub(super) fn append_symbol_copy(output: &mut TranslationUnit) {
    append_function(
        output,
        FunctionSignature::static_function(
            "MalType_Symbol",
            "mal_symbol_copy_from_bytes",
            [
                context_parameter(),
                Parameter::named(TypeName::const_named("uint8_t").pointer(), "data"),
                Parameter::named("uint64_t", "length"),
            ],
        ),
        Block::new([
            Statement::if_then(
                Expr::equal(Expr::identifier("length"), uint64(0)),
                Block::new([Statement::return_value(symbol([
                    Expr::identifier("NULL"),
                    uint64(0),
                    Expr::identifier("NULL"),
                ]))]),
            ),
            Statement::if_then(
                Expr::equal(Expr::identifier("data"), Expr::identifier("NULL")),
                trap("null Symbol data"),
            ),
            Statement::variable(
                "size_t",
                "size",
                Some(Expr::cast("size_t", Expr::identifier("length"))),
            ),
            Statement::if_then(
                Expr::not_equal(
                    Expr::cast("uint64_t", Expr::identifier("size")),
                    Expr::identifier("length"),
                ),
                trap("allocation size overflow"),
            ),
            Statement::variable(
                TypeName::named("uint8_t").pointer(),
                "copy",
                Some(Expr::cast(
                    TypeName::named("uint8_t").pointer(),
                    Expr::named_call(
                        "mal_allocate",
                        [Expr::identifier("context"), Expr::identifier("size")],
                    ),
                )),
            ),
            Statement::call(
                "memcpy",
                [
                    Expr::identifier("copy"),
                    Expr::identifier("data"),
                    Expr::identifier("size"),
                ],
            ),
            Statement::return_value(symbol([
                Expr::identifier("copy"),
                Expr::identifier("length"),
                Expr::identifier("copy"),
            ])),
        ]),
    );
}

pub(super) fn append_symbol_materialization(output: &mut TranslationUnit) {
    append_function(
        output,
        FunctionSignature::static_function(
            "uint8_t",
            "mal_symbol_copy_into",
            [
                Parameter::named("MalType_Symbol", "value"),
                Parameter::named(TypeName::named("uint8_t").pointer(), "bytes"),
            ],
        ),
        Block::new([
            Statement::if_then(
                Expr::logical_and(
                    Expr::not_equal(
                        Expr::identifier("value").field("ownership"),
                        Expr::identifier("NULL"),
                    ),
                    Expr::equal(
                        allocation_for(Expr::identifier("value").field("ownership"))
                            .pointer_field("capacity"),
                        Expr::identifier("SIZE_MAX"),
                    ),
                ),
                Block::new([
                    Statement::variable(
                        TypeName::const_named("MalSymbolRope").pointer(),
                        "rope",
                        Some(Expr::cast(
                            TypeName::const_named("MalSymbolRope").pointer(),
                            Expr::identifier("value").field("ownership"),
                        )),
                    ),
                    Statement::call(
                        "mal_symbol_copy_into",
                        [
                            Expr::identifier("rope").pointer_field("left"),
                            Expr::identifier("bytes"),
                        ],
                    ),
                    Statement::call(
                        "mal_symbol_copy_into",
                        [
                            Expr::identifier("rope").pointer_field("right"),
                            Expr::add(
                                Expr::identifier("bytes"),
                                Expr::cast(
                                    "size_t",
                                    Expr::identifier("rope")
                                        .pointer_field("left")
                                        .field("length"),
                                ),
                            ),
                        ],
                    ),
                    Statement::return_value(uint8(0)),
                ]),
            ),
            Statement::if_then(
                Expr::not_equal(Expr::identifier("value").field("length"), uint64(0)),
                Block::new([Statement::call(
                    "memcpy",
                    [
                        Expr::identifier("bytes"),
                        Expr::identifier("value").field("data"),
                        Expr::cast("size_t", Expr::identifier("value").field("length")),
                    ],
                )]),
            ),
            Statement::return_value(uint8(0)),
        ]),
    );
    append_function(
        output,
        FunctionSignature::new(
            "MalType_Symbol",
            "mal_symbol_materialize",
            [
                context_parameter(),
                Parameter::named("MalType_Symbol", "value"),
            ],
        ),
        Block::new([
            Statement::directive(Directive::If(PreprocessorExpr::defined(
                "MAL_TEST_MATERIALIZATION_LIMIT",
            ))),
            Statement::expression(Expr::pre_increment(Expr::identifier(
                "mal_test_materialization_count",
            ))),
            Statement::directive(Directive::Endif),
            Statement::if_then(
                Expr::logical_or(
                    Expr::not_equal(
                        Expr::identifier("value").field("data"),
                        Expr::identifier("NULL"),
                    ),
                    Expr::equal(Expr::identifier("value").field("length"), uint64(0)),
                ),
                Block::new([Statement::return_value(Expr::identifier("value"))]),
            ),
            Statement::variable(
                TypeName::named("MalSymbolRope").pointer(),
                "rope",
                Some(Expr::cast(
                    TypeName::named("MalSymbolRope").pointer(),
                    Expr::identifier("value").field("ownership"),
                )),
            ),
            Statement::if_then(
                Expr::equal(
                    Expr::identifier("rope").pointer_field("flattened"),
                    Expr::identifier("NULL"),
                ),
                Block::new([
                    Statement::directive(Directive::If(PreprocessorExpr::defined(
                        "MAL_TEST_FORCE_MATERIALIZATION_FAILURE",
                    ))),
                    Statement::call(
                        "mal_trap",
                        [
                            Expr::identifier("context"),
                            Expr::string("allocation failed"),
                        ],
                    ),
                    Statement::directive(Directive::Endif),
                    Statement::variable(
                        "size_t",
                        "size",
                        Some(Expr::cast(
                            "size_t",
                            Expr::identifier("value").field("length"),
                        )),
                    ),
                    Statement::if_then(
                        Expr::not_equal(
                            Expr::cast("uint64_t", Expr::identifier("size")),
                            Expr::identifier("value").field("length"),
                        ),
                        trap("allocation size overflow"),
                    ),
                    Statement::assignment(
                        Expr::identifier("rope").pointer_field("flattened"),
                        Expr::cast(
                            TypeName::named("uint8_t").pointer(),
                            Expr::named_call(
                                "mal_allocate",
                                [Expr::identifier("context"), Expr::identifier("size")],
                            ),
                        ),
                    ),
                    Statement::call(
                        "mal_symbol_copy_into",
                        [
                            Expr::identifier("value"),
                            Expr::identifier("rope").pointer_field("flattened"),
                        ],
                    ),
                    Statement::directive(Directive::If(PreprocessorExpr::defined(
                        "MAL_TEST_VALIDATE_SYMBOLS",
                    ))),
                    Statement::if_then(
                        Expr::not_equal(
                            allocation_for(Expr::identifier("rope").pointer_field("flattened"))
                                .pointer_field("capacity"),
                            Expr::identifier("size"),
                        ),
                        trap("invalid Symbol materialization cache"),
                    ),
                    Statement::directive(Directive::Endif),
                ]),
            ),
            Statement::return_value(symbol([
                Expr::identifier("rope").pointer_field("flattened"),
                Expr::identifier("value").field("length"),
                Expr::identifier("value").field("ownership"),
            ])),
        ]),
    );
}
