use crate::c_emit::scalar::INTEGER_TYPES;
use crate::c_emit::syntax::{
    BinaryOperator, Block, Expr, FunctionSignature, Parameter, Statement, TranslationUnit,
};

use super::append_function;

pub(in crate::c_emit::runtime) fn emit_float_to_integer(needs: u32) -> TranslationUnit {
    let mut output = TranslationUnit::default();
    for source_index in 0..2 {
        let (source_name, source_type) = if source_index == 0 {
            ("f32", "float")
        } else {
            ("f64", "double")
        };
        for target in INTEGER_TYPES {
            if needs & (1_u32 << (source_index * 8 + target.index)) == 0 {
                continue;
            }
            let target_name = target.name;
            let target_type = target.c_type;
            append_function(
                &mut output,
                FunctionSignature::static_inline(
                    target_type,
                    format!("mal_{source_name}_to_{target_name}"),
                    [Parameter::named(source_type, "value")],
                ),
                Block::new([Statement::return_value(Expr::cast(
                    target_type,
                    Expr::identifier("value"),
                ))]),
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

pub(in crate::c_emit::runtime) fn emit_integer_binary(
    needs: u16,
    operation: &str,
    operator: BinaryOperator,
) -> TranslationUnit {
    let mut output = TranslationUnit::default();
    for integer in INTEGER_TYPES {
        if needs & integer.mask() == 0 {
            continue;
        }
        let name = integer.name;
        let c_type = integer.c_type;
        append_function(
            &mut output,
            FunctionSignature::static_inline(
                c_type,
                format!("mal_{name}_{operation}"),
                [
                    Parameter::named(c_type, "dividend"),
                    Parameter::named(c_type, "divisor"),
                ],
            ),
            Block::new([Statement::return_value(Expr::binary(
                operator,
                Expr::identifier("dividend"),
                Expr::identifier("divisor"),
            ))]),
        );
    }
    output
}
