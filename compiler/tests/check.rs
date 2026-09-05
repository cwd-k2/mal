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
fn checks_the_m0_host_example_end_to_end_through_typed_ast() {
    let program = check_ok(
        "extern printInt32 :: Int32 -> Unit;\n\
         main :: Unit -> Int32 := \\() {\n\
           extern printInt32(42);\n\
           return 0;\n\
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
           return if (flag) then { 1 } else { 0 };\n\
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
         maximum := 2147483647Int32;",
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
        ("127Int8", Type::Int8, 127_i128),
        ("32767Int16", Type::Int16, 32_767),
        ("2147483647Int32", Type::Int32, 2_147_483_647),
        (
            "9223372036854775807Int64",
            Type::Int64,
            9_223_372_036_854_775_807,
        ),
        ("255UInt8", Type::UInt8, 255),
        ("65535UInt16", Type::UInt16, 65_535),
        ("4294967295UInt32", Type::UInt32, 4_294_967_295),
        (
            "18446744073709551615UInt64",
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

    let byte = check_ok(r"value := b'\xff';");
    assert_eq!(top_binding(&byte, 0).value.ty, Type::UInt8);
    assert!(matches!(
        top_binding(&byte, 0).value.kind,
        ExpressionKind::Integer(255)
    ));

    for (literal, value) in [
        ("-128Int8", -128_i128),
        ("-32768Int16", -32_768),
        ("-2147483648Int32", -2_147_483_648),
        ("-9223372036854775808Int64", -9_223_372_036_854_775_808),
    ] {
        let program = check_ok(&format!("value := {literal};"));
        assert!(
            matches!(top_binding(&program, 0).value.kind, ExpressionKind::Integer(actual) if actual == value),
            "literal: {literal}"
        );
    }

    for literal in [
        "128Int8",
        "32768Int16",
        "2147483648Int32",
        "9223372036854775808Int64",
        "256UInt8",
        "65536UInt16",
        "4294967296UInt32",
        "18446744073709551616UInt64",
        "-129Int8",
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
         maximum := 340282346638528859811704183484516925440f32;",
    );
    let expected = [
        (Type::Float64, 0x3fb9_9999_9999_999a),
        (Type::Float32, 0x3dcc_cccd),
        (Type::Float32, 0x3f80_0000),
        (Type::Float32, 0x3f80_0002),
        (Type::Float32, 0x0000_0001),
        (Type::Float32, 0x7f7f_ffff),
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
           return result >= 0.0f32 && result != value;\n\
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
fn checks_string_literals_as_immutable_bytes() {
    let program = check_ok(r#"empty :: String := ""; bytes := "あ\0\xff";"#);
    assert_eq!(top_binding(&program, 0).value.ty, Type::String);
    assert!(matches!(
        top_binding(&program, 1).value.kind,
        ExpressionKind::String(ref value) if value == &[0xe3, 0x81, 0x82, 0, 255]
    ));
}

#[test]
fn checks_string_primitives_and_byte_wise_equality() {
    let program = check_ok(
        r#"length :: String -> UInt64 := \(value :: String) { return byteLength(value); };
item :: (String, UInt64) -> UInt8 := \(value :: String, index :: UInt64) {
  return byteAt(value, index);
};
same :: Unit -> Bool := \() { return "a\0" == "a\x00"; };
different :: Unit -> Bool := \() { return "a" != "b"; };"#,
    );
    let ExpressionKind::Lambda(length) = &top_binding(&program, 0).value.kind else {
        panic!("expected lambda");
    };
    assert!(matches!(
        length.body.result.kind,
        ExpressionKind::StringLength { .. }
    ));
    let ExpressionKind::Lambda(item) = &top_binding(&program, 1).value.kind else {
        panic!("expected lambda");
    };
    assert!(matches!(
        item.body.result.kind,
        ExpressionKind::StringAt { .. }
    ));
    for index in 2..=3 {
        let Type::Function { result, .. } = &top_binding(&program, index).value.ty else {
            panic!("expected function type");
        };
        assert_eq!(result.as_ref(), &Type::Sum(vec![Type::Unit, Type::Unit]));
    }
}

#[test]
fn rejects_unsupported_or_mistyped_string_operations() {
    for text in [
        r#"bad := "a" + "b";"#,
        r#"bad := "a" < "b";"#,
        r#"bad := byteLength(1);"#,
        r#"bad := byteAt("a", 0UInt8);"#,
        "bad := byteLength;",
    ] {
        let error = check_error(text);
        assert!(error.primary.is_some(), "input: {text}");
    }
}

#[test]
fn rejects_binding_and_return_type_mismatches() {
    assert_eq!(check_error("value :: Unit := 0;").message, "type mismatch");
    assert_eq!(
        check_error("main :: Unit -> Unit := \\() { return 0; };").message,
        "type mismatch"
    );
}

#[test]
fn checks_function_application_and_zero_argument_unit_lowering() {
    let program = check_ok(
        "identity :: Int32 -> Int32 := \\(x :: Int32) { return x; };\n\
         thunk :: Unit -> Int32 := \\() { return identity(4); };\n\
         caller :: Unit -> Int32 := \\() { return thunk(); };",
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
            "identity :: Int32 -> Int32 := \\(x :: Int32) { return x; };\n\
             bad :: Int32 := identity();"
        )
        .message,
        "type mismatch"
    );
    assert_eq!(
        check_error("value :: Int32 := 1Int32();").message,
        "cannot call a non-function value"
    );
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
           return case value { [0](_) => 0; [1](x) => x; };\n\
         };",
    );

    let prefix = "Maybe :: [Unit, Int32]; get :: Maybe -> Int32 := \\(value :: Maybe) { return ";
    assert_eq!(
        check_error(&format!("{prefix}case value {{ [0](_) => 0; }}; }};")).message,
        "non-exhaustive case expression"
    );
    assert!(
        check_error(&format!(
            "{prefix}case value {{ [0](_) => 0; [0](_) => 1; [1](x) => x; }}; }};"
        ))
        .message
        .starts_with("duplicate case arm")
    );
    assert_eq!(
        check_error(&format!(
            "{prefix}case value {{ [0](_) => (); [1](x) => x; }}; }};"
        ))
        .message,
        "type mismatch"
    );
}

#[test]
fn checks_if_condition_and_branch_types() {
    check_ok(
        "choose :: Bool -> Int32 := \\(condition :: Bool) {\n\
           return if (condition) then { 1 } else { 2 };\n\
         };",
    );
    assert_eq!(
        check_error(
            "bad :: Int32 -> Int32 := \\(condition :: Int32) {\n\
               return if (condition) then { 1 } else { 2 };\n\
             };"
        )
        .message,
        "type mismatch"
    );
    assert_eq!(
        check_error(
            "bad :: Bool -> Int32 := \\(condition :: Bool) {\n\
               return if (condition) then { 1 } else { () };\n\
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
           return !(x + 1 < 2) || false && (x == 0);\n\
         };",
    );
    let Type::Function { result, .. } = &top_binding(&program, 0).value.ty else {
        panic!("expected function type");
    };
    assert_eq!(result.as_ref(), &Type::Sum(vec![Type::Unit, Type::Unit]));

    assert_eq!(
        check_error("bad :: Int32 -> Int32 := \\(x :: Int32) { return x + true; };").message,
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
               return ((~x + 1) * 2 - 1) / 1 % 1 << 0 >> 0 & x | x ^ x;\n\
             }};\n\
             compare :: {name} -> Bool := \\(x :: {name}) {{\n\
               return x < x || x <= x || x > x || x >= x || x == x || x != x;\n\
             }};"
        ));
    }

    assert_eq!(
        check_error("bad := 1Int8 + 1UInt8;").message,
        "type mismatch"
    );
    assert!(
        check_error("bad := ~();")
            .message
            .contains("requires an integer")
    );
}

#[test]
fn checks_modulo_integer_conversions() {
    for (expression, expected) in [
        ("UInt8(-1Int8)", Type::UInt8),
        ("Int8(255UInt16)", Type::Int8),
        ("UInt16(-1Int8)", Type::UInt16),
        ("Int16(255UInt8)", Type::Int16),
    ] {
        let program = check_ok(&format!("value := {expression};"));
        assert_eq!(top_binding(&program, 0).value.ty, expected);
        assert!(matches!(
            top_binding(&program, 0).value.kind,
            ExpressionKind::IntegerConversion { .. }
        ));
    }

    assert!(
        check_error("value := Int8(());")
            .message
            .contains("requires an integer value")
    );
    assert!(
        check_error("value := Bool(1Int8);")
            .message
            .contains("requires an integer type")
    );
}

#[test]
fn checks_products_destructuring_and_multiple_parameters() {
    let program = check_ok(
        "Pair :: (Int32, UInt8);\n\
         pair :: Pair := (1, 2);\n\
         add :: (Int32, Int32) -> Int32 := \\(left :: Int32, right :: Int32) {\n\
           return left + right;\n\
         };\n\
         main :: Unit -> Int32 := \\() {\n\
           (first, _) := pair;\n\
           nested := ((first, 2Int32), 39Int32);\n\
           ((left, right), extra) := nested;\n\
           return add(left + right, extra);\n\
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
           mem := extern allocate(4UInt64);\n\
           return Int32(extern length(mem));\n\
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
               return 0;\n\
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
           return if (n == 0) then { 0 } else { count(n - 1) };\n\
         };\n\
         main :: Unit -> Int32 := \\() {\n\
           local :: Int32 -> Int32 := \\(n :: Int32) {\n\
             return if (n == 0) then { 0 } else { local(n - 1) };\n\
           };\n\
           return local(10);\n\
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
    let error = check_error("looping :: Int32 -> Int32 := \\(n :: Int64) { return looping(n); };");
    assert_eq!(error.message, "type mismatch");
}

#[test]
fn propagates_types_through_capture_bindings() {
    let program = check_ok(
        "make :: Int32 -> (Int32 -> Int32) := \\(x :: Int32) {\n\
           return \\<x>(y :: Int32) { return x + y; };\n\
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
