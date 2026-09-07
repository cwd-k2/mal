use crate::c_emit::syntax::{
    Block, Expr, FunctionDefinition, FunctionSignature, Parameter, Statement, TranslationUnit,
    TypeName,
};

mod concatenate;

pub(super) fn emit_concatenate(consume_left: bool, consume_right: bool) -> FunctionDefinition {
    concatenate::emit(consume_left, consume_right)
}

pub(super) fn emit_rope_support() -> crate::c_emit::syntax::TranslationUnit {
    concatenate::emit_rope_support()
}

pub(super) fn emit_equality() -> TranslationUnit {
    let mut output = TranslationUnit::default();
    output.push(function(
        FunctionSignature::static_noinline(
            "uint8_t",
            "mal_symbol_equal_slow",
            [
                context_parameter(),
                Parameter::named("MalType_Symbol", "left"),
                Parameter::named("MalType_Symbol", "right"),
            ],
        ),
        Block::new([
            materialize_if_needed("left"),
            materialize_if_needed("right"),
            compare_bytes(),
        ]),
    ));
    output.blank_line();
    output.push(function(
        FunctionSignature::static_function(
            "uint8_t",
            "mal_symbol_equal",
            [
                context_parameter(),
                Parameter::named("MalType_Symbol", "left"),
                Parameter::named("MalType_Symbol", "right"),
            ],
        ),
        Block::new([
            Statement::if_then(
                Expr::not_equal(
                    Expr::identifier("left").field("length"),
                    Expr::identifier("right").field("length"),
                ),
                Block::new([Statement::return_value(uint8(0))]),
            ),
            Statement::if_then(
                Expr::equal(Expr::identifier("left").field("length"), Expr::number("0")),
                Block::new([Statement::return_value(uint8(1))]),
            ),
            Statement::if_then(
                Expr::logical_and(has_data("left"), has_data("right")),
                Block::new([compare_bytes()]),
            ),
            Statement::return_value(Expr::named_call(
                "mal_symbol_equal_slow",
                [
                    Expr::identifier("context"),
                    Expr::identifier("left"),
                    Expr::identifier("right"),
                ],
            )),
        ]),
    ));
    output
}

pub(super) fn emit_at() -> TranslationUnit {
    // Index validity belongs to the source operation contract; materializing
    // the language-owned representation remains a runtime responsibility.
    let mut output = TranslationUnit::default();
    output.push(function(
        FunctionSignature::static_noinline(
            "uint8_t",
            "mal_symbol_at_slow",
            [
                context_parameter(),
                Parameter::named("MalType_Symbol", "value"),
                Parameter::named("uint64_t", "index"),
            ],
        ),
        Block::new([
            Statement::assignment(
                Expr::identifier("value"),
                materialize(Expr::identifier("value")),
            ),
            byte_at("value"),
        ]),
    ));
    output.blank_line();
    output.push(function(
        FunctionSignature::static_function(
            "uint8_t",
            "mal_symbol_at",
            [
                context_parameter(),
                Parameter::named("MalType_Symbol", "value"),
                Parameter::named("uint64_t", "index"),
            ],
        ),
        Block::new([
            Statement::if_then(has_data("value"), Block::new([byte_at("value")])),
            Statement::return_value(Expr::named_call(
                "mal_symbol_at_slow",
                [
                    Expr::identifier("context"),
                    Expr::identifier("value"),
                    Expr::identifier("index"),
                ],
            )),
        ]),
    ));
    output
}

fn context_parameter() -> Parameter {
    Parameter::named(TypeName::named("MalContext").pointer(), "context")
}

fn uint8(value: u8) -> Expr {
    Expr::named_call("UINT8_C", [Expr::number(value.to_string())])
}

fn materialize(value: Expr) -> Expr {
    Expr::named_call(
        "mal_symbol_materialize",
        [Expr::identifier("context"), value],
    )
}

fn materialize_if_needed(name: &str) -> Statement {
    Statement::if_then(
        Expr::logical_not(has_data(name)),
        Block::new([Statement::assignment(
            Expr::identifier(name),
            materialize(Expr::identifier(name)),
        )]),
    )
}

fn has_data(name: &str) -> Expr {
    Expr::not_equal(
        Expr::identifier(name).field("data"),
        Expr::identifier("NULL"),
    )
}

fn compare_bytes() -> Statement {
    Statement::return_value(Expr::cast(
        "uint8_t",
        Expr::equal(
            Expr::named_call(
                "memcmp",
                [
                    Expr::identifier("left").field("data"),
                    Expr::identifier("right").field("data"),
                    Expr::cast("size_t", Expr::identifier("left").field("length")),
                ],
            ),
            Expr::number("0"),
        ),
    ))
}

fn byte_at(name: &str) -> Statement {
    Statement::return_value(
        Expr::identifier(name)
            .field("data")
            .index(Expr::identifier("index")),
    )
}

fn function(signature: FunctionSignature, body: Block) -> FunctionDefinition {
    FunctionDefinition::from_signature(signature, body)
}
