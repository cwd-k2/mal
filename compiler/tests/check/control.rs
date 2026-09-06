use super::*;

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
