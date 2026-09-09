use super::*;

#[test]
fn lowers_if_to_false_then_true_case_arms() {
    let program = lower_ok(
        "choose :: Bool -> Int32 := \\(flag) {\n\
           if (flag) then { value :: Int32 := 1; value } else { 0 };\n\
         };",
    );
    let body = lambda_body(&program.bindings[0].value);
    let (_, arms) = case(body);

    assert_eq!(arms.iter().map(|arm| arm.index).collect::<Vec<_>>(), [0, 1]);
    assert!(matches!(arms[0].value.kind, ExpressionKind::Integer(0)));
    assert!(matches!(arms[1].value.kind, ExpressionKind::Let { .. }));
}

#[test]
fn lowers_direct_comparison_conditions_without_materializing_bool() {
    let program = lower_ok(
        "choose :: Int32 -> Int32 := \\(value) {\n\
           if (value < 10) then { 1 } else { 2 };\n\
         };",
    );
    let body = lambda_body(&program.bindings[0].value);
    let ExpressionKind::PrimitiveBranch {
        operator,
        otherwise,
        then,
        ..
    } = &body.kind
    else {
        panic!("expected a primitive branch, found {:#?}", body.kind);
    };
    assert_eq!(*operator, BinaryPrimitive::Less);
    assert!(matches!(otherwise.kind, ExpressionKind::Integer(2)));
    assert!(matches!(then.kind, ExpressionKind::Integer(1)));
}

#[test]
fn lowers_short_circuit_operators_without_eager_right_evaluation() {
    let and_program = lower_ok(
        "extern observe :: Bool -> Bool;\n\
         test :: Bool -> Bool := \\(flag) {\n\
           flag && extern observe(flag);\n\
         };",
    );
    let body = lambda_body(&and_program.bindings[0].value);
    let (_, arms) = case(body);
    assert!(!injected_bool(&arms[0].value));
    assert!(matches!(
        arms[1].value.kind,
        ExpressionKind::ExternalCall { .. }
    ));

    let or_program = lower_ok(
        "extern observe :: Bool -> Bool;\n\
         test :: Bool -> Bool := \\(flag) {\n\
           flag || extern observe(flag);\n\
         };",
    );
    let body = lambda_body(&or_program.bindings[0].value);
    let (_, arms) = case(body);
    assert!(matches!(
        arms[0].value.kind,
        ExpressionKind::ExternalCall { .. }
    ));
    assert!(injected_bool(&arms[1].value));
}

#[test]
fn removes_logical_not_but_retains_typed_numeric_primitives() {
    let program = lower_ok(
        "test :: Int32 -> Bool := \\(value) {\n\
           !(value + 1 < 3);\n\
         };",
    );
    let body = lambda_body(&program.bindings[0].value);
    let (comparison, arms) = case(body);
    assert_eq!(arms.len(), 2);
    assert!(injected_bool(&arms[0].value));
    assert!(!injected_bool(&arms[1].value));

    let ExpressionKind::PrimitiveBinary {
        operator: BinaryPrimitive::Less,
        left,
        ..
    } = &comparison.kind
    else {
        panic!("expected an Int32 comparison");
    };
    assert!(matches!(
        left.kind,
        ExpressionKind::PrimitiveBinary {
            operator: BinaryPrimitive::Add,
            ..
        }
    ));
}

#[test]
fn binds_both_bool_equality_operands_once_before_branching() {
    let program = lower_ok(
        "extern first :: Unit -> Bool;\n\
         extern second :: Unit -> Bool;\n\
         test :: Unit -> Bool := \\() {\n\
           extern first() == extern second();\n\
         };",
    );
    let first_let = lambda_body(&program.bindings[0].value);
    let ExpressionKind::Let {
        binding: first,
        body: second_let,
    } = &first_let.kind
    else {
        panic!("left operand should be bound first");
    };
    assert!(matches!(
        first.value.kind,
        ExpressionKind::ExternalCall { id, .. } if id == program.interface.externals[0].id
    ));
    let Pattern::Binding { id: first_id, .. } = first.pattern else {
        panic!("expected a synthetic left binding");
    };

    let ExpressionKind::Let {
        binding: second,
        body: comparison,
    } = &second_let.kind
    else {
        panic!("right operand should be bound second");
    };
    assert!(matches!(
        second.value.kind,
        ExpressionKind::ExternalCall { id, .. } if id == program.interface.externals[1].id
    ));
    let Pattern::Binding { id: second_id, .. } = second.pattern else {
        panic!("expected a synthetic right binding");
    };
    assert!(matches!(first_id, ValueId::Temporary(_)));
    assert!(matches!(second_id, ValueId::Temporary(_)));

    let (left_reference, arms) = case(comparison);
    assert!(matches!(
        left_reference.kind,
        ExpressionKind::Reference(id) if id == first_id
    ));
    for arm in arms {
        let (right_reference, _) = case(&arm.value);
        assert!(matches!(
            right_reference.kind,
            ExpressionKind::Reference(id) if id == second_id
        ));
    }
}
