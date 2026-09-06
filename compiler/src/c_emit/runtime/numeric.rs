use crate::c_emit::scalar::{INTEGER_TYPES, IntegerType};
use crate::c_emit::syntax::{Block, Expr, FunctionDefinition, Statement};

pub(super) fn emit_float_to_integer(needs: u32) -> String {
    let mut output = String::new();
    for source_index in 0..2 {
        let (source_name, source_type, precision, literal_suffix) = if source_index == 0 {
            ("f32", "float", 24, "f")
        } else {
            ("f64", "double", 53, "")
        };
        for target in INTEGER_TYPES {
            if needs & (1_u32 << (source_index * 8 + target.index)) == 0 {
                continue;
            }
            let target_name = target.name;
            let target_type = target.c_type;
            let bits = target.width;
            let upper_exponent = if target.signed() { bits - 1 } else { bits };
            let upper = Expr::literal(format!("0x1p{upper_exponent}{literal_suffix}"));
            let lower = if target.signed() && bits <= precision {
                Expr::binary(
                    ">",
                    Expr::identifier("value"),
                    Expr::binary(
                        "-",
                        Expr::unary(
                            "-",
                            Expr::literal(format!("0x1p{}{literal_suffix}", bits - 1)),
                        ),
                        Expr::literal(format!("1.0{literal_suffix}")),
                    ),
                )
            } else if target.signed() {
                Expr::binary(
                    ">=",
                    Expr::identifier("value"),
                    Expr::unary(
                        "-",
                        Expr::literal(format!("0x1p{}{literal_suffix}", bits - 1)),
                    ),
                )
            } else {
                Expr::binary(
                    ">",
                    Expr::identifier("value"),
                    Expr::unary("-", Expr::literal(format!("1.0{literal_suffix}"))),
                )
            };
            append_function(
                &mut output,
                format!(
                    "static inline {target_type} mal_{source_name}_to_{target_name}(MalContext *context, {source_type} value)"
                ),
                Block::new([
                    Statement::if_then(
                        Expr::unary(
                            "!",
                            Expr::binary(
                                "&&",
                                lower,
                                Expr::binary("<", Expr::identifier("value"), upper),
                            ),
                        ),
                        trap("float-to-integer conversion out of range"),
                    ),
                    Statement::return_value(Expr::cast(target_type, Expr::identifier("value"))),
                ]),
            );
        }
    }
    output
}

pub(super) fn emit_integer_wrap(needs: u16) -> String {
    let mut output = String::new();
    for integer in INTEGER_TYPES {
        if !integer.signed() || needs & integer.mask() == 0 {
            continue;
        }
        let name = integer.name;
        let c_type = integer.c_type;
        let unsigned = integer.unsigned;
        append_function(
            &mut output,
            format!("static {c_type} mal_{name}_from_{unsigned}({unsigned} bits)"),
            Block::new([
                Statement::declaration(format!("{c_type} value"), None),
                Statement::expression(Expr::named_call(
                    "memcpy",
                    [
                        Expr::unary("&", Expr::identifier("value")),
                        Expr::unary("&", Expr::identifier("bits")),
                        Expr::sizeof_type("value"),
                    ],
                )),
                Statement::return_value(Expr::identifier("value")),
            ]),
        );
    }
    output
}

pub(super) fn emit_integer_checked(
    needs: u16,
    operation: &str,
    symbol: &'static str,
    zero_message: &str,
) -> String {
    let mut output = String::new();
    let overflow_operation = if operation == "divide" {
        "division"
    } else {
        operation
    };
    for integer in INTEGER_TYPES {
        if needs & integer.mask() == 0 {
            continue;
        }
        let name = integer.name;
        let c_type = integer.c_type;
        let mut body = Block::default();
        body.push(Statement::if_then(
            Expr::binary(
                "==",
                Expr::identifier("divisor"),
                Expr::cast(c_type, Expr::literal("0")),
            ),
            trap(zero_message),
        ));
        if let Some(minimum) = integer.minimum {
            body.push(Statement::if_then(
                Expr::binary(
                    "&&",
                    Expr::binary(
                        "==",
                        Expr::identifier("dividend"),
                        Expr::identifier(minimum),
                    ),
                    Expr::binary(
                        "==",
                        Expr::identifier("divisor"),
                        Expr::cast(c_type, Expr::unary("-", Expr::literal("1"))),
                    ),
                ),
                trap(&format!("signed {overflow_operation} overflow")),
            ));
        }
        body.push(Statement::return_value(Expr::binary(
            symbol,
            Expr::identifier("dividend"),
            Expr::identifier("divisor"),
        )));
        append_function(
            &mut output,
            format!(
                "static inline {c_type} mal_{name}_{operation}(MalContext *context, {c_type} dividend, {c_type} divisor)"
            ),
            body,
        );
    }
    output
}

pub(super) fn emit_integer_shift(left_needs: u16, right_needs: u16) -> String {
    let mut output = String::new();
    for integer in INTEGER_TYPES {
        let mask = integer.mask();
        if (left_needs | right_needs) & mask == 0 {
            continue;
        }
        let name = integer.name;
        let c_type = integer.c_type;
        let unsigned = integer.unsigned;
        let result = if integer.signed() {
            Expr::named_call(
                format!("mal_{name}_from_{unsigned}"),
                [Expr::cast(unsigned, Expr::identifier("shifted"))],
            )
        } else {
            Expr::cast(c_type, Expr::identifier("shifted"))
        };
        if left_needs & mask != 0 {
            emit_shift_left(&mut output, integer, result.clone());
        }
        if right_needs & mask != 0 {
            if integer.signed() {
                emit_signed_shift_right(&mut output, integer, result);
            } else {
                emit_unsigned_shift_right(&mut output, integer);
            }
        }
    }
    output
}

fn emit_shift_left(output: &mut String, integer: IntegerType, result: Expr) {
    let name = integer.name;
    let c_type = integer.c_type;
    let unsigned = integer.unsigned;
    let carrier = integer.carrier;
    let width = integer.width;
    let range_check = if integer.signed() {
        Expr::binary(
            "||",
            Expr::binary("<", Expr::identifier("count"), Expr::literal("0")),
            Expr::binary(
                ">=",
                Expr::cast(carrier, Expr::identifier("count")),
                Expr::literal(width.to_string()),
            ),
        )
    } else {
        Expr::binary(
            ">=",
            Expr::cast(carrier, Expr::identifier("count")),
            Expr::literal(width.to_string()),
        )
    };
    append_function(
        output,
        format!(
            "static inline {c_type} mal_{name}_shift_left(MalContext *context, {c_type} value, {c_type} count)"
        ),
        Block::new([
            Statement::if_then(range_check, trap("shift count out of range")),
            Statement::declaration(
                format!("{carrier} shifted"),
                Some(Expr::binary(
                    "<<",
                    Expr::cast(carrier, Expr::cast(unsigned, Expr::identifier("value"))),
                    Expr::cast(carrier, Expr::identifier("count")),
                )),
            ),
            Statement::return_value(result),
        ]),
    );
}

fn emit_signed_shift_right(output: &mut String, integer: IntegerType, result: Expr) {
    let name = integer.name;
    let c_type = integer.c_type;
    let unsigned = integer.unsigned;
    let carrier = integer.carrier;
    let width = integer.width;
    let maximum = integer.maximum;
    append_function(
        output,
        format!(
            "static inline {c_type} mal_{name}_shift_right(MalContext *context, {c_type} value, {c_type} count)"
        ),
        Block::new([
            Statement::if_then(
                Expr::binary(
                    "||",
                    Expr::binary("<", Expr::identifier("count"), Expr::literal("0")),
                    Expr::binary(
                        ">=",
                        Expr::cast(carrier, Expr::identifier("count")),
                        Expr::literal(width.to_string()),
                    ),
                ),
                trap("shift count out of range"),
            ),
            Statement::if_then(
                Expr::binary("==", Expr::identifier("count"), Expr::literal("0")),
                Block::new([Statement::return_value(Expr::identifier("value"))]),
            ),
            Statement::declaration(
                format!("{carrier} bits"),
                Some(Expr::cast(
                    carrier,
                    Expr::cast(unsigned, Expr::identifier("value")),
                )),
            ),
            Statement::declaration(
                format!("{carrier} shifted"),
                Some(Expr::binary(
                    ">>",
                    Expr::identifier("bits"),
                    Expr::cast(carrier, Expr::identifier("count")),
                )),
            ),
            Statement::if_then(
                Expr::binary(
                    "!=",
                    Expr::binary(
                        "&",
                        Expr::identifier("bits"),
                        Expr::binary(
                            "<<",
                            Expr::cast(carrier, Expr::literal("1")),
                            Expr::literal((width - 1).to_string()),
                        ),
                    ),
                    Expr::literal("0"),
                ),
                Block::new([Statement::assignment(
                    Expr::identifier("shifted"),
                    Expr::binary(
                        "|",
                        Expr::identifier("shifted"),
                        Expr::binary(
                            "<<",
                            Expr::cast(carrier, Expr::identifier(maximum)),
                            Expr::binary(
                                "-",
                                Expr::literal(width.to_string()),
                                Expr::cast(carrier, Expr::identifier("count")),
                            ),
                        ),
                    ),
                )]),
            ),
            Statement::return_value(result),
        ]),
    );
}

fn emit_unsigned_shift_right(output: &mut String, integer: IntegerType) {
    let name = integer.name;
    let c_type = integer.c_type;
    let carrier = integer.carrier;
    let width = integer.width;
    append_function(
        output,
        format!(
            "static inline {c_type} mal_{name}_shift_right(MalContext *context, {c_type} value, {c_type} count)"
        ),
        Block::new([
            Statement::if_then(
                Expr::binary(
                    ">=",
                    Expr::cast(carrier, Expr::identifier("count")),
                    Expr::literal(width.to_string()),
                ),
                trap("shift count out of range"),
            ),
            Statement::return_value(Expr::cast(
                c_type,
                Expr::binary(
                    ">>",
                    Expr::cast(carrier, Expr::identifier("value")),
                    Expr::cast(carrier, Expr::identifier("count")),
                ),
            )),
        ]),
    );
}

fn trap(message: &str) -> Block {
    Block::new([Statement::expression(Expr::named_call(
        "mal_trap",
        [
            Expr::identifier("context"),
            Expr::literal(format!("\"{message}\"")),
        ],
    ))])
}

fn append_function(output: &mut String, signature: impl Into<String>, body: Block) {
    output.push_str(&FunctionDefinition::new(signature, body).render());
    output.push('\n');
}
