use super::*;

#[test]
fn pratt_parser_preserves_precedence_and_left_associativity() {
    let expression = binding_value("value := 1 + 2 * 3 - 4;");
    let Expression::Binary {
        operator,
        left,
        right,
    } = expression
    else {
        panic!("expected subtraction");
    };
    assert_eq!(operator.kind, BinaryOperator::Subtract);
    assert!(matches!(right.kind, Expression::Integer(_)));
    let Expression::Binary {
        operator, right, ..
    } = left.kind
    else {
        panic!("expected addition on the left");
    };
    assert_eq!(operator.kind, BinaryOperator::Add);
    assert!(matches!(
        right.kind,
        Expression::Binary {
            operator: malc::ast::Node {
                kind: BinaryOperator::Multiply,
                ..
            },
            ..
        }
    ));
}

#[test]
fn calls_bind_more_tightly_than_unary_operators() {
    let expression = binding_value("value := -make()(1);");
    let Expression::Unary { operator, operand } = expression else {
        panic!("expected unary expression");
    };
    assert_eq!(operator.kind, UnaryOperator::Negate);
    let Expression::Call { callee, .. } = operand.kind else {
        panic!("expected outer call");
    };
    assert!(matches!(callee.kind, Expression::Call { .. }));
}

#[test]
fn parses_symbol_length_and_byte_access_with_access_precedence() {
    let Expression::Unary { operator, operand } = binding_value(r#"value := #make();"#) else {
        panic!("expected Symbol length");
    };
    assert_eq!(operator.kind, UnaryOperator::SymbolLength);
    assert!(matches!(operand.kind, Expression::Call { .. }));

    let Expression::Binary {
        operator,
        left,
        right,
    } = binding_value(r#"value := "abc" # 1u64 + 2u8;"#)
    else {
        panic!("expected addition");
    };
    assert_eq!(operator.kind, BinaryOperator::Add);
    assert!(matches!(
        left.kind,
        Expression::Binary {
            operator: malc::ast::Node {
                kind: BinaryOperator::SymbolAt,
                ..
            },
            ..
        }
    ));
    assert!(matches!(right.kind, Expression::Integer(_)));
}

#[test]
fn rejects_chained_symbol_byte_access() {
    let source = source(r#"value := "abc" # 0u64 # 1u64;"#);
    let error = parse(&source).expect_err("Symbol byte access must be non-associative");
    assert_eq!(error.message, "non-associative operator chain");
}

#[test]
fn rejects_non_associative_operator_chains() {
    for text in ["value := a < b <= c;", "value := a == b != c;"] {
        let source = source(text);
        let error = parse(&source).expect_err("comparison chain should be rejected");
        assert_eq!(error.message, "non-associative operator chain");
    }
}

#[test]
fn parses_closed_shapes_memory_operators_and_postfix_chains() {
    let Expression::StrideQuery(shape) = binding_value("value := #(address, bytesize);") else {
        panic!("expected a stride query");
    };
    assert!(
        matches!(shape.kind, malc::ast::LayoutShape::Product(ref members) if members.len() == 2)
    );

    let Expression::Align(placement) = binding_value("value := address@u64@count!;") else {
        panic!("expected postfix alignment");
    };
    assert!(matches!(placement.kind, Expression::Placement { .. }));

    let Expression::Binary { operator, .. } = binding_value("value := cursor <- item;") else {
        panic!("expected store");
    };
    assert_eq!(operator.kind, BinaryOperator::Store);

    let Expression::Unary { operator, .. } = binding_value("value := <-cursor;") else {
        panic!("expected load");
    };
    assert_eq!(operator.kind, UnaryOperator::Load);
}

#[test]
fn keeps_comparison_and_shift_distinct_from_generic_delimiters() {
    let comparison = binding_value("value := left < right;");
    assert!(matches!(
        comparison,
        Expression::Binary {
            operator: malc::ast::Node {
                kind: BinaryOperator::Less,
                ..
            },
            ..
        }
    ));

    let shift = binding_value("value := left >> right;");
    assert!(matches!(
        shift,
        Expression::Binary {
            operator: malc::ast::Node {
                kind: BinaryOperator::ShiftRight,
                ..
            },
            ..
        }
    ));

    let generic = binding_value("value := identity<Pair<Int32>>(input);");
    let Expression::Call { callee, .. } = generic else {
        panic!("expected call")
    };
    assert!(matches!(callee.kind, Expression::GenericName { .. }));
}
