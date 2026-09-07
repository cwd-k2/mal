use crate::c_emit::scalar::{INTEGER_TYPES, IntegerType};
use crate::c_emit::syntax::{
    Block, Expr, FunctionDefinition, FunctionSignature, Parameter, Statement, TranslationUnit,
};

mod conversion;

pub(super) use conversion::{emit_float_to_integer, emit_integer_binary, emit_integer_wrap};

pub(super) fn emit_integer_shift(left_needs: u16, right_needs: u16) -> TranslationUnit {
    // The source contract keeps counts within the operand width. Unsigned
    // carriers provide the specified bit behavior for every valid count.
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
    append_function(
        output,
        FunctionSignature::static_inline(
            c_type,
            format!("mal_{name}_shift_left"),
            [
                Parameter::named(c_type, "value"),
                Parameter::named(c_type, "count"),
            ],
        ),
        Block::new([
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
                Parameter::named(c_type, "value"),
                Parameter::named(c_type, "count"),
            ],
        ),
        Block::new([
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
    append_function(
        output,
        FunctionSignature::static_inline(
            c_type,
            format!("mal_{name}_shift_right"),
            [
                Parameter::named(c_type, "value"),
                Parameter::named(c_type, "count"),
            ],
        ),
        Block::new([Statement::return_value(Expr::cast(
            c_type,
            Expr::shift_right(
                Expr::cast(carrier, Expr::identifier("value")),
                Expr::cast(carrier, Expr::identifier("count")),
            ),
        ))]),
    );
}

fn append_function(output: &mut TranslationUnit, signature: FunctionSignature, body: Block) {
    output.push(FunctionDefinition::from_signature(signature, body));
    output.blank_line();
}
