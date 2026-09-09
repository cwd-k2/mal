use super::*;

pub(super) fn append_symbol_admission(output: &mut TranslationUnit) {
    append_function(
        output,
        FunctionSignature::new(
            "MalSymbolAdmission",
            "mal_SymbolAdmission_begin",
            [
                context_parameter(),
                Parameter::named("uint64_t", "minimum_capacity"),
            ],
        ),
        Block::new([
            Statement::variable(
                TypeName::named("void").pointer(),
                "state",
                Some(Expr::identifier("NULL")),
            ),
            Statement::if_then(
                Expr::not_equal(Expr::identifier("minimum_capacity"), uint64(0)),
                Block::new([
                    Statement::variable(
                        "size_t",
                        "size",
                        Some(Expr::cast("size_t", Expr::identifier("minimum_capacity"))),
                    ),
                    Statement::if_then(
                        Expr::not_equal(
                            Expr::cast("uint64_t", Expr::identifier("size")),
                            Expr::identifier("minimum_capacity"),
                        ),
                        trap("allocation size overflow"),
                    ),
                    Statement::assignment(
                        Expr::identifier("state"),
                        Expr::named_call(
                            "mal_allocate",
                            [Expr::identifier("context"), Expr::identifier("size")],
                        ),
                    ),
                ]),
            ),
            Statement::return_value(Expr::compound_literal(
                "MalSymbolAdmission",
                [Initializer::designated("state", Expr::identifier("state"))],
            )),
        ]),
    );
    append_function(
        output,
        FunctionSignature::new(
            "uint64_t",
            "mal_SymbolAdmission_capacity",
            [Parameter::named(
                TypeName::const_named("MalSymbolAdmission").pointer(),
                "admission",
            )],
        ),
        Block::new([
            Statement::if_then(
                Expr::equal(
                    Expr::identifier("admission").pointer_field("state"),
                    Expr::identifier("NULL"),
                ),
                Block::new([Statement::return_value(uint64(0))]),
            ),
            Statement::return_value(Expr::cast(
                "uint64_t",
                allocation_for(Expr::identifier("admission").pointer_field("state"))
                    .pointer_field("capacity"),
            )),
        ]),
    );
    append_function(
        output,
        FunctionSignature::new(
            TypeName::named("uint8_t").pointer(),
            "mal_SymbolAdmission_data",
            [Parameter::named(
                TypeName::named("MalSymbolAdmission").pointer(),
                "admission",
            )],
        ),
        Block::new([Statement::return_value(Expr::cast(
            TypeName::named("uint8_t").pointer(),
            Expr::identifier("admission").pointer_field("state"),
        ))]),
    );

    let resize = Block::new([
        Statement::variable(
            TypeName::named("MalAllocation").pointer(),
            "allocation",
            Some(allocation_for(
                Expr::identifier("admission").pointer_field("state"),
            )),
        ),
        Statement::variable(
            "size_t",
            "capacity",
            Some(Expr::identifier("allocation").pointer_field("capacity")),
        ),
        Statement::if_then(
            Expr::greater_equal(Expr::identifier("capacity"), Expr::identifier("size")),
            Block::new([Statement::goto("mal_admission_reserve_done")]),
        ),
        Statement::variable(
            "size_t",
            "maximum_capacity",
            Some(Expr::subtract(
                Expr::identifier("SIZE_MAX"),
                Expr::sizeof_type("MalAllocation"),
            )),
        ),
        Statement::if_else(
            Expr::greater(
                Expr::identifier("capacity"),
                Expr::subtract(
                    Expr::identifier("maximum_capacity"),
                    Expr::identifier("capacity"),
                ),
            ),
            Block::new([Statement::assignment(
                Expr::identifier("capacity"),
                Expr::identifier("maximum_capacity"),
            )]),
            Block::new([Statement::assignment(
                Expr::identifier("capacity"),
                Expr::add(Expr::identifier("capacity"), Expr::identifier("capacity")),
            )]),
        ),
        Statement::if_then(
            Expr::less(Expr::identifier("capacity"), Expr::identifier("size")),
            Block::new([Statement::assignment(
                Expr::identifier("capacity"),
                Expr::identifier("size"),
            )]),
        ),
        Statement::directive(Directive::If(PreprocessorExpr::defined(
            "MAL_TEST_TOTAL_ALLOCATION_LIMIT",
        ))),
        Statement::expression(Expr::pre_increment(Expr::identifier(
            "mal_total_allocations",
        ))),
        Statement::directive(Directive::Endif),
        Statement::directive(Directive::If(PreprocessorExpr::defined(
            "MAL_TEST_FORCE_REALLOCATION_FAILURE",
        ))),
        Statement::variable(
            TypeName::named("MalAllocation").pointer(),
            "resized",
            Some(Expr::identifier("NULL")),
        ),
        Statement::directive(Directive::Else),
        Statement::variable(
            TypeName::named("MalAllocation").pointer(),
            "resized",
            Some(Expr::named_call(
                "realloc",
                [
                    Expr::identifier("allocation"),
                    Expr::add(
                        Expr::sizeof_type("MalAllocation"),
                        Expr::identifier("capacity"),
                    ),
                ],
            )),
        ),
        Statement::directive(Directive::Endif),
        Statement::if_then(
            Expr::equal(Expr::identifier("resized"), Expr::identifier("NULL")),
            trap("allocation failed"),
        ),
        Statement::assignment(
            Expr::identifier("resized").pointer_field("capacity"),
            Expr::identifier("capacity"),
        ),
        Statement::assignment(
            Expr::identifier("admission").pointer_field("state"),
            Expr::cast(
                TypeName::named("void").pointer(),
                Expr::add(Expr::identifier("resized"), Expr::number("1")),
            ),
        ),
    ]);
    append_function(
        output,
        FunctionSignature::new(
            "void",
            "mal_SymbolAdmission_reserve",
            [
                context_parameter(),
                Parameter::named(TypeName::named("MalSymbolAdmission").pointer(), "admission"),
                Parameter::named("uint64_t", "minimum_capacity"),
            ],
        ),
        Block::new([
            Statement::variable(
                "size_t",
                "size",
                Some(Expr::cast("size_t", Expr::identifier("minimum_capacity"))),
            ),
            Statement::if_then(
                Expr::not_equal(
                    Expr::cast("uint64_t", Expr::identifier("size")),
                    Expr::identifier("minimum_capacity"),
                ),
                trap("allocation size overflow"),
            ),
            Statement::if_else(
                Expr::equal(
                    Expr::identifier("admission").pointer_field("state"),
                    Expr::identifier("NULL"),
                ),
                Block::new([Statement::if_then(
                    Expr::not_equal(Expr::identifier("size"), Expr::number("0")),
                    Block::new([Statement::assignment(
                        Expr::identifier("admission").pointer_field("state"),
                        Expr::named_call(
                            "mal_allocate",
                            [Expr::identifier("context"), Expr::identifier("size")],
                        ),
                    )]),
                )]),
                resize,
            ),
            Statement::label(
                "mal_admission_reserve_done",
                Block::new([Statement::expression(Expr::cast("void", Expr::number("0")))]),
            ),
        ]),
    );
    append_function(
        output,
        FunctionSignature::new(
            "MalType_Symbol",
            "mal_SymbolAdmission_finish",
            [
                context_parameter(),
                Parameter::named(TypeName::named("MalSymbolAdmission").pointer(), "admission"),
                Parameter::named("uint64_t", "length"),
            ],
        ),
        Block::new([
            Statement::if_then(
                Expr::greater(
                    Expr::identifier("length"),
                    Expr::named_call(
                        "mal_SymbolAdmission_capacity",
                        [Expr::identifier("admission")],
                    ),
                ),
                trap("Symbol admission length exceeds capacity"),
            ),
            Statement::variable(
                TypeName::named("uint8_t").pointer(),
                "data",
                Some(Expr::cast(
                    TypeName::named("uint8_t").pointer(),
                    Expr::identifier("admission").pointer_field("state"),
                )),
            ),
            Statement::assignment(
                Expr::identifier("admission").pointer_field("state"),
                Expr::identifier("NULL"),
            ),
            Statement::if_then(
                Expr::equal(Expr::identifier("length"), uint64(0)),
                Block::new([
                    Statement::if_then(
                        Expr::not_equal(Expr::identifier("data"), Expr::identifier("NULL")),
                        Block::new([Statement::call(
                            "mal_deallocate",
                            [Expr::identifier("data")],
                        )]),
                    ),
                    Statement::return_value(symbol([
                        Expr::identifier("NULL"),
                        uint64(0),
                        Expr::identifier("NULL"),
                    ])),
                ]),
            ),
            Statement::return_value(symbol([
                Expr::identifier("data"),
                Expr::identifier("length"),
                Expr::identifier("data"),
            ])),
        ]),
    );
    append_function(
        output,
        FunctionSignature::new(
            "void",
            "mal_SymbolAdmission_drop",
            [
                context_parameter(),
                Parameter::named(TypeName::named("MalSymbolAdmission").pointer(), "admission"),
            ],
        ),
        Block::new([
            Statement::expression(Expr::cast("void", Expr::identifier("context"))),
            Statement::if_then(
                Expr::not_equal(
                    Expr::identifier("admission").pointer_field("state"),
                    Expr::identifier("NULL"),
                ),
                Block::new([Statement::call(
                    "mal_deallocate",
                    [Expr::identifier("admission").pointer_field("state")],
                )]),
            ),
            Statement::assignment(
                Expr::identifier("admission").pointer_field("state"),
                Expr::identifier("NULL"),
            ),
        ]),
    );
}
