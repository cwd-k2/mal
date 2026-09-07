use crate::c_emit::syntax::{
    Block, Expr, FunctionDefinition, FunctionSignature, Initializer, Parameter, Statement, TypeName,
};

mod capacity;

pub(super) fn emit(consume_left: bool) -> FunctionDefinition {
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
    let empty_left = if consume_left {
        Block::new([
            Statement::variable(
                "MalType_Symbol",
                "result",
                Some(Expr::named_call(
                    "mal_symbol_retain",
                    [Expr::identifier("context"), Expr::identifier("right")],
                )),
            ),
            Statement::call("mal_symbol_release", [Expr::identifier("left")]),
            Statement::return_value(Expr::identifier("result")),
        ])
    } else {
        Block::new([Statement::return_value(Expr::named_call(
            "mal_symbol_retain",
            [Expr::identifier("context"), Expr::identifier("right")],
        ))])
    };
    let mut body = Block::new([
        Statement::if_then(
            Expr::equal(Expr::identifier("left").field("length"), uint64(0)),
            empty_left,
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
    ]);
    if consume_left {
        // Any separately live right alias owns another share, so uniqueness also
        // keeps right.data valid if realloc moves the left allocation.
        let allocation = capacity::allocation_for(Expr::identifier("left").field("ownership"));
        body.push(Statement::if_then(
            Expr::logical_and(
                Expr::not_equal(
                    Expr::identifier("left").field("ownership"),
                    Expr::identifier("NULL"),
                ),
                Expr::equal(allocation.clone().pointer_field("references"), uint64(1)),
            ),
            capacity::unique_concatenation(allocation),
        ));
    }
    body.push(Statement::variable(
        TypeName::named("uint8_t").pointer(),
        "bytes",
        Some(Expr::cast(
            TypeName::named("uint8_t").pointer(),
            Expr::named_call(
                "mal_allocate",
                [Expr::identifier("context"), Expr::identifier("size")],
            ),
        )),
    ));
    body.push(Statement::call(
        "memcpy",
        [
            Expr::identifier("bytes"),
            Expr::identifier("left").field("data"),
            Expr::cast("size_t", Expr::identifier("left").field("length")),
        ],
    ));
    body.push(append_right_bytes());
    if consume_left {
        body.push(Statement::call(
            "mal_symbol_release",
            [Expr::identifier("left")],
        ));
    }
    body.push(Statement::return_value(symbol_result()));
    FunctionDefinition::from_signature(
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

pub(super) fn append_right_bytes() -> Statement {
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
    )
}

pub(super) fn symbol_result() -> Expr {
    Expr::compound_literal(
        "MalType_Symbol",
        [
            Initializer::positional(Expr::identifier("bytes")),
            Initializer::positional(Expr::identifier("length")),
            Initializer::positional(Expr::identifier("bytes")),
        ],
    )
}

pub(super) fn trap(message: &str) -> Block {
    Block::new([Statement::call(
        "mal_trap",
        [Expr::identifier("context"), Expr::string(message)],
    )])
}

fn context_parameter() -> Parameter {
    Parameter::named(TypeName::named("MalContext").pointer(), "context")
}

fn uint64(value: u64) -> Expr {
    Expr::named_call("UINT64_C", [Expr::number(value.to_string())])
}
