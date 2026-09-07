use crate::c_emit::syntax::{
    Block, Expr, ForInitializer, FunctionDefinition, FunctionSignature, Parameter, Statement,
    TypeName,
};

mod concatenate;

pub(super) fn emit_concatenate(consume_left: bool) -> FunctionDefinition {
    concatenate::emit(consume_left)
}

pub(super) fn emit_equality() -> FunctionDefinition {
    function(
        FunctionSignature::static_function(
            "uint8_t",
            "mal_symbol_equal",
            [
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
            Statement::for_loop(
                ForInitializer::variable("uint64_t", "index", uint64(0)),
                Expr::less(
                    Expr::identifier("index"),
                    Expr::identifier("left").field("length"),
                ),
                Expr::pre_increment(Expr::identifier("index")),
                Block::new([Statement::if_then(
                    Expr::not_equal(
                        Expr::identifier("left")
                            .field("data")
                            .index(Expr::identifier("index")),
                        Expr::identifier("right")
                            .field("data")
                            .index(Expr::identifier("index")),
                    ),
                    Block::new([Statement::return_value(uint8(0))]),
                )]),
            ),
            Statement::return_value(uint8(1)),
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
            Statement::if_then(
                Expr::greater_equal(
                    Expr::identifier("index"),
                    Expr::identifier("value").field("length"),
                ),
                trap("Symbol index out of range"),
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

fn trap(message: &str) -> Block {
    Block::new([Statement::call(
        "mal_trap",
        [Expr::identifier("context"), Expr::string(message)],
    )])
}

fn uint8(value: u8) -> Expr {
    Expr::named_call("UINT8_C", [Expr::number(value.to_string())])
}

fn uint64(value: u64) -> Expr {
    Expr::named_call("UINT64_C", [Expr::number(value.to_string())])
}

fn function(signature: FunctionSignature, body: Block) -> FunctionDefinition {
    FunctionDefinition::from_signature(signature, body)
}
