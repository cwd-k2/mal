use super::*;

/// Operands that contain control are lowered through a lexical join, a different path from control-free operands, and
/// the operation then lives in a join body rather than the lambda body. Both paths must produce the `Buffer` form, because
/// later stages do not accept a `Buffer` primitive in the generic memory form.
#[test]
fn lowers_buffer_operations_with_control_operands_to_the_buffer_form() {
    let program = lower_ok(
        "Maybe :: [Unit, Int32];\n\
         update :: Buffer<Maybe> -> Unit := (options) -> {\n\
           options.new([none, some] => some(1i32));\n\
           options.put(0usize, [none, some] => some(2i32));\n\
           options.fill(0usize, 1usize, [none, some] => none());\n\
         };",
    );
    let text = format!("{:?}", top_lambda_definition(&program, "update"));

    assert!(
        text.contains("Goto"),
        "operands lower through a join: {text}"
    );
    assert!(!text.contains("primitive: Buffer"), "{text}");
    for operation in ["New", "Put", "Fill"] {
        assert_eq!(
            text.matches(&format!("operation: {operation}")).count(),
            1,
            "{operation} in {text}"
        );
    }
}

#[test]
fn lowers_buffer_operations_with_control_free_operands_to_the_same_form() {
    let program = lower_ok(
        "update :: Buffer<Int32> -> Unit := (values) -> {\n\
           values.new(1i32);\n\
           values.put(0usize, 2i32);\n\
         };",
    );
    let text = format!("{:?}", top_lambda_definition(&program, "update"));

    assert!(!text.contains("primitive: Buffer"), "{text}");
    assert!(text.contains("operation: New") && text.contains("operation: Put"));
}
