use crate::c_emit::syntax::{
    AggregateDefinition, AggregateField, Block, Expr, FunctionDefinition, FunctionSignature,
    Parameter, Statement, TranslationUnit, TypeName,
};

mod index;

pub(super) fn emit(include_index_access: bool) -> TranslationUnit {
    let mut output = TranslationUnit::default();
    output.push(AggregateDefinition::typedef_structure(
        None,
        [
            AggregateField::array(
                TypeName::const_named("MalSymbolRope").pointer(),
                "pending",
                Expr::add(Expr::identifier("UINT8_MAX"), Expr::number("1")),
            ),
            AggregateField::variable("size_t", "depth"),
            AggregateField::variable("MalType_Symbol", "leaf"),
            AggregateField::variable("uint64_t", "index"),
            AggregateField::variable("uint64_t", "position"),
            AggregateField::variable("uint8_t", "initialized"),
        ],
        "MalSymbolLeafCursor",
    ));
    output.blank_line();
    append(&mut output, descend_definition());
    append(&mut output, advance_definition());
    if include_index_access {
        output.extend(index::emit());
    }
    output
}

fn descend_definition() -> FunctionDefinition {
    function(
        FunctionSignature::static_function(
            "void",
            "mal_symbol_leaf_cursor_descend",
            [
                Parameter::named(TypeName::named("MalSymbolLeafCursor").pointer(), "cursor"),
                Parameter::named("MalType_Symbol", "value"),
            ],
        ),
        Block::new([Statement::if_else(
            has_data("value"),
            Block::new([
                Statement::assignment(
                    Expr::identifier("cursor").pointer_field("leaf"),
                    Expr::identifier("value"),
                ),
                Statement::assignment(
                    Expr::identifier("cursor").pointer_field("index"),
                    Expr::number("0"),
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
                    "mal_symbol_leaf_cursor_descend",
                    [
                        Expr::identifier("cursor"),
                        Expr::identifier("rope").pointer_field("left"),
                    ],
                ),
            ]),
        )]),
    )
}

fn advance_definition() -> FunctionDefinition {
    function(
        FunctionSignature::static_function(
            "void",
            "mal_symbol_leaf_cursor_advance",
            [
                Parameter::named(TypeName::named("MalSymbolLeafCursor").pointer(), "cursor"),
                Parameter::named("uint64_t", "count"),
            ],
        ),
        Block::new([
            Statement::assignment(
                Expr::identifier("cursor").pointer_field("index"),
                Expr::add(
                    Expr::identifier("cursor").pointer_field("index"),
                    Expr::identifier("count"),
                ),
            ),
            Statement::if_then(
                Expr::logical_and(
                    Expr::equal(
                        Expr::identifier("cursor").pointer_field("index"),
                        Expr::identifier("cursor")
                            .pointer_field("leaf")
                            .field("length"),
                    ),
                    Expr::not_equal(
                        Expr::identifier("cursor").pointer_field("depth"),
                        Expr::number("0"),
                    ),
                ),
                Block::new([
                    Statement::assignment(
                        Expr::identifier("cursor").pointer_field("depth"),
                        Expr::subtract(
                            Expr::identifier("cursor").pointer_field("depth"),
                            Expr::number("1"),
                        ),
                    ),
                    Statement::variable(
                        TypeName::const_named("MalSymbolRope").pointer(),
                        "rope",
                        Some(
                            Expr::identifier("cursor")
                                .pointer_field("pending")
                                .index(Expr::identifier("cursor").pointer_field("depth")),
                        ),
                    ),
                    Statement::call(
                        "mal_symbol_leaf_cursor_descend",
                        [
                            Expr::identifier("cursor"),
                            Expr::identifier("rope").pointer_field("right"),
                        ],
                    ),
                ]),
            ),
        ]),
    )
}

fn has_data(name: &str) -> Expr {
    Expr::not_equal(
        Expr::identifier(name).field("data"),
        Expr::identifier("NULL"),
    )
}

fn function(signature: FunctionSignature, body: Block) -> FunctionDefinition {
    FunctionDefinition::from_signature(signature, body)
}

fn append(output: &mut TranslationUnit, definition: FunctionDefinition) {
    output.push(definition);
    output.blank_line();
}
