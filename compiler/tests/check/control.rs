use super::*;

#[test]
fn checks_sum_injection_payload_and_index() {
    let program = check_ok(
        "Maybe :: [Unit, Int32];\n\
         some :: Maybe := 1[Maybe](42);",
    );
    assert_eq!(
        top_binding(&program, 1).value.ty,
        Type::Sum(vec![Type::Unit, Type::Int32])
    );

    assert_eq!(
        check_error("Maybe :: [Unit, Int32]; bad :: Maybe := 2[Maybe](0);").message,
        "sum variant index is out of range"
    );
    assert_eq!(
        check_error("Maybe :: [Unit, Int32]; bad :: Maybe := 0[Maybe](0);").message,
        "type mismatch"
    );
}

#[test]
fn checks_sum_elimination_arity_and_result_types() {
    check_ok(
        "Maybe :: [Unit, Int32];\n\
         get :: Maybe -> Int32 := (value) {\n\
           value[\n\
             () { 0 },\n\
             (x) { y := x; y }];\n\
         };",
    );

    let prefix = "Maybe :: [Unit, Int32]; get :: Maybe -> Int32 := (value) { ";
    assert_eq!(
        check_error(&format!(
            "{prefix}value[() {{ 0 }}, (x) {{ x }}, (_) {{ 1 }}]; }};"
        ))
        .message,
        "sum continuation count does not match its type"
    );
    assert_eq!(
        check_error(&format!("{prefix}value[() {{ () }}, (x) {{ x }}]; }};")).message,
        "type mismatch"
    );
}

#[test]
fn checks_if_condition_and_branch_types() {
    check_ok(
        "choose :: Bool -> Int32 := (condition) {\n\
           if (condition) then { 1 } else { 2 };\n\
         };",
    );
    assert_eq!(
        check_error(
            "bad :: Int32 -> Int32 := (condition) {\n\
               if (condition) then { 1 } else { 2 };\n\
             };"
        )
        .message,
        "type mismatch"
    );
    assert_eq!(
        check_error(
            "bad :: Bool -> Int32 := (condition) {\n\
               if (condition) then { 1 } else { () };\n\
             };"
        )
        .message,
        "type mismatch"
    );
}
