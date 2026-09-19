use super::*;

#[test]
fn checks_int32_literal_context_and_boundaries() {
    let program = check_ok(
        "minimum :: Int32 := -2147483648;\n\
         maximum := 2147483647i32;",
    );
    assert!(matches!(
        top_binding(&program, 0).value.kind,
        ExpressionKind::Integer(value) if value == i128::from(i32::MIN)
    ));
    assert!(matches!(
        top_binding(&program, 1).value.kind,
        ExpressionKind::Integer(value) if value == i128::from(i32::MAX)
    ));

    assert_eq!(
        top_binding(&check_ok("value := 1;"), 0).value.ty,
        Type::Int64
    );
    for text in [
        "value :: Int32 := 2147483648;",
        "value :: Int32 := -2147483649;",
    ] {
        assert!(
            check_error(text).message.contains("out of range"),
            "input: {text}"
        );
    }
}

#[test]
fn checks_all_fixed_width_literal_boundaries_and_byte_literals() {
    let cases = [
        ("127i8", Type::Int8, 127_i128),
        ("32767i16", Type::Int16, 32_767),
        ("2147483647i32", Type::Int32, 2_147_483_647),
        (
            "9223372036854775807i64",
            Type::Int64,
            9_223_372_036_854_775_807,
        ),
        ("255u8", Type::UInt8, 255),
        ("65535u16", Type::UInt16, 65_535),
        ("4294967295u32", Type::UInt32, 4_294_967_295),
        (
            "18446744073709551615u64",
            Type::UInt64,
            18_446_744_073_709_551_615,
        ),
    ];
    for (literal, ty, value) in cases {
        let program = check_ok(&format!("value := {literal};"));
        let expression = &top_binding(&program, 0).value;
        assert_eq!(expression.ty, ty, "literal: {literal}");
        assert!(
            matches!(expression.kind, ExpressionKind::Integer(actual) if actual == value),
            "literal: {literal}"
        );
    }

    let byte = check_ok(r"value := '\xff';");
    assert_eq!(top_binding(&byte, 0).value.ty, Type::UInt8);
    assert!(matches!(
        top_binding(&byte, 0).value.kind,
        ExpressionKind::Integer(255)
    ));

    for (literal, value) in [
        ("-128i8", -128_i128),
        ("-32768i16", -32_768),
        ("-2147483648i32", -2_147_483_648),
        ("-9223372036854775808i64", -9_223_372_036_854_775_808),
    ] {
        let program = check_ok(&format!("value := {literal};"));
        assert!(
            matches!(top_binding(&program, 0).value.kind, ExpressionKind::Integer(actual) if actual == value),
            "literal: {literal}"
        );
    }

    for literal in [
        "128i8",
        "32768i16",
        "2147483648i32",
        "9223372036854775808i64",
        "256u8",
        "65536u16",
        "4294967296u32",
        "18446744073709551616u64",
        "-129i8",
    ] {
        assert!(
            check_error(&format!("value := {literal};"))
                .message
                .contains("out of range"),
            "literal: {literal}"
        );
    }
}

#[test]
fn rounds_decimal_float_literals_to_exact_binary_bits() {
    let program = check_ok(
        "default := 0.1;\n\
         single := 0.1f32;\n\
         contextual :: Float32 := 1.000000059604644775390625;\n\
         nextEven := 1.000000178813934326171875f32;\n\
         minimumSubnormal := 1.40129846e-45f32;\n\
         maximum := 340282346638528859811704183484516925440f32;\n\
         doubleHalfEven := 1.00000000000000011102230246251565404236316680908203125f64;\n\
         doubleNextEven := 1.00000000000000033306690738754696212708950042724609375f64;\n\
         doubleMinimumSubnormal := 4.9406564584124654e-324f64;",
    );
    let expected = [
        (Type::Float64, 0x3fb9_9999_9999_999a),
        (Type::Float32, 0x3dcc_cccd),
        (Type::Float32, 0x3f80_0000),
        (Type::Float32, 0x3f80_0002),
        (Type::Float32, 0x0000_0001),
        (Type::Float32, 0x7f7f_ffff),
        (Type::Float64, 0x3ff0_0000_0000_0000),
        (Type::Float64, 0x3ff0_0000_0000_0002),
        (Type::Float64, 0x0000_0000_0000_0001),
    ];
    for (index, (ty, bits)) in expected.into_iter().enumerate() {
        let binding = top_binding(&program, index);
        assert_eq!(binding.value.ty, ty);
        assert!(matches!(binding.value.kind, ExpressionKind::Float(actual) if actual == bits));
    }
}

#[test]
fn rounds_very_long_decimal_coefficients_in_linear_input_space() {
    let tail = "2".repeat(100_000);
    let long = format!("value := 1{tail}e-100000f64;");
    let representative = format!("value := 1{}1e-1100f64;", "2".repeat(1099));

    let long = top_binding(&check_ok(&long), 0).value.kind.clone();
    let representative = top_binding(&check_ok(&representative), 0)
        .value
        .kind
        .clone();

    assert_eq!(long, representative);

    let above_half = format!(
        "value := 1.00000000000000011102230246251565404236316680908203125{}1f64;",
        "0".repeat(1_200)
    );
    assert!(matches!(
        top_binding(&check_ok(&above_half), 0).value.kind,
        ExpressionKind::Float(0x3ff0_0000_0000_0001)
    ));
}

#[test]
fn rejects_decimal_float_literals_above_the_finite_range() {
    for text in [
        "value := 340282346638528859811704183484516925441f32;",
        "value := 1e309f64;",
    ] {
        assert!(
            check_error(text)
                .message
                .contains("literal is out of range"),
            "input: {text}"
        );
    }
}

#[test]
fn checks_float_arithmetic_comparison_and_negation() {
    let program = check_ok(
        "calculate :: Float32 -> Bool := (value) -> {\n\
           negative := -value;\n\
           result := (negative + 2.0f32) * 3.0f32 / 4.0f32;\n\
           result >= 0.0f32 && result != value;\n\
         };",
    );
    assert_eq!(
        top_binding(&program, 0).value.ty,
        Type::Function {
            parameter: Box::new(Type::Float32).into(),
            result: Box::new(Type::Sum(vec![Type::Unit, Type::Unit].into())).into(),
        }
    );
    assert_eq!(
        check_error("value := 1.0f32 % 1.0f32;").message,
        "integer operator requires integer operands"
    );
}

#[test]
fn checks_int32_and_bool_operator_families() {
    let program = check_ok(
        "predicate :: Int32 -> Bool := (x) -> {\n\
           !(x + 1 < 2) || false && (x == 0);\n\
         };",
    );
    let Type::Function { result, .. } = &top_binding(&program, 0).value.ty else {
        panic!("expected function type");
    };
    assert_eq!(
        result.as_ref(),
        &Type::Sum(vec![Type::Unit, Type::Unit].into())
    );

    assert_eq!(
        check_error("bad :: Int32 -> Int32 := (x) -> { x + true; };").message,
        "type mismatch"
    );
}

#[test]
fn checks_integer_operators_for_every_fixed_width_type() {
    for name in [
        "Int8", "Int16", "Int32", "Int64", "UInt8", "UInt16", "UInt32", "UInt64",
    ] {
        check_ok(&format!(
            "compute :: {name} -> {name} := (x) -> {{\n\
               ((~x + 1) * 2 - 1) / 1 % 1 << 0 >> 0 & x | x ^ x;\n\
             }};\n\
             compare :: {name} -> Bool := (x) -> {{\n\
               x < x || x <= x || x > x || x >= x || x == x || x != x;\n\
             }};"
        ));
    }

    assert_eq!(check_error("bad := 1i8 + 1u8;").message, "type mismatch");
    assert!(
        check_error("bad := ~();")
            .message
            .contains("requires an integer")
    );
}

#[test]
fn checks_modulo_integer_conversions() {
    for (expression, expected) in [
        ("(-1i8).u8", Type::UInt8),
        ("255u16.i8", Type::Int8),
        ("(-1i8).u16", Type::UInt16),
        ("255u8.i16", Type::Int16),
    ] {
        let program = check_ok(&format!("value := {expression};"));
        assert_eq!(top_binding(&program, 0).value.ty, expected);
        assert!(matches!(
            top_binding(&program, 0).value.kind,
            ExpressionKind::NumericConversion { .. }
        ));
    }

    assert!(
        check_error("value := ().i8;")
            .message
            .contains("requires a numeric value")
    );
}

#[test]
fn checks_conversions_between_integer_and_float_types() {
    let program = check_ok(
        "single := 16777217u64.f32;\n\
         double := 0.1f32.f64;\n\
         narrowed := 0.1f64.f32;\n\
         signed := (-1.75f64).i32;\n\
         unsigned := 1.75f32.u64;",
    );
    let expected = [
        Type::Float32,
        Type::Float64,
        Type::Float32,
        Type::Int32,
        Type::UInt64,
    ];
    for (index, ty) in expected.into_iter().enumerate() {
        let binding = top_binding(&program, index);
        assert_eq!(binding.value.ty, ty);
        assert!(matches!(
            binding.value.kind,
            ExpressionKind::NumericConversion { .. }
        ));
    }

    assert_eq!(
        check_error("value := ().f32;").message,
        "numeric conversion requires a numeric value"
    );
}

#[test]
fn checks_target_quantity_literals_arithmetic_and_conversions() {
    check_ok(
        "scale :: (USize, ByteSize) -> ByteSize := (count, size) -> count * size;\n\
         reversed :: (ByteSize, USize) -> ByteSize := (size, count) -> size * count;\n\
         count :: USize -> USize := (value) -> (value / 3usize) % 3usize;\n\
         converted :: USize -> ByteSize := (value) -> value.bytes;\n\
         compared :: (ByteSize, ByteSize) -> Bool := (left, right) -> left >= right;",
    );
}

#[test]
fn rejects_operations_outside_target_quantity_algebra() {
    for text in [
        "bad := 2bytes * 3bytes;",
        "bad := 2bytes / 1bytes;",
        "bad := 2bytes % 1bytes;",
        "bad := 2usize << 1usize;",
        "bad := 2bytes & 1bytes;",
        "bad := ~2usize;",
        "bad := -2bytes;",
    ] {
        assert!(
            check_error(text).message.contains("not defined"),
            "input: {text}"
        );
    }
}

#[test]
fn checks_address_offsets_and_rejects_address_values_as_numbers() {
    check_ok("forward :: (Address, ByteSize) -> Address := (address, offset) -> address + offset;");

    for text in [
        "bad :: (Address, Address) -> Bool := (left, right) -> left == right;",
        "bad :: (Address, ByteSize) -> Address := (address, offset) -> address - offset;",
        "bad :: Address -> Address := (address) -> address + 1usize;",
        "bad :: Address -> USize := (address) -> address.usize;",
    ] {
        let message = check_error(text).message;
        assert!(
            message.contains("not defined")
                || message.contains("type mismatch")
                || message.contains("numeric value")
                || message.contains("forward byte offset"),
            "input: {text}"
        );
    }
}
