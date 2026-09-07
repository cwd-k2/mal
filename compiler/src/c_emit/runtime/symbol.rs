use crate::c_emit::syntax::{
    Block, Expr, ForInitializer, FunctionDefinition, FunctionSignature, Initializer, Parameter,
    Statement, TypeName,
};

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

pub(super) fn emit_concatenate(consume_left: bool) -> FunctionDefinition {
    let name = if consume_left {
        "mal_symbol_concatenate_consuming_left"
    } else {
        "mal_symbol_concatenate"
    };
    let empty_right_result = if consume_left {
        Expr::identifier("left")
    } else {
        Expr::named_call(
            "mal_symbol_retain",
            [Expr::identifier("context"), Expr::identifier("left")],
        )
    };
    let mut body = Block::new([
        Statement::if_then(
            Expr::equal(Expr::identifier("left").field("length"), uint64(0)),
            Block::new([Statement::return_value(Expr::named_call(
                "mal_symbol_retain",
                [Expr::identifier("context"), Expr::identifier("right")],
            ))]),
        ),
        Statement::if_then(
            Expr::equal(Expr::identifier("right").field("length"), uint64(0)),
            Block::new([Statement::return_value(empty_right_result)]),
        ),
        Statement::if_then(
            Expr::greater(
                Expr::identifier("left").field("length"),
                Expr::subtract(
                    Expr::identifier("UINT64_MAX"),
                    Expr::identifier("right").field("length"),
                ),
            ),
            trap("Symbol length overflow"),
        ),
        Statement::variable(
            "uint64_t",
            "length",
            Some(Expr::add(
                Expr::identifier("left").field("length"),
                Expr::identifier("right").field("length"),
            )),
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
            "bytes",
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
                Expr::identifier("bytes"),
                Expr::identifier("left").field("data"),
                Expr::cast("size_t", Expr::identifier("left").field("length")),
            ],
        ),
        Statement::call(
            "memcpy",
            [
                Expr::add(
                    Expr::identifier("bytes"),
                    Expr::cast("size_t", Expr::identifier("left").field("length")),
                ),
                Expr::identifier("right").field("data"),
                Expr::cast("size_t", Expr::identifier("right").field("length")),
            ],
        ),
    ]);
    if consume_left {
        body.push(Statement::call(
            "mal_symbol_release",
            [Expr::identifier("left")],
        ));
    }
    body.push(Statement::return_value(Expr::compound_literal(
        "MalType_Symbol",
        [
            Initializer::positional(Expr::identifier("bytes")),
            Initializer::positional(Expr::identifier("length")),
            Initializer::positional(Expr::identifier("bytes")),
        ],
    )));
    function(
        FunctionSignature::static_function(
            "MalType_Symbol",
            name,
            [
                context_parameter(),
                Parameter::named("MalType_Symbol", "left"),
                Parameter::named("MalType_Symbol", "right"),
            ],
        ),
        body,
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
