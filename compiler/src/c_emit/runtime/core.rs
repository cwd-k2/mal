use crate::c_emit::syntax::{
    AggregateDefinition, AggregateField, Block, Declaration, Directive, Expr, FunctionDefinition,
    FunctionSignature, Initializer, Parameter, PreprocessorExpr, Statement, TranslationUnit,
    TypeName, VariableDeclaration,
};

pub(super) fn emit() -> TranslationUnit {
    let mut output = TranslationUnit::default();
    output.push(AggregateDefinition::typedef_structure(
        None,
        [
            AggregateField::variable("uint64_t", "references"),
            AggregateField::variable("size_t", "capacity"),
        ],
        "MalAllocation",
    ));
    output.blank_line();
    output.push(AggregateDefinition::typedef_structure(
        Some("MalSymbolRope".into()),
        [
            AggregateField::variable("MalType_Symbol", "left"),
            AggregateField::variable("MalType_Symbol", "right"),
            AggregateField::variable(TypeName::named("uint8_t").pointer(), "flattened"),
            AggregateField::variable("uint8_t", "height"),
        ],
        "MalSymbolRope",
    ));
    output.blank_line();
    output.push(AggregateDefinition::structure(
        "MalContext",
        [AggregateField::variable("uint8_t", "unused")],
    ));
    output.push(Directive::If(live_allocation_tracking()));
    output.push(Declaration::variable(VariableDeclaration::static_variable(
        "size_t",
        "mal_live_allocations",
    )));
    output.push(Directive::Endif);
    output.push(Directive::If(PreprocessorExpr::defined(
        "MAL_TEST_TOTAL_ALLOCATION_LIMIT",
    )));
    output.push(Declaration::variable(VariableDeclaration::static_variable(
        "size_t",
        "mal_total_allocations",
    )));
    output.push(Directive::Endif);
    output.blank_line();

    append_trap(&mut output);
    append_function(
        &mut output,
        FunctionSignature::static_function("void", "mal_context_destroy", [context_parameter()]),
        Block::new([
            Statement::expression(Expr::cast("void", Expr::identifier("context"))),
            Statement::directive(Directive::If(PreprocessorExpr::defined(
                "MAL_TEST_REQUIRE_NO_LIVE_ALLOCATIONS",
            ))),
            Statement::if_then(
                Expr::not_equal(Expr::identifier("mal_live_allocations"), Expr::number("0")),
                trap("live allocations at context destruction"),
            ),
            Statement::directive(Directive::Endif),
            Statement::directive(Directive::If(PreprocessorExpr::defined(
                "MAL_TEST_TOTAL_ALLOCATION_LIMIT",
            ))),
            Statement::if_then(
                Expr::greater(
                    Expr::identifier("mal_total_allocations"),
                    Expr::cast(
                        "size_t",
                        Expr::identifier("MAL_TEST_TOTAL_ALLOCATION_LIMIT"),
                    ),
                ),
                trap("total allocation limit exceeded"),
            ),
            Statement::directive(Directive::Endif),
        ]),
    );
    append_allocation(&mut output);
    append_reference_counting(&mut output);
    append_symbol_lifetime(&mut output);
    append_host_symbol_lifetime(&mut output);
    append_symbol_copy(&mut output);
    append_symbol_materialization(&mut output);
    output
}

fn append_trap(output: &mut TranslationUnit) {
    append_function(
        output,
        FunctionSignature::no_return(
            "void",
            "mal_trap",
            [
                context_parameter(),
                Parameter::named(TypeName::const_named("char").pointer(), "message"),
            ],
        ),
        Block::new([
            Statement::expression(Expr::cast("void", Expr::identifier("context"))),
            Statement::call(
                "fputs",
                [Expr::string("mal trap: "), Expr::identifier("stderr")],
            ),
            Statement::call(
                "fputs",
                [Expr::identifier("message"), Expr::identifier("stderr")],
            ),
            Statement::call("fputc", [Expr::character('\n'), Expr::identifier("stderr")]),
            Statement::call("abort", []),
        ]),
    );
}

fn append_allocation(output: &mut TranslationUnit) {
    append_function(
        output,
        FunctionSignature::static_function(
            TypeName::named("void").pointer(),
            "mal_allocate",
            [context_parameter(), Parameter::named("size_t", "size")],
        ),
        Block::new([
            Statement::if_then(
                Expr::greater(
                    Expr::identifier("size"),
                    Expr::subtract(
                        Expr::identifier("SIZE_MAX"),
                        Expr::sizeof_type("MalAllocation"),
                    ),
                ),
                trap("allocation size overflow"),
            ),
            Statement::directive(Directive::If(PreprocessorExpr::defined(
                "MAL_TEST_LIVE_ALLOCATION_LIMIT",
            ))),
            Statement::if_then(
                Expr::greater_equal(
                    Expr::identifier("mal_live_allocations"),
                    Expr::cast("size_t", Expr::identifier("MAL_TEST_LIVE_ALLOCATION_LIMIT")),
                ),
                trap("allocation failed"),
            ),
            Statement::directive(Directive::Endif),
            Statement::directive(Directive::If(PreprocessorExpr::defined(
                "MAL_TEST_FORCE_ALLOCATION_FAILURE",
            ))),
            Statement::variable(
                TypeName::named("MalAllocation").pointer(),
                "allocation",
                Some(Expr::identifier("NULL")),
            ),
            Statement::directive(Directive::Else),
            Statement::variable(
                TypeName::named("MalAllocation").pointer(),
                "allocation",
                Some(Expr::named_call(
                    "malloc",
                    [Expr::add(
                        Expr::sizeof_type("MalAllocation"),
                        Expr::identifier("size"),
                    )],
                )),
            ),
            Statement::directive(Directive::Endif),
            Statement::if_then(
                Expr::equal(Expr::identifier("allocation"), Expr::identifier("NULL")),
                trap("allocation failed"),
            ),
            Statement::assignment(
                Expr::identifier("allocation").pointer_field("references"),
                uint64(1),
            ),
            Statement::assignment(
                Expr::identifier("allocation").pointer_field("capacity"),
                Expr::identifier("size"),
            ),
            Statement::directive(Directive::If(live_allocation_tracking())),
            Statement::expression(Expr::pre_increment(Expr::identifier(
                "mal_live_allocations",
            ))),
            Statement::directive(Directive::Endif),
            Statement::directive(Directive::If(PreprocessorExpr::defined(
                "MAL_TEST_TOTAL_ALLOCATION_LIMIT",
            ))),
            Statement::expression(Expr::pre_increment(Expr::identifier(
                "mal_total_allocations",
            ))),
            Statement::directive(Directive::Endif),
            Statement::return_value(Expr::add(Expr::identifier("allocation"), Expr::number("1"))),
        ]),
    );
}

fn append_reference_counting(output: &mut TranslationUnit) {
    append_function(
        output,
        FunctionSignature::static_function(
            TypeName::const_named("void").pointer(),
            "mal_retain",
            [
                context_parameter(),
                Parameter::named(TypeName::const_named("void").pointer(), "value"),
            ],
        ),
        Block::new([
            Statement::if_then(
                Expr::equal(Expr::identifier("value"), Expr::identifier("NULL")),
                Block::new([Statement::return_value(Expr::identifier("NULL"))]),
            ),
            Statement::variable(
                TypeName::named("MalAllocation").pointer(),
                "allocation",
                Some(allocation_for(Expr::identifier("value"))),
            ),
            Statement::if_then(
                Expr::equal(
                    Expr::identifier("allocation").pointer_field("references"),
                    Expr::identifier("UINT64_MAX"),
                ),
                trap("reference count overflow"),
            ),
            Statement::expression(Expr::pre_increment(
                Expr::identifier("allocation").pointer_field("references"),
            )),
            Statement::return_value(Expr::identifier("value")),
        ]),
    );
    append_function(
        output,
        FunctionSignature::static_function(
            "uint8_t",
            "mal_release",
            [Parameter::named(
                TypeName::const_named("void").pointer(),
                "value",
            )],
        ),
        Block::new([
            Statement::if_then(
                Expr::equal(Expr::identifier("value"), Expr::identifier("NULL")),
                Block::new([Statement::return_value(uint8(0))]),
            ),
            Statement::variable(
                TypeName::named("MalAllocation").pointer(),
                "allocation",
                Some(allocation_for(Expr::identifier("value"))),
            ),
            Statement::assignment(
                Expr::identifier("allocation").pointer_field("references"),
                Expr::subtract(
                    Expr::identifier("allocation").pointer_field("references"),
                    uint64(1),
                ),
            ),
            Statement::return_value(Expr::cast(
                "uint8_t",
                Expr::equal(
                    Expr::identifier("allocation").pointer_field("references"),
                    uint64(0),
                ),
            )),
        ]),
    );
    append_function(
        output,
        FunctionSignature::static_function(
            "void",
            "mal_deallocate",
            [Parameter::named(
                TypeName::const_named("void").pointer(),
                "value",
            )],
        ),
        Block::new([
            Statement::directive(Directive::If(live_allocation_tracking())),
            Statement::assignment(
                Expr::identifier("mal_live_allocations"),
                Expr::subtract(Expr::identifier("mal_live_allocations"), Expr::number("1")),
            ),
            Statement::directive(Directive::Endif),
            Statement::call("free", [allocation_for(Expr::identifier("value"))]),
        ]),
    );
}

fn append_symbol_lifetime(output: &mut TranslationUnit) {
    append_function(
        output,
        FunctionSignature::new(
            "MalType_Symbol",
            "mal_symbol_retain",
            [
                context_parameter(),
                Parameter::named("MalType_Symbol", "value"),
            ],
        ),
        Block::new([
            Statement::expression(Expr::named_call(
                "mal_retain",
                [
                    Expr::identifier("context"),
                    Expr::identifier("value").field("ownership"),
                ],
            )),
            Statement::return_value(Expr::identifier("value")),
        ]),
    );
    append_function(
        output,
        FunctionSignature::new(
            "void",
            "mal_symbol_release",
            [Parameter::named("MalType_Symbol", "value")],
        ),
        Block::new([Statement::if_then(
            Expr::not_equal(
                Expr::identifier("value").field("ownership"),
                Expr::identifier("NULL"),
            ),
            Block::new([Statement::if_then(
                Expr::not_equal(
                    Expr::named_call(
                        "mal_release",
                        [Expr::identifier("value").field("ownership")],
                    ),
                    uint8(0),
                ),
                Block::new([
                    Statement::if_then(
                        Expr::equal(
                            allocation_for(Expr::identifier("value").field("ownership"))
                                .pointer_field("capacity"),
                            Expr::identifier("SIZE_MAX"),
                        ),
                        Block::new([
                            Statement::variable(
                                TypeName::named("MalSymbolRope").pointer(),
                                "rope",
                                Some(Expr::cast(
                                    TypeName::named("MalSymbolRope").pointer(),
                                    Expr::identifier("value").field("ownership"),
                                )),
                            ),
                            Statement::call(
                                "mal_symbol_release",
                                [Expr::identifier("rope").pointer_field("right")],
                            ),
                            Statement::call(
                                "mal_symbol_release",
                                [Expr::identifier("rope").pointer_field("left")],
                            ),
                            Statement::if_then(
                                Expr::not_equal(
                                    Expr::identifier("rope").pointer_field("flattened"),
                                    Expr::identifier("NULL"),
                                ),
                                Block::new([Statement::call(
                                    "mal_deallocate",
                                    [Expr::identifier("rope").pointer_field("flattened")],
                                )]),
                            ),
                        ]),
                    ),
                    Statement::call(
                        "mal_deallocate",
                        [Expr::identifier("value").field("ownership")],
                    ),
                ]),
            )]),
        )]),
    );
}

fn append_host_symbol_lifetime(output: &mut TranslationUnit) {
    append_function(
        output,
        FunctionSignature::new(
            "MalType_Symbol",
            "mal_Symbol_clone",
            [
                context_parameter(),
                Parameter::named("MalType_Symbol", "value"),
            ],
        ),
        Block::new([Statement::return_value(Expr::named_call(
            "mal_symbol_retain",
            [Expr::identifier("context"), Expr::identifier("value")],
        ))]),
    );
    append_function(
        output,
        FunctionSignature::new(
            "MalType_Symbol",
            "mal_Symbol_take",
            [Parameter::named(
                TypeName::named("MalType_Symbol").pointer(),
                "value",
            )],
        ),
        Block::new([
            Statement::variable(
                "MalType_Symbol",
                "result",
                Some(Expr::dereference(Expr::identifier("value"))),
            ),
            Statement::assignment(
                Expr::dereference(Expr::identifier("value")),
                symbol([
                    Expr::identifier("NULL"),
                    uint64(0),
                    Expr::identifier("NULL"),
                ]),
            ),
            Statement::return_value(Expr::identifier("result")),
        ]),
    );
    append_function(
        output,
        FunctionSignature::new(
            "void",
            "mal_Symbol_drop",
            [
                context_parameter(),
                Parameter::named(TypeName::named("MalType_Symbol").pointer(), "value"),
            ],
        ),
        Block::new([
            Statement::expression(Expr::cast("void", Expr::identifier("context"))),
            Statement::call(
                "mal_symbol_release",
                [Expr::dereference(Expr::identifier("value"))],
            ),
            Statement::assignment(
                Expr::dereference(Expr::identifier("value")),
                symbol([
                    Expr::identifier("NULL"),
                    uint64(0),
                    Expr::identifier("NULL"),
                ]),
            ),
        ]),
    );
}

fn append_symbol_copy(output: &mut TranslationUnit) {
    append_function(
        output,
        FunctionSignature::new(
            "MalType_Symbol",
            "mal_Symbol_copy_from_bytes",
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

fn append_symbol_materialization(output: &mut TranslationUnit) {
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
            Statement::if_then(
                Expr::logical_or(
                    Expr::equal(
                        Expr::identifier("value").field("ownership"),
                        Expr::identifier("NULL"),
                    ),
                    Expr::not_equal(
                        allocation_for(Expr::identifier("value").field("ownership"))
                            .pointer_field("capacity"),
                        Expr::identifier("SIZE_MAX"),
                    ),
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

fn allocation_for(value: Expr) -> Expr {
    Expr::subtract(
        Expr::cast(TypeName::named("MalAllocation").pointer(), value),
        Expr::number("1"),
    )
}

fn context_parameter() -> Parameter {
    Parameter::named(TypeName::named("MalContext").pointer(), "context")
}

fn trap(message: &str) -> Block {
    Block::new([Statement::call(
        "mal_trap",
        [Expr::identifier("context"), Expr::string(message)],
    )])
}

fn symbol(fields: impl IntoIterator<Item = Expr>) -> Expr {
    Expr::compound_literal(
        "MalType_Symbol",
        fields.into_iter().map(Initializer::positional),
    )
}

fn uint8(value: u8) -> Expr {
    Expr::named_call("UINT8_C", [Expr::number(value.to_string())])
}

fn uint64(value: u64) -> Expr {
    Expr::named_call("UINT64_C", [Expr::number(value.to_string())])
}

fn live_allocation_tracking() -> PreprocessorExpr {
    PreprocessorExpr::logical_or(
        PreprocessorExpr::defined("MAL_TEST_LIVE_ALLOCATION_LIMIT"),
        PreprocessorExpr::defined("MAL_TEST_REQUIRE_NO_LIVE_ALLOCATIONS"),
    )
}

fn append_function(output: &mut TranslationUnit, signature: FunctionSignature, body: Block) {
    output.push(FunctionDefinition::from_signature(signature, body));
    output.blank_line();
}
