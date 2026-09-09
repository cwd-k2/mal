use crate::c_emit::syntax::{
    Block, Declaration, Directive, Expr, FunctionDefinition, FunctionSignature, Parameter,
    PreprocessorExpr, Statement, TranslationUnit, TypeName, VariableDeclaration,
};

pub(super) fn emit() -> TranslationUnit {
    let mut output = TranslationUnit::default();
    output.push(Directive::If(PreprocessorExpr::defined(
        "MAL_TEST_SYMBOL_CURSOR_SEEK_LIMIT",
    )));
    output.push(Declaration::variable(VariableDeclaration::static_variable(
        "size_t",
        "mal_test_symbol_cursor_seek_count",
    )));
    output.push(Directive::Endif);
    output.blank_line();
    append(&mut output, seek_definition());
    append(&mut output, at_definition());
    output
}

fn seek_definition() -> FunctionDefinition {
    let mut body = seek_limit_check();
    body.push(Statement::if_else(
        has_data("value"),
        Block::new([
            Statement::assignment(
                Expr::identifier("cursor").pointer_field("leaf"),
                Expr::identifier("value"),
            ),
            Statement::assignment(
                Expr::identifier("cursor").pointer_field("index"),
                Expr::identifier("index"),
            ),
        ]),
        Block::new([
            Statement::variable(
                TypeName::const_named("MalSymbolRope").pointer(),
                "rope",
                Some(Expr::cast(
                    TypeName::const_named("MalSymbolRope").pointer(),
                    Expr::identifier("value").field("ownership"),
                )),
            ),
            Statement::if_else(
                Expr::less(
                    Expr::identifier("index"),
                    Expr::identifier("rope")
                        .pointer_field("left")
                        .field("length"),
                ),
                Block::new([
                    Statement::assignment(
                        Expr::identifier("cursor")
                            .pointer_field("pending")
                            .index(Expr::identifier("cursor").pointer_field("depth")),
                        Expr::identifier("rope"),
                    ),
                    Statement::expression(Expr::pre_increment(
                        Expr::identifier("cursor").pointer_field("depth"),
                    )),
                    Statement::call(
                        "mal_symbol_leaf_cursor_seek",
                        [
                            Expr::identifier("cursor"),
                            Expr::identifier("rope").pointer_field("left"),
                            Expr::identifier("index"),
                        ],
                    ),
                ]),
                Block::new([Statement::call(
                    "mal_symbol_leaf_cursor_seek",
                    [
                        Expr::identifier("cursor"),
                        Expr::identifier("rope").pointer_field("right"),
                        Expr::subtract(
                            Expr::identifier("index"),
                            Expr::identifier("rope")
                                .pointer_field("left")
                                .field("length"),
                        ),
                    ],
                )]),
            ),
        ]),
    ));
    FunctionDefinition::from_signature(
        FunctionSignature::static_function(
            "void",
            "mal_symbol_leaf_cursor_seek",
            [
                Parameter::named(TypeName::named("MalSymbolLeafCursor").pointer(), "cursor"),
                Parameter::named("MalType_Symbol", "value"),
                Parameter::named("uint64_t", "index"),
            ],
        ),
        body,
    )
}

fn at_definition() -> FunctionDefinition {
    FunctionDefinition::from_signature(
        FunctionSignature::static_function(
            "uint8_t",
            "mal_symbol_leaf_cursor_at",
            [
                Parameter::named(TypeName::named("MalSymbolLeafCursor").pointer(), "cursor"),
                Parameter::named("MalType_Symbol", "value"),
                Parameter::named("uint64_t", "index"),
            ],
        ),
        Block::new([
            Statement::if_then(
                has_data("value"),
                Block::new([Statement::return_value(
                    Expr::identifier("value")
                        .field("data")
                        .index(Expr::identifier("index")),
                )]),
            ),
            Statement::if_else(needs_seek(), seek_cursor(), advance_cursor()),
            Statement::assignment(
                Expr::identifier("cursor").pointer_field("position"),
                Expr::identifier("index"),
            ),
            Statement::return_value(
                Expr::identifier("cursor")
                    .pointer_field("leaf")
                    .field("data")
                    .index(Expr::identifier("cursor").pointer_field("index")),
            ),
        ]),
    )
}

fn seek_limit_check() -> Block {
    Block::new([
        Statement::directive(Directive::If(PreprocessorExpr::defined(
            "MAL_TEST_SYMBOL_CURSOR_SEEK_LIMIT",
        ))),
        Statement::expression(Expr::pre_increment(Expr::identifier(
            "mal_test_symbol_cursor_seek_count",
        ))),
        Statement::if_then(
            Expr::greater(
                Expr::identifier("mal_test_symbol_cursor_seek_count"),
                Expr::cast(
                    "size_t",
                    Expr::identifier("MAL_TEST_SYMBOL_CURSOR_SEEK_LIMIT"),
                ),
            ),
            Block::new([Statement::call(
                "mal_resource_failure",
                [Expr::string("Symbol cursor seek limit exceeded")],
            )]),
        ),
        Statement::directive(Directive::Endif),
    ])
}

fn needs_seek() -> Expr {
    Expr::logical_or(
        Expr::equal(
            Expr::identifier("cursor").pointer_field("initialized"),
            Expr::number("0"),
        ),
        Expr::logical_or(
            Expr::less(
                Expr::identifier("index"),
                Expr::identifier("cursor").pointer_field("position"),
            ),
            Expr::greater(
                Expr::identifier("index"),
                Expr::add(
                    Expr::identifier("cursor").pointer_field("position"),
                    Expr::number("1"),
                ),
            ),
        ),
    )
}

fn seek_cursor() -> Block {
    Block::new([
        Statement::assignment(
            Expr::identifier("cursor").pointer_field("depth"),
            Expr::number("0"),
        ),
        Statement::call(
            "mal_symbol_leaf_cursor_seek",
            [
                Expr::identifier("cursor"),
                Expr::identifier("value"),
                Expr::identifier("index"),
            ],
        ),
        Statement::assignment(
            Expr::identifier("cursor").pointer_field("initialized"),
            Expr::named_call("UINT8_C", [Expr::number("1")]),
        ),
    ])
}

fn advance_cursor() -> Block {
    Block::new([Statement::if_then(
        Expr::not_equal(
            Expr::identifier("index"),
            Expr::identifier("cursor").pointer_field("position"),
        ),
        Block::new([Statement::call(
            "mal_symbol_leaf_cursor_advance",
            [Expr::identifier("cursor"), Expr::number("1")],
        )]),
    )])
}

fn has_data(name: &str) -> Expr {
    Expr::not_equal(
        Expr::identifier(name).field("data"),
        Expr::identifier("NULL"),
    )
}

fn append(output: &mut TranslationUnit, definition: FunctionDefinition) {
    output.push(definition);
    output.blank_line();
}
