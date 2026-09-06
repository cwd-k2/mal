use crate::c_emit::scalar::INTEGER_TYPES;
use crate::c_emit::syntax::{
    BinaryOperator, Block, Expr, FunctionSignature, Parameter, Statement, TranslationUnit,
};

use super::{append_function, context_parameter, trap};

pub(in crate::c_emit::runtime) fn emit_float_to_integer(needs: u32) -> TranslationUnit {
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

pub(in crate::c_emit::runtime) fn emit_integer_wrap(needs: u16) -> TranslationUnit {
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
                Statement::call(
                    "memcpy",
                    [
                        Expr::address_of(Expr::identifier("value")),
                        Expr::address_of(Expr::identifier("bits")),
                        Expr::sizeof_expr(Expr::identifier("value")),
                    ],
                ),
                Statement::return_value(Expr::identifier("value")),
            ]),
        );
    }
    output
}

pub(in crate::c_emit::runtime) fn emit_integer_checked(
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
