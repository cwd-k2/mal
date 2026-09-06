use crate::c_emit::scalar::{INTEGER_TYPES, IntegerType};
use crate::c_emit::syntax::{
    BinaryOperator, Block, Expr, FunctionDefinition, FunctionSignature, Parameter, Statement,
    TranslationUnit, TypeName,
};

pub(super) fn emit_float_to_integer(needs: u32) -> TranslationUnit {
    let mut output = TranslationUnit::default();
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
            let upper = Expr::number(format!("0x1p{upper_exponent}{literal_suffix}"));
            let lower = if target.signed() && bits <= precision {
                Expr::greater(
                    Expr::identifier("value"),
                    Expr::subtract(
                        Expr::negate(Expr::number(format!("0x1p{}{literal_suffix}", bits - 1))),
                        Expr::number(format!("1.0{literal_suffix}")),
                    ),
                )
            } else if target.signed() {
                Expr::greater_equal(
                    Expr::identifier("value"),
                    Expr::negate(Expr::number(format!("0x1p{}{literal_suffix}", bits - 1))),
                )
            } else {
                Expr::greater(
                    Expr::identifier("value"),
                    Expr::negate(Expr::number(format!("1.0{literal_suffix}"))),
                )
            };
            append_function(
                &mut output,
                FunctionSignature::static_inline(
                    target_type,
                    format!("mal_{source_name}_to_{target_name}"),
                    [context_parameter(), Parameter::named(source_type, "value")],
                ),
                Block::new([
                    Statement::if_then(
                        Expr::logical_not(Expr::logical_and(
                            lower,
                            Expr::less(Expr::identifier("value"), upper),
                        )),
                        trap("float-to-integer conversion out of range"),
                    ),
                    Statement::return_value(Expr::cast(target_type, Expr::identifier("value"))),
                ]),
            );
        }
    }
    output
}

pub(super) fn emit_integer_wrap(needs: u16) -> TranslationUnit {
    let mut output = TranslationUnit::default();
    for integer in INTEGER_TYPES {
        if !integer.signed() || needs & integer.mask() == 0 {
            continue;
        }
        let name = integer.name;
        let c_type = integer.c_type;
        let unsigned = integer.unsigned;
        append_function(
            &mut output,
            FunctionSignature::static_function(
                c_type,
                format!("mal_{name}_from_{unsigned}"),
                [Parameter::named(unsigned, "bits")],
            ),
            Block::new([
                Statement::variable(c_type, "value", None),
                Statement::expression(Expr::named_call(
                    "memcpy",
                    [
                        Expr::address_of(Expr::identifier("value")),
                        Expr::address_of(Expr::identifier("bits")),
                        Expr::sizeof_expr(Expr::identifier("value")),
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
    operator: BinaryOperator,
    zero_message: &str,
) -> TranslationUnit {
    let mut output = TranslationUnit::default();
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
            Expr::equal(
                Expr::identifier("divisor"),
                Expr::cast(c_type, Expr::number("0")),
            ),
            trap(zero_message),
        ));
        if let Some(minimum) = integer.minimum {
            body.push(Statement::if_then(
                Expr::logical_and(
                    Expr::equal(Expr::identifier("dividend"), Expr::identifier(minimum)),
                    Expr::equal(
                        Expr::identifier("divisor"),
                        Expr::cast(c_type, Expr::negate(Expr::number("1"))),
                    ),
                ),
                trap(&format!("signed {overflow_operation} overflow")),
            ));
        }
        body.push(Statement::return_value(Expr::binary(
            operator,
            Expr::identifier("dividend"),
            Expr::identifier("divisor"),
        )));
        append_function(
            &mut output,
            FunctionSignature::static_inline(
                c_type,
                format!("mal_{name}_{operation}"),
                [
                    context_parameter(),
                    Parameter::named(c_type, "dividend"),
                    Parameter::named(c_type, "divisor"),
                ],
            ),
            body,
        );
    }
    output
}

pub(super) fn emit_integer_shift(left_needs: u16, right_needs: u16) -> TranslationUnit {
    let mut output = TranslationUnit::default();
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

fn emit_shift_left(output: &mut TranslationUnit, integer: IntegerType, result: Expr) {
    let name = integer.name;
    let c_type = integer.c_type;
    let unsigned = integer.unsigned;
    let carrier = integer.carrier;
    let width = integer.width;
    let range_check = if integer.signed() {
        Expr::logical_or(
            Expr::less(Expr::identifier("count"), Expr::number("0")),
            Expr::greater_equal(
                Expr::cast(carrier, Expr::identifier("count")),
                Expr::number(width.to_string()),
            ),
        )
    } else {
        Expr::greater_equal(
            Expr::cast(carrier, Expr::identifier("count")),
            Expr::number(width.to_string()),
        )
    };
    append_function(
        output,
        FunctionSignature::static_inline(
            c_type,
            format!("mal_{name}_shift_left"),
            [
                context_parameter(),
                Parameter::named(c_type, "value"),
                Parameter::named(c_type, "count"),
            ],
        ),
        Block::new([
            Statement::if_then(range_check, trap("shift count out of range")),
            Statement::variable(
                carrier,
                "shifted",
                Some(Expr::shift_left(
                    Expr::cast(carrier, Expr::cast(unsigned, Expr::identifier("value"))),
                    Expr::cast(carrier, Expr::identifier("count")),
                )),
            ),
            Statement::return_value(result),
        ]),
    );
}

fn emit_signed_shift_right(output: &mut TranslationUnit, integer: IntegerType, result: Expr) {
    let name = integer.name;
    let c_type = integer.c_type;
    let unsigned = integer.unsigned;
    let carrier = integer.carrier;
    let width = integer.width;
    let maximum = integer.maximum;
    append_function(
        output,
        FunctionSignature::static_inline(
            c_type,
            format!("mal_{name}_shift_right"),
            [
                context_parameter(),
                Parameter::named(c_type, "value"),
                Parameter::named(c_type, "count"),
            ],
        ),
        Block::new([
            Statement::if_then(
                Expr::logical_or(
                    Expr::less(Expr::identifier("count"), Expr::number("0")),
                    Expr::greater_equal(
                        Expr::cast(carrier, Expr::identifier("count")),
                        Expr::number(width.to_string()),
                    ),
                ),
                trap("shift count out of range"),
            ),
            Statement::if_then(
                Expr::equal(Expr::identifier("count"), Expr::number("0")),
                Block::new([Statement::return_value(Expr::identifier("value"))]),
            ),
            Statement::variable(
                carrier,
                "bits",
                Some(Expr::cast(
                    carrier,
                    Expr::cast(unsigned, Expr::identifier("value")),
                )),
            ),
            Statement::variable(
                carrier,
                "shifted",
                Some(Expr::shift_right(
                    Expr::identifier("bits"),
                    Expr::cast(carrier, Expr::identifier("count")),
                )),
            ),
            Statement::if_then(
                Expr::not_equal(
                    Expr::bitwise_and(
                        Expr::identifier("bits"),
                        Expr::shift_left(
                            Expr::cast(carrier, Expr::number("1")),
                            Expr::number((width - 1).to_string()),
                        ),
                    ),
                    Expr::number("0"),
                ),
                Block::new([Statement::assignment(
                    Expr::identifier("shifted"),
                    Expr::bitwise_or(
                        Expr::identifier("shifted"),
                        Expr::shift_left(
                            Expr::cast(carrier, Expr::identifier(maximum)),
                            Expr::subtract(
                                Expr::number(width.to_string()),
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

fn emit_unsigned_shift_right(output: &mut TranslationUnit, integer: IntegerType) {
    let name = integer.name;
    let c_type = integer.c_type;
    let carrier = integer.carrier;
    let width = integer.width;
    append_function(
        output,
        FunctionSignature::static_inline(
            c_type,
            format!("mal_{name}_shift_right"),
            [
                context_parameter(),
                Parameter::named(c_type, "value"),
                Parameter::named(c_type, "count"),
            ],
        ),
        Block::new([
            Statement::if_then(
                Expr::greater_equal(
                    Expr::cast(carrier, Expr::identifier("count")),
                    Expr::number(width.to_string()),
                ),
                trap("shift count out of range"),
            ),
            Statement::return_value(Expr::cast(
                c_type,
                Expr::shift_right(
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
        [Expr::identifier("context"), Expr::string(message)],
    ))])
}

fn context_parameter() -> Parameter {
    Parameter::named(TypeName::named("MalContext").pointer(), "context")
}

fn append_function(output: &mut TranslationUnit, signature: FunctionSignature, body: Block) {
    output.push(FunctionDefinition::from_signature(signature, body));
    output.blank_line();
}
