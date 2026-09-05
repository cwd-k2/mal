use std::fmt::Write;

use super::body::RuntimeNeeds;
use super::scalar::INTEGER_TYPES;

pub(super) fn emit(needs: &RuntimeNeeds) -> String {
    let mut output = String::from(RUNTIME_CORE);
    if needs.wrap != 0 {
        output.push_str(&integer_wrap_runtime(needs.wrap));
    }
    if needs.divide != 0 {
        output.push_str(&integer_checked_runtime(
            needs.divide,
            "divide",
            "/",
            "division by zero",
        ));
    }
    if needs.remainder != 0 {
        output.push_str(&integer_checked_runtime(
            needs.remainder,
            "remainder",
            "%",
            "remainder by zero",
        ));
    }
    if needs.shift_left != 0 || needs.shift_right != 0 {
        output.push_str(&integer_shift_runtime(needs.shift_left, needs.shift_right));
    }
    if needs.string_equality {
        output.push_str(RUNTIME_STRING_EQUALITY);
    }
    if needs.string_at {
        output.push_str(RUNTIME_STRING_AT);
    }
    if needs.float_to_integer != 0 {
        output.push_str(&float_to_integer_runtime(needs.float_to_integer));
    }
    output
}

const RUNTIME_CORE: &str = include_str!("runtime/core.c");
const RUNTIME_STRING_EQUALITY: &str = include_str!("runtime/string_equal.c");
const RUNTIME_STRING_AT: &str = include_str!("runtime/string_at.c");

fn float_to_integer_runtime(needs: u32) -> String {
    let mut output = String::new();
    for source_index in 0..2 {
        let (source_name, source_type, precision, literal_suffix) = if source_index == 0 {
            ("f32", "float", 24, "f")
        } else {
            ("f64", "double", 53, "")
        };
        for target in INTEGER_TYPES {
            let target_index = target.index;
            if needs & (1_u32 << (source_index * 8 + target_index)) == 0 {
                continue;
            }
            let target_name = target.name;
            let target_type = target.c_type;
            let signed = target.signed();
            let bits = match target_index {
                0 | 4 => 8,
                1 | 5 => 16,
                2 | 6 => 32,
                3 | 7 => 64,
                _ => unreachable!(),
            };
            let upper_exponent = if signed { bits - 1 } else { bits };
            let upper = format!("0x1p{upper_exponent}{literal_suffix}");
            let lower = if signed && bits <= precision {
                format!(
                    "value > (-0x1p{}{literal_suffix} - 1.0{literal_suffix})",
                    bits - 1
                )
            } else if signed {
                format!("value >= -0x1p{}{literal_suffix}", bits - 1)
            } else {
                format!("value > -1.0{literal_suffix}")
            };
            writeln!(
                output,
                "static inline {target_type} mal_{source_name}_to_{target_name}(MalContext *context, {source_type} value) {{"
            )
            .unwrap();
            writeln!(
                output,
                "    if (!({lower} && value < {upper})) {{ mal_trap(context, \"float-to-integer conversion out of range\"); }}"
            )
            .unwrap();
            writeln!(output, "    return ({target_type})value;\n}}\n").unwrap();
        }
    }
    output
}

fn integer_wrap_runtime(needs: u16) -> String {
    let mut output = String::new();
    for integer in INTEGER_TYPES {
        if integer.signed() && needs & integer.mask() != 0 {
            let name = integer.name;
            let c_type = integer.c_type;
            let unsigned = integer.unsigned;
            writeln!(output, "static {c_type} mal_{name}_from_{unsigned}({unsigned} bits) {{ {c_type} value; memcpy(&value, &bits, sizeof(value)); return value; }}").unwrap();
        }
    }
    output.push('\n');
    output
}

fn integer_checked_runtime(
    needs: u16,
    operation: &str,
    symbol: &str,
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
        writeln!(
            output,
            "static inline {c_type} mal_{name}_{operation}(MalContext *context, {c_type} dividend, {c_type} divisor) {{"
        )
        .unwrap();
        writeln!(
            output,
            "    if (divisor == ({c_type})0) {{ mal_trap(context, \"{zero_message}\"); }}"
        )
        .unwrap();
        if let Some(minimum) = integer.minimum {
            writeln!(output, "    if (dividend == {minimum} && divisor == ({c_type})-1) {{ mal_trap(context, \"signed {overflow_operation} overflow\"); }}").unwrap();
        }
        writeln!(output, "    return dividend {symbol} divisor;\n}}\n").unwrap();
    }
    output
}

fn integer_shift_runtime(left_needs: u16, right_needs: u16) -> String {
    let mut output = String::new();
    for integer in INTEGER_TYPES {
        let mask = integer.mask();
        if (left_needs | right_needs) & mask == 0 {
            continue;
        }
        let name = integer.name;
        let c_type = integer.c_type;
        let signed = integer.signed();
        let unsigned = integer.unsigned;
        let carrier = integer.carrier;
        let width = integer.width;
        let maximum = integer.maximum;
        let negative_check = if signed { "count < 0 || " } else { "" };
        let result = if signed {
            format!("mal_{name}_from_{unsigned}(({unsigned})shifted)")
        } else {
            format!("({c_type})shifted")
        };
        if left_needs & mask != 0 {
            writeln!(output, "static inline {c_type} mal_{name}_shift_left(MalContext *context, {c_type} value, {c_type} count) {{\n    if ({negative_check}({carrier})count >= {width}) {{ mal_trap(context, \"shift count out of range\"); }}\n    {carrier} shifted = ({carrier})({unsigned})value << ({carrier})count;\n    return {result};\n}}\n").unwrap();
        }
        if right_needs & mask != 0 && signed {
            writeln!(output, "static inline {c_type} mal_{name}_shift_right(MalContext *context, {c_type} value, {c_type} count) {{\n    if ({negative_check}({carrier})count >= {width}) {{ mal_trap(context, \"shift count out of range\"); }}\n    if (count == 0) {{ return value; }}\n    {carrier} bits = ({carrier})({unsigned})value;\n    {carrier} shifted = bits >> ({carrier})count;\n    if ((bits & (({carrier})1 << {})) != 0) {{ shifted |= ({carrier}){maximum} << ({width} - ({carrier})count); }}\n    return {result};\n}}\n", width - 1).unwrap();
        } else if right_needs & mask != 0 {
            writeln!(output, "static inline {c_type} mal_{name}_shift_right(MalContext *context, {c_type} value, {c_type} count) {{\n    if (({carrier})count >= {width}) {{ mal_trap(context, \"shift count out of range\"); }}\n    return ({c_type})(({carrier})value >> ({carrier})count);\n}}\n").unwrap();
        }
    }
    output
}
