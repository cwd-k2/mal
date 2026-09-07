use crate::c_emit::syntax::{
    Block, Expr, FunctionDefinition, FunctionSignature, Parameter, Statement, TypeName,
};

mod concatenate;

pub(super) fn emit_concatenate(consume_left: bool, consume_right: bool) -> FunctionDefinition {
    concatenate::emit(consume_left, consume_right)
}

pub(super) fn emit_rope_support() -> crate::c_emit::syntax::TranslationUnit {
    concatenate::emit_rope_support()
}

pub(super) fn emit_equality() -> FunctionDefinition {
    function(
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
            Statement::assignment(
                Expr::identifier("left"),
                materialize(Expr::identifier("left")),
            ),
            Statement::assignment(
                Expr::identifier("right"),
                materialize(Expr::identifier("right")),
            ),
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
            )),
        ]),
    )
}

pub(super) fn emit_at() -> FunctionDefinition {
    function(
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
            Statement::assignment(
                Expr::identifier("value"),
                materialize(Expr::identifier("value")),
            ),
            Statement::return_value(
                Expr::identifier("value")
                    .field("data")
                    .index(Expr::identifier("index")),
            ),
        ]),
    )
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

fn function(signature: FunctionSignature, body: Block) -> FunctionDefinition {
    FunctionDefinition::from_signature(signature, body)
}
