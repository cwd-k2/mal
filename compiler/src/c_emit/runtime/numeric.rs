use crate::c_emit::scalar::{INTEGER_TYPES, IntegerType};

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
            let upper = format!("0x1p{upper_exponent}{literal_suffix}");
            let lower = if target.signed() && bits <= precision {
                format!(
                    "value > (-0x1p{}{literal_suffix} - 1.0{literal_suffix})",
                    bits - 1
                )
            } else if target.signed() {
                format!("value >= -0x1p{}{literal_suffix}", bits - 1)
            } else {
                format!("value > -1.0{literal_suffix}")
            };
            c_line!(
                &mut output,
                0,
                "static inline {target_type} mal_{source_name}_to_{target_name}(MalContext *context, {source_type} value) {{"
            );
            c_line!(
                &mut output,
                1,
                "if (!({lower} && value < {upper})) {{ mal_trap(context, \"float-to-integer conversion out of range\"); }}"
            );
            c_line!(&mut output, 1, "return ({target_type})value;");
            c_line!(&mut output, 0, "}}\n");
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
        c_line!(
            &mut output,
            0,
            "static {c_type} mal_{name}_from_{unsigned}({unsigned} bits) {{"
        );
        c_line!(&mut output, 1, "{c_type} value;");
        c_line!(&mut output, 1, "memcpy(&value, &bits, sizeof(value));");
        c_line!(&mut output, 1, "return value;");
        c_line!(&mut output, 0, "}}\n");
    }
    output
}

pub(super) fn emit_integer_checked(
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
        c_line!(
            &mut output,
            0,
            "static inline {c_type} mal_{name}_{operation}(MalContext *context, {c_type} dividend, {c_type} divisor) {{"
        );
        c_line!(
            &mut output,
            1,
            "if (divisor == ({c_type})0) {{ mal_trap(context, \"{zero_message}\"); }}"
        );
        if let Some(minimum) = integer.minimum {
            c_line!(
                &mut output,
                1,
                "if (dividend == {minimum} && divisor == ({c_type})-1) {{ mal_trap(context, \"signed {overflow_operation} overflow\"); }}"
            );
        }
        c_line!(&mut output, 1, "return dividend {symbol} divisor;");
        c_line!(&mut output, 0, "}}\n");
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
            format!("mal_{name}_from_{unsigned}(({unsigned})shifted)")
        } else {
            format!("({c_type})shifted")
        };
        if left_needs & mask != 0 {
            emit_shift_left(&mut output, integer, &result);
        }
        if right_needs & mask != 0 {
            if integer.signed() {
                emit_signed_shift_right(&mut output, integer, &result);
            } else {
                emit_unsigned_shift_right(&mut output, integer);
            }
        }
    }
    output
}

fn emit_shift_left(output: &mut String, integer: IntegerType, result: &str) {
    let name = integer.name;
    let c_type = integer.c_type;
    let unsigned = integer.unsigned;
    let carrier = integer.carrier;
    let width = integer.width;
    let negative_check = if integer.signed() {
        "count < 0 || "
    } else {
        ""
    };
    c_line!(
        output,
        0,
        "static inline {c_type} mal_{name}_shift_left(MalContext *context, {c_type} value, {c_type} count) {{"
    );
    c_line!(
        output,
        1,
        "if ({negative_check}({carrier})count >= {width}) {{ mal_trap(context, \"shift count out of range\"); }}"
    );
    c_line!(
        output,
        1,
        "{carrier} shifted = ({carrier})({unsigned})value << ({carrier})count;"
    );
    c_line!(output, 1, "return {result};");
    c_line!(output, 0, "}}\n");
}

fn emit_signed_shift_right(output: &mut String, integer: IntegerType, result: &str) {
    let name = integer.name;
    let c_type = integer.c_type;
    let unsigned = integer.unsigned;
    let carrier = integer.carrier;
    let width = integer.width;
    let maximum = integer.maximum;
    let negative_check = "count < 0 || ";
    c_line!(
        output,
        0,
        "static inline {c_type} mal_{name}_shift_right(MalContext *context, {c_type} value, {c_type} count) {{"
    );
    c_line!(
        output,
        1,
        "if ({negative_check}({carrier})count >= {width}) {{ mal_trap(context, \"shift count out of range\"); }}"
    );
    c_line!(output, 1, "if (count == 0) {{ return value; }}");
    c_line!(output, 1, "{carrier} bits = ({carrier})({unsigned})value;");
    c_line!(output, 1, "{carrier} shifted = bits >> ({carrier})count;");
    c_line!(
        output,
        1,
        "if ((bits & (({carrier})1 << {})) != 0) {{ shifted |= ({carrier}){maximum} << ({width} - ({carrier})count); }}",
        width - 1
    );
    c_line!(output, 1, "return {result};");
    c_line!(output, 0, "}}\n");
}

fn emit_unsigned_shift_right(output: &mut String, integer: IntegerType) {
    let name = integer.name;
    let c_type = integer.c_type;
    let carrier = integer.carrier;
    let width = integer.width;
    c_line!(
        output,
        0,
        "static inline {c_type} mal_{name}_shift_right(MalContext *context, {c_type} value, {c_type} count) {{"
    );
    c_line!(
        output,
        1,
        "if (({carrier})count >= {width}) {{ mal_trap(context, \"shift count out of range\"); }}"
    );
    c_line!(
        output,
        1,
        "return ({c_type})(({carrier})value >> ({carrier})count);"
    );
    c_line!(output, 0, "}}\n");
}
