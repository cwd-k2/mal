use malc::check;
use malc::check::ast::{ExpressionKind, TopItem, Type};
use malc::parser::parse;
use malc::resolve;
use malc::source::{FileId, SourceFile};

fn source(text: &str) -> SourceFile {
    SourceFile::new(FileId::new(23), "check-test.mal", text.into())
}

fn check_ok(text: &str) -> check::ast::Program {
    let source = source(text);
    let parsed = parse(&source).unwrap_or_else(|error| panic!("{}", error.render(&source)));
    let resolved =
        resolve::resolve(&parsed).unwrap_or_else(|error| panic!("{}", error.render(&source)));
    check::check(&resolved).unwrap_or_else(|error| panic!("{}", error.render(&source)))
}

fn check_error(text: &str) -> malc::diagnostic::Diagnostic {
    let source = source(text);
    let parsed = parse(&source).unwrap_or_else(|error| panic!("{}", error.render(&source)));
    let resolved =
        resolve::resolve(&parsed).unwrap_or_else(|error| panic!("{}", error.render(&source)));
    check::check(&resolved).expect_err("type checking should fail")
}

fn top_binding(program: &check::ast::Program, index: usize) -> &check::ast::Binding {
    let TopItem::Binding(binding) = &program.items[index].kind else {
        panic!("expected a top-level binding");
    };
    binding
}

#[test]
fn checks_the_basic_host_example_end_to_end_through_typed_ast() {
    let program = check_ok(
        "extern printInt32 :: Int32 -> Unit;\n\
         main :: Unit -> Int32 := \\() {\n\
           extern printInt32(42);\n\
           0;\n\
         };",
    );
    let TopItem::ExternalOperation {
        parameter, result, ..
    } = &program.items[0].kind
    else {
        panic!("expected external operation");
    };
    assert_eq!(*parameter, Type::Int32);
    assert_eq!(*result, Type::Unit);
    assert_eq!(
        top_binding(&program, 1).value.ty,
        Type::Function {
            parameter: Box::new(Type::Unit),
            result: Box::new(Type::Int32),
        }
    );
}

#[test]
fn expands_aliases_and_compares_types_structurally() {
    let program = check_ok(
        "Flag :: [Unit, Unit];\n\
         choose :: Flag -> Int32 := \\(flag :: Bool) {\n\
           if (flag) then { 1 } else { 0 };\n\
         };",
    );
    let TopItem::TypeAlias { ty, .. } = &program.items[0].kind else {
        panic!("expected alias");
    };
    assert_eq!(*ty, Type::Sum(vec![Type::Unit, Type::Unit]));
}

#[test]
fn rejects_recursive_aliases_even_when_unused() {
    let error = check_error("First :: Second; Second :: First;");
    assert_eq!(error.message, "recursive type alias");
}

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
        "calculate :: Float32 -> Bool := \\(value :: Float32) {\n\
           negative := -value;\n\
           result := (negative + 2.0f32) * 3.0f32 / 4.0f32;\n\
           result >= 0.0f32 && result != value;\n\
         };",
    );
    assert_eq!(
        top_binding(&program, 0).value.ty,
        Type::Function {
            parameter: Box::new(Type::Float32),
            result: Box::new(Type::Sum(vec![Type::Unit, Type::Unit])),
        }
    );
    assert_eq!(
        check_error("value := 1.0f32 % 1.0f32;").message,
        "integer operator requires integer operands"
    );
}

#[test]
fn checks_engram_literals_as_immutable_bytes() {
    let program = check_ok(r#"empty :: Engram := ""; bytes := "あ\0\xff";"#);
    assert_eq!(top_binding(&program, 0).value.ty, Type::Engram);
    assert!(matches!(
        top_binding(&program, 1).value.kind,
        ExpressionKind::Engram(ref value) if value == &[0xe3, 0x81, 0x82, 0, 255]
    ));
}

#[test]
fn checks_engram_operators_and_byte_wise_equality() {
    let program = check_ok(
        r#"length :: Engram -> UInt64 := \(value :: Engram) { #value; };
item :: (Engram, UInt64) -> UInt8 := \(value :: Engram, index :: UInt64) {
  value # index;
};
same :: Unit -> Bool := \() { "a\0" == "a\x00"; };
different :: Unit -> Bool := \() { "a" != "b"; };
literal :: Unit -> UInt64 := \() { #"hoge" + UInt64("hoge" # 1); };
concatenate :: (Engram, Engram) -> Engram := \(left :: Engram, right :: Engram) {
  left + right;
};"#,
    );
    let ExpressionKind::Lambda(length) = &top_binding(&program, 0).value.kind else {
        panic!("expected lambda");
    };
    assert!(matches!(
        length.body.result.kind,
        ExpressionKind::EngramLength { .. }
    ));
    let ExpressionKind::Lambda(item) = &top_binding(&program, 1).value.kind else {
        panic!("expected lambda");
    };
    assert!(matches!(
        item.body.result.kind,
        ExpressionKind::EngramAt { .. }
    ));
    for index in 2..=3 {
        let Type::Function { result, .. } = &top_binding(&program, index).value.ty else {
            panic!("expected function type");
        };
        assert_eq!(result.as_ref(), &Type::Sum(vec![Type::Unit, Type::Unit]));
    }
    let Type::Function { result, .. } = &top_binding(&program, 4).value.ty else {
        panic!("expected function type");
    };
    assert_eq!(result.as_ref(), &Type::UInt64);
    let Type::Function { result, .. } = &top_binding(&program, 5).value.ty else {
        panic!("expected function type");
    };
    assert_eq!(result.as_ref(), &Type::Engram);
}

#[test]
fn rejects_unsupported_or_mistyped_engram_operations() {
    for text in [
        r#"bad := \() { "a" + 1; };"#,
        r#"bad := "a" < "b";"#,
        r#"bad := #1;"#,
        r#"bad := "a" # 0u8;"#,
        r#"bad := 1 # 0u64;"#,
    ] {
        let error = check_error(text);
        assert!(error.primary.is_some(), "input: {text}");
    }
}

#[test]
fn checks_ptr_extern_signatures_and_memory_primitives() {
    let program = check_ok(
        "extern memory :: Unit -> Ptr;\n\
         useMemory :: Ptr -> UInt8 := \\(pointer :: Ptr) {\n\
           slot := pointer + 8u64;\n\
           storeInt64(slot, 42i64);\n\
           value := loadInt64(slot);\n\
           storeUInt8(slot, UInt8(value));\n\
           loadUInt8(slot);\n\
         };",
    );
    let TopItem::ExternalOperation {
        parameter, result, ..
    } = &program.items[0].kind
    else {
        panic!("expected external operation");
    };
    assert_eq!(*parameter, Type::Unit);
    assert_eq!(*result, Type::Ptr);
    let ExpressionKind::Lambda(function) = &top_binding(&program, 1).value.kind else {
        panic!("expected lambda");
    };
    assert_eq!(function.body.result.ty, Type::UInt8);
    assert!(matches!(
        function.body.result.kind,
        ExpressionKind::Memory { .. }
    ));
}

#[test]
fn checks_memory_primitives_for_every_supported_value_type() {
    let program = check_ok(
        "useMemory :: Ptr -> Unit := \\(pointer :: Ptr) {\n\
           storeInt8(pointer, loadInt8(pointer));\n\
           storeInt16(pointer, loadInt16(pointer));\n\
           storeInt32(pointer, loadInt32(pointer));\n\
           storeInt64(pointer, loadInt64(pointer));\n\
           storeUInt8(pointer, loadUInt8(pointer));\n\
           storeUInt16(pointer, loadUInt16(pointer));\n\
           storeUInt32(pointer, loadUInt32(pointer));\n\
           storeUInt64(pointer, loadUInt64(pointer));\n\
           storeFloat32(pointer, loadFloat32(pointer));\n\
           storeFloat64(pointer, loadFloat64(pointer));\n\
           storePtr(pointer, loadPtr(pointer));\n\
           storeEngram(pointer, loadEngram(pointer));\n\
           ();\n\
         };",
    );
    let ExpressionKind::Lambda(function) = &top_binding(&program, 0).value.kind else {
        panic!("expected lambda");
    };
    assert_eq!(function.body.items.len(), 12);
    assert!(function
        .body
        .items
        .iter()
        .all(|item| matches!(item, malc::check::ast::BodyItem::Expression(expression) if expression.ty == Type::Unit)));
}

#[test]
fn checks_storage_sizes_for_scalar_ptr_and_engram_types() {
    let program = check_ok(
        "Byte :: UInt8;\n\
         byteSize :: UInt64 := @Byte;\n\
         sizes :: Unit -> UInt64 := \\() {\n\
           @Int8 + @Int16 + @Int32 + @Int64 + byteSize\n\
             + @UInt16 + @UInt32 + @UInt64 + @Float32 + @Float64 + @Ptr + @Engram;\n\
         };",
    );
    assert_eq!(top_binding(&program, 1).value.ty, Type::UInt64);
    assert!(matches!(
        top_binding(&program, 1).value.kind,
        ExpressionKind::StorageSize(Type::UInt8)
    ));
    let ExpressionKind::Lambda(function) = &top_binding(&program, 2).value.kind else {
        panic!("expected lambda");
    };
    assert_eq!(function.body.result.ty, Type::UInt64);
}

#[test]
fn rejects_storage_sizes_without_a_memory_representation() {
    for text in [
        "value := @Unit;",
        "value := @(UInt8, Engram);",
        "value := @[UInt8, Engram];",
        "extern Resource; value := @Resource;",
        "value := @(Int32 -> Int32);",
    ] {
        let error = check_error(text);
        assert_eq!(
            error.message, "type has no defined memory storage representation",
            "input: {text}"
        );
        assert!(error.primary.is_some(), "input: {text}");
    }
}

#[test]
fn rejects_mistyped_memory_operations() {
    for text in [
        "extern memory :: Unit -> Ptr; bad := \\() { extern memory() + 1i64; };",
        "bad := \\() { loadInt64(0u64); };",
        "extern memory :: Unit -> Ptr; bad := \\() { storeUInt8(extern memory(), 1u64); (); };",
        "extern memory :: Unit -> Ptr; bad := \\() { storePtr(extern memory(), 1u64); (); };",
        "bad := \\() { loadEngram(0u64); };",
        "extern memory :: Unit -> Ptr; bad := \\() { storeEngram(extern memory(), 1u64); (); };",
        "extern memory :: Unit -> Ptr; bad := \\() { 1u64 + extern memory(); };",
        "extern memory :: Unit -> Ptr; bad := \\() { extern memory() + extern memory(); };",
        "extern memory :: Unit -> Ptr; bad := \\() { 1u64 - extern memory(); };",
    ] {
        let error = check_error(text);
        assert!(error.primary.is_some(), "input: {text}");
    }
}

#[test]
fn gives_every_memory_function_a_first_class_function_type() {
    let program = check_ok(
        "li8 :: Ptr -> Int8 := loadInt8; si8 :: (Ptr, Int8) -> Unit := storeInt8;\n\
         li16 :: Ptr -> Int16 := loadInt16; si16 :: (Ptr, Int16) -> Unit := storeInt16;\n\
         li32 :: Ptr -> Int32 := loadInt32; si32 :: (Ptr, Int32) -> Unit := storeInt32;\n\
         li64 :: Ptr -> Int64 := loadInt64; si64 :: (Ptr, Int64) -> Unit := storeInt64;\n\
         lu8 :: Ptr -> UInt8 := loadUInt8; su8 :: (Ptr, UInt8) -> Unit := storeUInt8;\n\
         lu16 :: Ptr -> UInt16 := loadUInt16; su16 :: (Ptr, UInt16) -> Unit := storeUInt16;\n\
         lu32 :: Ptr -> UInt32 := loadUInt32; su32 :: (Ptr, UInt32) -> Unit := storeUInt32;\n\
         lu64 :: Ptr -> UInt64 := loadUInt64; su64 :: (Ptr, UInt64) -> Unit := storeUInt64;\n\
         lf32 :: Ptr -> Float32 := loadFloat32; sf32 :: (Ptr, Float32) -> Unit := storeFloat32;\n\
         lf64 :: Ptr -> Float64 := loadFloat64; sf64 :: (Ptr, Float64) -> Unit := storeFloat64;\n\
         lp :: Ptr -> Ptr := loadPtr; sp :: (Ptr, Ptr) -> Unit := storePtr;\n\
         le :: Ptr -> Engram := loadEngram; se :: (Ptr, Engram) -> Unit := storeEngram;",
    );
    assert_eq!(program.items.len(), 24);
    assert!(program.items.iter().all(|item| {
        matches!(
            &item.kind,
            TopItem::Binding(binding)
                if matches!(binding.value.kind, ExpressionKind::MemoryFunction { .. })
                    && matches!(binding.value.ty, Type::Function { .. })
        )
    }));
}

#[test]
fn rejects_binding_and_block_result_type_mismatches() {
    assert_eq!(check_error("value :: Unit := 0;").message, "type mismatch");
    assert_eq!(
        check_error("main :: Unit -> Unit := \\() { 0; };").message,
        "type mismatch"
    );
}

#[test]
fn checks_function_application_and_zero_argument_unit_lowering() {
    let program = check_ok(
        "identity :: Int32 -> Int32 := \\(x :: Int32) { x; };\n\
         thunk :: Unit -> Int32 := \\() { identity(4); };\n\
         caller :: Unit -> Int32 := \\() { thunk(); };",
    );
    assert_eq!(
        top_binding(&program, 2).value.ty,
        Type::Function {
            parameter: Box::new(Type::Unit),
            result: Box::new(Type::Int32),
        }
    );

    assert_eq!(
        check_error(
            "identity :: Int32 -> Int32 := \\(x :: Int32) { x; };\n\
             bad :: Int32 := identity();"
        )
        .message,
        "type mismatch"
    );
    assert_eq!(
        check_error("value :: Int32 := 1i32();").message,
        "cannot call a non-function value"
    );
}

#[test]
fn checks_return_as_an_ordinary_local_name() {
    check_ok("value :: Unit -> Int64 := \\() { return := 1; return; };");
}

#[test]
fn checks_sum_injection_payload_and_index() {
    let program = check_ok(
        "Maybe :: [Unit, Int32];\n\
         some :: Maybe := Maybe[1](42);",
    );
    assert_eq!(
        top_binding(&program, 1).value.ty,
        Type::Sum(vec![Type::Unit, Type::Int32])
    );

    assert_eq!(
        check_error("Maybe :: [Unit, Int32]; bad :: Maybe := Maybe[2](0);").message,
        "sum variant index is out of range"
    );
    assert_eq!(
        check_error("Maybe :: [Unit, Int32]; bad :: Maybe := Maybe[0](0);").message,
        "type mismatch"
    );
}

#[test]
fn checks_case_exhaustiveness_uniqueness_and_result_types() {
    check_ok(
        "Maybe :: [Unit, Int32];\n\
         get :: Maybe -> Int32 := \\(value :: Maybe) {\n\
           case (value)\n\
             [0](_) { 0 }\n\
             [1](x) { y := x; y };\n\
         };",
    );

    let prefix = "Maybe :: [Unit, Int32]; get :: Maybe -> Int32 := \\(value :: Maybe) { ";
    assert_eq!(
        check_error(&format!("{prefix}case (value) [0](_) {{ 0 }}; }};")).message,
        "non-exhaustive case expression"
    );
    assert!(
        check_error(&format!(
            "{prefix}case (value) [0](_) {{ 0 }} [0](_) {{ 1 }} [1](x) {{ x }}; }};"
        ))
        .message
        .starts_with("duplicate case arm")
    );
    assert_eq!(
        check_error(&format!(
            "{prefix}case (value) [0](_) {{ () }} [1](x) {{ x }}; }};"
        ))
        .message,
        "type mismatch"
    );
}

#[test]
fn checks_if_condition_and_branch_types() {
    check_ok(
        "choose :: Bool -> Int32 := \\(condition :: Bool) {\n\
           if (condition) then { 1 } else { 2 };\n\
         };",
    );
    assert_eq!(
        check_error(
            "bad :: Int32 -> Int32 := \\(condition :: Int32) {\n\
               if (condition) then { 1 } else { 2 };\n\
             };"
        )
        .message,
        "type mismatch"
    );
    assert_eq!(
        check_error(
            "bad :: Bool -> Int32 := \\(condition :: Bool) {\n\
               if (condition) then { 1 } else { () };\n\
             };"
        )
        .message,
        "type mismatch"
    );
}

#[test]
fn checks_int32_and_bool_operator_families() {
    let program = check_ok(
        "predicate :: Int32 -> Bool := \\(x :: Int32) {\n\
           !(x + 1 < 2) || false && (x == 0);\n\
         };",
    );
    let Type::Function { result, .. } = &top_binding(&program, 0).value.ty else {
        panic!("expected function type");
    };
    assert_eq!(result.as_ref(), &Type::Sum(vec![Type::Unit, Type::Unit]));

    assert_eq!(
        check_error("bad :: Int32 -> Int32 := \\(x :: Int32) { x + true; };").message,
        "type mismatch"
    );
}

#[test]
fn checks_integer_operators_for_every_fixed_width_type() {
    for name in [
        "Int8", "Int16", "Int32", "Int64", "UInt8", "UInt16", "UInt32", "UInt64",
    ] {
        check_ok(&format!(
            "compute :: {name} -> {name} := \\(x :: {name}) {{\n\
               ((~x + 1) * 2 - 1) / 1 % 1 << 0 >> 0 & x | x ^ x;\n\
             }};\n\
             compare :: {name} -> Bool := \\(x :: {name}) {{\n\
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
        ("UInt8(-1i8)", Type::UInt8),
        ("Int8(255u16)", Type::Int8),
        ("UInt16(-1i8)", Type::UInt16),
        ("Int16(255u8)", Type::Int16),
    ] {
        let program = check_ok(&format!("value := {expression};"));
        assert_eq!(top_binding(&program, 0).value.ty, expected);
        assert!(matches!(
            top_binding(&program, 0).value.kind,
            ExpressionKind::NumericConversion { .. }
        ));
    }

    assert!(
        check_error("value := Int8(());")
            .message
            .contains("requires a numeric value")
    );
    assert!(
        check_error("value := Bool(1i8);")
            .message
            .contains("requires a numeric type")
    );
}

#[test]
fn checks_conversions_between_integer_and_float_types() {
    let program = check_ok(
        "single := Float32(16777217u64);\n\
         double := Float64(0.1f32);\n\
         narrowed := Float32(0.1f64);\n\
         signed := Int32(-1.75f64);\n\
         unsigned := UInt64(1.75f32);",
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
        check_error("value := Float32(());").message,
        "numeric conversion requires a numeric value"
    );
}

#[test]
fn checks_products_destructuring_and_multiple_parameters() {
    let program = check_ok(
        "Pair :: (Int32, UInt8);\n\
         pair :: Pair := (1, 2);\n\
         add :: (Int32, Int32) -> Int32 := \\(left :: Int32, right :: Int32) {\n\
           left + right;\n\
         };\n\
         main :: Unit -> Int32 := \\() {\n\
           (first, _) := pair;\n\
           nested := ((first, 2i32), 39i32);\n\
           ((left, right), extra) := nested;\n\
           add(left + right, extra);\n\
         };",
    );
    assert_eq!(
        top_binding(&program, 1).value.ty,
        Type::Product(vec![Type::Int32, Type::UInt8])
    );
    assert_eq!(
        top_binding(&program, 2).value.ty,
        Type::Function {
            parameter: Box::new(Type::Product(vec![Type::Int32, Type::Int32])),
            result: Box::new(Type::Int32),
        }
    );

    assert_eq!(
        check_error("pair := (1, 2); (first, second, third) := pair;").message,
        "product pattern has the wrong arity"
    );
    assert_eq!(
        check_error("value := 1; (first, second) := value;").message,
        "product pattern requires a product value"
    );
}

#[test]
fn checks_nominal_external_opaque_types() {
    let program = check_ok(
        "extern Mem;\n\
         extern File;\n\
         extern allocate :: UInt64 -> Mem;\n\
         extern length :: Mem -> UInt64;\n\
         main :: Unit -> Int32 := \\() {\n\
           mem := extern allocate(4u64);\n\
           Int32(extern length(mem));\n\
         };",
    );
    assert!(matches!(
        program.items[0].kind,
        TopItem::ExternalType { .. }
    ));
    let TopItem::ExternalOperation {
        result: Type::External { name, .. },
        ..
    } = &program.items[2].kind
    else {
        panic!("allocate should return an external type");
    };
    assert_eq!(name, "Mem");

    assert_eq!(
        check_error(
            "extern Mem;\n\
             extern File;\n\
             extern getFile :: Unit -> File;\n\
             extern useMem :: Mem -> Unit;\n\
             main :: Unit -> Int32 := \\() {\n\
               extern useMem(extern getFile());\n\
               0;\n\
             };"
        )
        .message,
        "type mismatch"
    );
}

#[test]
fn validates_extern_signatures_recursively() {
    assert_eq!(
        check_error("extern callback :: (Int32 -> Unit) -> Unit;").message,
        "external operation `callback` uses a function value"
    );
    assert_eq!(
        check_error("extern wrapped :: [Unit, Int32 -> Int32] -> Unit;").message,
        "external operation `wrapped` uses a function value"
    );
    assert_eq!(
        check_error("extern constant :: Int32;").message,
        "external operation `constant` must have a function type"
    );
}

#[test]
fn rejects_effectful_top_level_initializers() {
    let error = check_error(
        "extern read :: Unit -> Int32;\n\
         value :: Int32 := extern read();",
    );
    assert_eq!(error.message, "unsupported top-level initializer");
}

#[test]
fn checks_top_level_and_local_self_recursion_against_the_annotation() {
    let program = check_ok(
        "count :: Int64 -> Int64 := \\(n :: Int64) {\n\
           if (n == 0) then { 0 } else { count(n - 1) };\n\
         };\n\
         main :: Unit -> Int32 := \\() {\n\
           local :: Int32 -> Int32 := \\(n :: Int32) {\n\
             if (n == 0) then { 0 } else { local(n - 1) };\n\
           };\n\
           local(10);\n\
         };",
    );

    for index in [0, 1] {
        let ExpressionKind::Lambda(lambda) = &top_binding(&program, index).value.kind else {
            panic!("expected lambda");
        };
        assert!(lambda.self_binding.is_some());
    }
}

#[test]
fn rejects_recursive_lambda_that_disagrees_with_its_annotation() {
    let error = check_error("looping :: Int32 -> Int32 := \\(n :: Int64) { looping(n); };");
    assert_eq!(error.message, "type mismatch");
}

#[test]
fn propagates_types_through_capture_bindings() {
    let program = check_ok(
        "make :: Int32 -> (Int32 -> Int32) := \\(x :: Int32) {\n\
           \\<x>(y :: Int32) { x + y; };\n\
         };",
    );
    let ExpressionKind::Lambda(outer) = &top_binding(&program, 0).value.kind else {
        panic!("expected outer lambda");
    };
    let ExpressionKind::Lambda(inner) = &outer.body.result.kind else {
        panic!("expected inner lambda");
    };
    assert_eq!(inner.captures[0].ty, Type::Int32);
    assert_eq!(inner.parameters[0].ty, Type::Int32);
}

#[test]
fn type_errors_keep_a_renderable_source_span() {
    let source = source("value :: Unit := 0;");
    let parsed = parse(&source).expect("parse");
    let resolved = resolve::resolve(&parsed).expect("resolve");
    let error = check::check(&resolved).expect_err("type mismatch");

    assert!(error.render(&source).contains("check-test.mal:1:18"));
}
