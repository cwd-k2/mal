use crate::c_emit::syntax::{
    AggregateDefinition, AggregateField, Block, Directive, Expr, FunctionDefinition,
    FunctionSignature, Initializer, Parameter, PreprocessorExpr, Statement, TranslationUnit,
    TypeName,
};

pub(super) fn emit() -> TranslationUnit {
    let mut output = TranslationUnit::default();
    output.push(AggregateDefinition::typedef_structure(
        Some("MalAllocation".into()),
        [AggregateField::variable(
            TypeName::structure("MalAllocation").pointer(),
            "next",
        )],
        "MalAllocation",
    ));
    output.blank_line();
    output.push(AggregateDefinition::structure(
        "MalContext",
        [AggregateField::variable(
            TypeName::named("MalAllocation").pointer(),
            "allocations",
        )],
    ));
    output.blank_line();

    append_function(
        &mut output,
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
            call_statement(
                "fputs",
                [Expr::string("mal trap: "), Expr::identifier("stderr")],
            ),
            call_statement(
                "fputs",
                [Expr::identifier("message"), Expr::identifier("stderr")],
            ),
            call_statement("fputc", [Expr::character('\n'), Expr::identifier("stderr")]),
            call_statement("abort", []),
        ]),
    );

    append_function(
        &mut output,
        FunctionSignature::static_function("void", "mal_context_destroy", [context_parameter()]),
        Block::new([
            Statement::variable(
                TypeName::named("MalAllocation").pointer(),
                "allocation",
                Some(Expr::identifier("context").pointer_field("allocations")),
            ),
            Statement::while_loop(
                Expr::not_equal(Expr::identifier("allocation"), Expr::identifier("NULL")),
                Block::new([
                    Statement::variable(
                        TypeName::named("MalAllocation").pointer(),
                        "next",
                        Some(Expr::identifier("allocation").pointer_field("next")),
                    ),
                    call_statement("free", [Expr::identifier("allocation")]),
                    Statement::assignment(Expr::identifier("allocation"), Expr::identifier("next")),
                ]),
            ),
        ]),
    );

    append_function(
        &mut output,
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
                Expr::identifier("allocation").pointer_field("next"),
                Expr::identifier("context").pointer_field("allocations"),
            ),
            Statement::assignment(
                Expr::identifier("context").pointer_field("allocations"),
                Expr::identifier("allocation"),
            ),
            Statement::return_value(Expr::add(Expr::identifier("allocation"), Expr::number("1"))),
        ]),
    );

    append_function(
        &mut output,
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
                Expr::equal(
                    Expr::identifier("length"),
                    Expr::named_call("UINT64_C", [Expr::number("0")]),
                ),
                Block::new([Statement::return_value(symbol([
                    Expr::identifier("NULL"),
                    Expr::named_call("UINT64_C", [Expr::number("0")]),
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
            call_statement(
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
            ])),
        ]),
    );

    output
}

fn context_parameter() -> Parameter {
    Parameter::named(TypeName::named("MalContext").pointer(), "context")
}

fn trap(message: &str) -> Block {
    Block::new([call_statement(
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

fn call_statement(name: &str, arguments: impl IntoIterator<Item = Expr>) -> Statement {
    Statement::expression(Expr::named_call(name, arguments))
}

fn append_function(output: &mut TranslationUnit, signature: FunctionSignature, body: Block) {
    output.push(FunctionDefinition::from_signature(signature, body));
    output.blank_line();
}
