use crate::c_emit::syntax::{
    Block, Declaration, Directive, Expr, FunctionDefinition, FunctionSignature, Initializer,
    Parameter, PreprocessorExpr, Statement, TranslationUnit, TypeName,
};

use super::{allocation_for, trap};

pub(super) fn emit_support() -> TranslationUnit {
    let mut output = TranslationUnit::default();
    output.push(Declaration::function(signature("mal_symbol_rope_join")));
    output.blank_line();
    append(&mut output, height_definition());
    append(&mut output, node_definition());
    append(&mut output, balance_definition());
    append(&mut output, join_definition());
    output
}

fn height_definition() -> FunctionDefinition {
    definition(
        FunctionSignature::static_function(
            "uint8_t",
            "mal_symbol_rope_height",
            [Parameter::named("MalType_Symbol", "value")],
        ),
        Block::new([
            Statement::if_then(
                is_rope(Expr::identifier("value")),
                Block::new([Statement::return_value(
                    rope(Expr::identifier("value")).pointer_field("height"),
                )]),
            ),
            Statement::return_value(uint8(0)),
        ]),
    )
}

fn node_definition() -> FunctionDefinition {
    definition(
        signature("mal_symbol_rope_node"),
        Block::new([
            Statement::variable(
                TypeName::named("MalSymbolRope").pointer(),
                "rope",
                Some(Expr::cast(
                    TypeName::named("MalSymbolRope").pointer(),
                    Expr::named_call(
                        "mal_allocate",
                        [
                            Expr::identifier("context"),
                            Expr::sizeof_type("MalSymbolRope"),
                        ],
                    ),
                )),
            ),
            // Payload capacity is always at most SIZE_MAX - sizeof(MalAllocation),
            // leaving SIZE_MAX as a tag without enlarging every allocation header.
            Statement::assignment(
                allocation_for(Expr::identifier("rope")).pointer_field("capacity"),
                Expr::identifier("SIZE_MAX"),
            ),
            Statement::assignment(
                Expr::dereference(Expr::identifier("rope")),
                Expr::compound_literal(
                    "MalSymbolRope",
                    [
                        Initializer::designated("left", retain(Expr::identifier("left"))),
                        Initializer::designated("right", retain(Expr::identifier("right"))),
                        Initializer::designated("flattened", Expr::identifier("NULL")),
                        Initializer::designated("height", node_height()),
                    ],
                ),
            ),
            Statement::directive(Directive::If(PreprocessorExpr::defined(
                "MAL_TEST_VALIDATE_SYMBOLS",
            ))),
            Statement::if_then(
                Expr::logical_or(
                    Expr::logical_or(
                        Expr::equal(Expr::identifier("left").field("length"), Expr::number("0")),
                        Expr::equal(Expr::identifier("right").field("length"), Expr::number("0")),
                    ),
                    Expr::logical_or(
                        Expr::not_equal(
                            Expr::identifier("rope").pointer_field("flattened"),
                            Expr::identifier("NULL"),
                        ),
                        Expr::logical_or(
                            Expr::not_equal(
                                Expr::identifier("rope").pointer_field("height"),
                                node_height(),
                            ),
                            Expr::not_equal(
                                allocation_for(Expr::identifier("rope")).pointer_field("capacity"),
                                Expr::identifier("SIZE_MAX"),
                            ),
                        ),
                    ),
                ),
                trap("invalid Symbol rope node"),
            ),
            Statement::directive(Directive::Endif),
            Statement::return_value(symbol(
                Expr::add(
                    Expr::identifier("left").field("length"),
                    Expr::identifier("right").field("length"),
                ),
                Expr::identifier("rope"),
            )),
        ]),
    )
}

fn balance_definition() -> FunctionDefinition {
    let left_heavy = Block::new([
        Statement::variable(
            TypeName::const_named("MalSymbolRope").pointer(),
            "left_rope",
            Some(rope(Expr::identifier("left"))),
        ),
        Statement::if_else(
            Expr::greater_equal(
                height(Expr::identifier("left_rope").pointer_field("left")),
                height(Expr::identifier("left_rope").pointer_field("right")),
            ),
            single_right_rotation(),
            double_right_rotation(),
        ),
    ]);
    let right_heavy = Block::new([
        Statement::variable(
            TypeName::const_named("MalSymbolRope").pointer(),
            "right_rope",
            Some(rope(Expr::identifier("right"))),
        ),
        Statement::if_else(
            Expr::greater_equal(
                height(Expr::identifier("right_rope").pointer_field("right")),
                height(Expr::identifier("right_rope").pointer_field("left")),
            ),
            single_left_rotation(),
            double_left_rotation(),
        ),
    ]);
    definition(
        signature("mal_symbol_rope_balance"),
        Block::new([
            Statement::variable(
                "uint8_t",
                "left_height",
                Some(height(Expr::identifier("left"))),
            ),
            Statement::variable(
                "uint8_t",
                "right_height",
                Some(height(Expr::identifier("right"))),
            ),
            Statement::if_then(
                Expr::greater(
                    Expr::identifier("left_height"),
                    Expr::add(Expr::identifier("right_height"), uint8(1)),
                ),
                left_heavy,
            ),
            Statement::if_then(
                Expr::greater(
                    Expr::identifier("right_height"),
                    Expr::add(Expr::identifier("left_height"), uint8(1)),
                ),
                right_heavy,
            ),
            Statement::return_value(node(Expr::identifier("left"), Expr::identifier("right"))),
        ]),
    )
}

fn single_right_rotation() -> Block {
    finish_rotation(
        [
            (
                "inner",
                node(
                    Expr::identifier("left_rope").pointer_field("right"),
                    Expr::identifier("right"),
                ),
            ),
            (
                "result",
                node(
                    Expr::identifier("left_rope").pointer_field("left"),
                    Expr::identifier("inner"),
                ),
            ),
        ],
        ["inner"],
    )
}

fn double_right_rotation() -> Block {
    let middle = Expr::identifier("middle");
    finish_rotation_with_prefix(
        Statement::variable(
            TypeName::const_named("MalSymbolRope").pointer(),
            "middle",
            Some(rope(Expr::identifier("left_rope").pointer_field("right"))),
        ),
        [
            (
                "new_left",
                node(
                    Expr::identifier("left_rope").pointer_field("left"),
                    middle.clone().pointer_field("left"),
                ),
            ),
            (
                "new_right",
                node(middle.pointer_field("right"), Expr::identifier("right")),
            ),
            (
                "result",
                node(Expr::identifier("new_left"), Expr::identifier("new_right")),
            ),
        ],
        ["new_right", "new_left"],
    )
}

fn single_left_rotation() -> Block {
    finish_rotation(
        [
            (
                "inner",
                node(
                    Expr::identifier("left"),
                    Expr::identifier("right_rope").pointer_field("left"),
                ),
            ),
            (
                "result",
                node(
                    Expr::identifier("inner"),
                    Expr::identifier("right_rope").pointer_field("right"),
                ),
            ),
        ],
        ["inner"],
    )
}

fn double_left_rotation() -> Block {
    let middle = Expr::identifier("middle");
    finish_rotation_with_prefix(
        Statement::variable(
            TypeName::const_named("MalSymbolRope").pointer(),
            "middle",
            Some(rope(Expr::identifier("right_rope").pointer_field("left"))),
        ),
        [
            (
                "new_left",
                node(
                    Expr::identifier("left"),
                    middle.clone().pointer_field("left"),
                ),
            ),
            (
                "new_right",
                node(
                    middle.pointer_field("right"),
                    Expr::identifier("right_rope").pointer_field("right"),
                ),
            ),
            (
                "result",
                node(Expr::identifier("new_left"), Expr::identifier("new_right")),
            ),
        ],
        ["new_right", "new_left"],
    )
}

fn finish_rotation<const N: usize, const D: usize>(
    values: [(&str, Expr); N],
    drops: [&str; D],
) -> Block {
    let mut statements = Vec::new();
    append_rotation(&mut statements, values, drops);
    Block::new(statements)
}

fn finish_rotation_with_prefix<const N: usize, const D: usize>(
    prefix: Statement,
    values: [(&str, Expr); N],
    drops: [&str; D],
) -> Block {
    let mut statements = vec![prefix];
    append_rotation(&mut statements, values, drops);
    Block::new(statements)
}

fn append_rotation<const N: usize, const D: usize>(
    statements: &mut Vec<Statement>,
    values: [(&str, Expr); N],
    drops: [&str; D],
) {
    for (name, value) in values {
        statements.push(Statement::variable("MalType_Symbol", name, Some(value)));
    }
    for name in drops {
        statements.push(Statement::call(
            "mal_symbol_release",
            [Expr::identifier(name)],
        ));
    }
    statements.push(Statement::return_value(Expr::identifier("result")));
}

fn join_definition() -> FunctionDefinition {
    let recurse_left = Block::new([
        Statement::variable(
            TypeName::const_named("MalSymbolRope").pointer(),
            "rope",
            Some(rope(Expr::identifier("left"))),
        ),
        Statement::variable(
            "MalType_Symbol",
            "joined",
            Some(join(
                Expr::identifier("rope").pointer_field("right"),
                Expr::identifier("right"),
            )),
        ),
        Statement::variable(
            "MalType_Symbol",
            "result",
            Some(balance(
                Expr::identifier("rope").pointer_field("left"),
                Expr::identifier("joined"),
            )),
        ),
        Statement::call("mal_symbol_release", [Expr::identifier("joined")]),
        Statement::return_value(Expr::identifier("result")),
    ]);
    let recurse_right = Block::new([
        Statement::variable(
            TypeName::const_named("MalSymbolRope").pointer(),
            "rope",
            Some(rope(Expr::identifier("right"))),
        ),
        Statement::variable(
            "MalType_Symbol",
            "joined",
            Some(join(
                Expr::identifier("left"),
                Expr::identifier("rope").pointer_field("left"),
            )),
        ),
        Statement::variable(
            "MalType_Symbol",
            "result",
            Some(balance(
                Expr::identifier("joined"),
                Expr::identifier("rope").pointer_field("right"),
            )),
        ),
        Statement::call("mal_symbol_release", [Expr::identifier("joined")]),
        Statement::return_value(Expr::identifier("result")),
    ]);
    definition(
        signature("mal_symbol_rope_join"),
        Block::new([
            Statement::variable(
                "uint8_t",
                "left_height",
                Some(height(Expr::identifier("left"))),
            ),
            Statement::variable(
                "uint8_t",
                "right_height",
                Some(height(Expr::identifier("right"))),
            ),
            Statement::if_then(
                Expr::greater(
                    Expr::identifier("left_height"),
                    Expr::add(Expr::identifier("right_height"), uint8(1)),
                ),
                recurse_left,
            ),
            Statement::if_then(
                Expr::greater(
                    Expr::identifier("right_height"),
                    Expr::add(Expr::identifier("left_height"), uint8(1)),
                ),
                recurse_right,
            ),
            Statement::return_value(node(Expr::identifier("left"), Expr::identifier("right"))),
        ]),
    )
}

fn signature(name: &str) -> FunctionSignature {
    FunctionSignature::static_function(
        "MalType_Symbol",
        name,
        [
            Parameter::named(TypeName::named("MalContext").pointer(), "context"),
            Parameter::named("MalType_Symbol", "left"),
            Parameter::named("MalType_Symbol", "right"),
        ],
    )
}

fn definition(signature: FunctionSignature, body: Block) -> FunctionDefinition {
    FunctionDefinition::from_signature(signature, body)
}

fn append(output: &mut TranslationUnit, definition: FunctionDefinition) {
    output.push(definition);
    output.blank_line();
}

fn is_rope(value: Expr) -> Expr {
    Expr::logical_and(
        Expr::not_equal(value.clone().field("ownership"), Expr::identifier("NULL")),
        Expr::equal(
            allocation_for(value.field("ownership")).pointer_field("capacity"),
            Expr::identifier("SIZE_MAX"),
        ),
    )
}

fn rope(value: Expr) -> Expr {
    Expr::cast(
        TypeName::const_named("MalSymbolRope").pointer(),
        value.field("ownership"),
    )
}

fn height(value: Expr) -> Expr {
    Expr::named_call("mal_symbol_rope_height", [value])
}

fn node_height() -> Expr {
    Expr::add(
        Expr::conditional(
            Expr::greater(
                height(Expr::identifier("left")),
                height(Expr::identifier("right")),
            ),
            height(Expr::identifier("left")),
            height(Expr::identifier("right")),
        ),
        uint8(1),
    )
}

fn retain(value: Expr) -> Expr {
    Expr::named_call("mal_symbol_retain", [Expr::identifier("context"), value])
}

fn node(left: Expr, right: Expr) -> Expr {
    Expr::named_call(
        "mal_symbol_rope_node",
        [Expr::identifier("context"), left, right],
    )
}

fn balance(left: Expr, right: Expr) -> Expr {
    Expr::named_call(
        "mal_symbol_rope_balance",
        [Expr::identifier("context"), left, right],
    )
}

fn join(left: Expr, right: Expr) -> Expr {
    Expr::named_call(
        "mal_symbol_rope_join",
        [Expr::identifier("context"), left, right],
    )
}

fn symbol(length: Expr, ownership: Expr) -> Expr {
    Expr::compound_literal(
        "MalType_Symbol",
        [
            Initializer::positional(Expr::identifier("NULL")),
            Initializer::positional(length),
            Initializer::positional(ownership),
        ],
    )
}

fn uint8(value: u8) -> Expr {
    Expr::named_call("UINT8_C", [Expr::number(value.to_string())])
}
