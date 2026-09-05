use std::fmt::Write;

use super::body::RuntimeNeeds;

pub(super) fn emit(needs: &RuntimeNeeds) -> String {
    let mut output = String::from(RUNTIME_BASE);
    output.push_str(RUNTIME_ALLOCATION);
    output.push_str(RUNTIME_STRING_COPY);
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

const RUNTIME_BASE: &str = "typedef struct MalAllocation {\n    struct MalAllocation *next;\n} MalAllocation;\n\nstruct MalContext {\n    MalAllocation *allocations;\n};\n\n_Noreturn void mal_trap(MalContext *context, const char *message) {\n    (void)context;\n    fputs(\"mal trap: \", stderr);\n    fputs(message, stderr);\n    fputc('\\n', stderr);\n    abort();\n}\n\nstatic void mal_context_destroy(MalContext *context) {\n    MalAllocation *allocation = context->allocations;\n    while (allocation != NULL) {\n        MalAllocation *next = allocation->next;\n        free(allocation);\n        allocation = next;\n    }\n}\n\n";

const RUNTIME_ALLOCATION: &str = "static void *mal_allocate(MalContext *context, size_t size) {\n    if (size > SIZE_MAX - sizeof(MalAllocation)) {\n        mal_trap(context, \"allocation size overflow\");\n    }\n#ifdef MAL_TEST_FORCE_ALLOCATION_FAILURE\n    MalAllocation *allocation = NULL;\n#else\n    MalAllocation *allocation = malloc(sizeof(MalAllocation) + size);\n#endif\n    if (allocation == NULL) {\n        mal_trap(context, \"allocation failed\");\n    }\n    allocation->next = context->allocations;\n    context->allocations = allocation;\n    return allocation + 1;\n}\n\n";

const RUNTIME_STRING_COPY: &str = "MalString mal_string_copy(MalContext *context, const uint8_t *data, uint64_t length) {\n    if (length == UINT64_C(0)) { return (MalString){ NULL, UINT64_C(0) }; }\n    if (data == NULL) { mal_trap(context, \"null string data\"); }\n    size_t size = (size_t)length;\n    if ((uint64_t)size != length) { mal_trap(context, \"allocation size overflow\"); }\n    uint8_t *copy = (uint8_t *)mal_allocate(context, size);\n    memcpy(copy, data, size);\n    return (MalString){ copy, length };\n}\n\n";

const RUNTIME_STRING_EQUALITY: &str = "static uint8_t mal_string_equal(MalString left, MalString right) {\n    if (left.length != right.length) { return UINT8_C(0); }\n    for (uint64_t index = UINT64_C(0); index < left.length; index += UINT64_C(1)) {\n        if (left.data[index] != right.data[index]) { return UINT8_C(0); }\n    }\n    return UINT8_C(1);\n}\n\n";

const RUNTIME_STRING_AT: &str = "static uint8_t mal_string_at(MalContext *context, MalString value, uint64_t index) {\n    if (index >= value.length) { mal_trap(context, \"string index out of range\"); }\n    return value.data[index];\n}\n\n";

fn float_to_integer_runtime(needs: u32) -> String {
    let mut output = String::new();
    for source_index in 0..2 {
        let (source_name, source_type, precision, literal_suffix) = if source_index == 0 {
            ("f32", "float", 24, "f")
        } else {
            ("f64", "double", 53, "")
        };
        for (target_index, (target_name, target_type, _, signed)) in
            integer_runtime_types().into_iter().enumerate()
        {
            if needs & (1_u32 << (source_index * 8 + target_index)) == 0 {
                continue;
            }
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
    for (index, (name, c_type, _, signed)) in integer_runtime_types().into_iter().enumerate() {
        if signed && needs & (1 << index) != 0 {
            let unsigned = c_type.replacen("int", "uint", 1);
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
    for (index, (name, c_type, minimum, signed)) in integer_runtime_types().into_iter().enumerate()
    {
        if needs & (1 << index) == 0 {
            continue;
        }
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
        if signed {
            writeln!(output, "    if (dividend == {minimum} && divisor == ({c_type})-1) {{ mal_trap(context, \"signed {overflow_operation} overflow\"); }}").unwrap();
        }
        writeln!(output, "    return dividend {symbol} divisor;\n}}\n").unwrap();
    }
    output
}

fn integer_shift_runtime(left_needs: u16, right_needs: u16) -> String {
    let mut output = String::new();
    for (index, (name, c_type, _, signed)) in integer_runtime_types().into_iter().enumerate() {
        let mask = 1 << index;
        if (left_needs | right_needs) & mask == 0 {
            continue;
        }
        let (unsigned, carrier, width, maximum) = match name {
            "i8" | "u8" => ("uint8_t", "uint32_t", 8, "UINT8_MAX"),
            "i16" | "u16" => ("uint16_t", "uint32_t", 16, "UINT16_MAX"),
            "i32" | "u32" => ("uint32_t", "uint32_t", 32, "UINT32_MAX"),
            "i64" | "u64" => ("uint64_t", "uint64_t", 64, "UINT64_MAX"),
            _ => unreachable!(),
        };
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

fn integer_runtime_types() -> [(&'static str, &'static str, &'static str, bool); 8] {
    [
        ("i8", "int8_t", "INT8_MIN", true),
        ("i16", "int16_t", "INT16_MIN", true),
        ("i32", "int32_t", "INT32_MIN", true),
        ("i64", "int64_t", "INT64_MIN", true),
        ("u8", "uint8_t", "", false),
        ("u16", "uint16_t", "", false),
        ("u32", "uint32_t", "", false),
        ("u64", "uint64_t", "", false),
    ]
}
