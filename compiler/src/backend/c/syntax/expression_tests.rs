use super::expression::{Expr, Initializer};

#[test]
fn renders_composed_expressions() {
    let expression = Expr::named_call(
        "consume",
        [
            Expr::identifier("object").pointer_field("value"),
            Expr::cast(
                "uint8_t",
                Expr::equal(Expr::identifier("left"), Expr::identifier("right")),
            ),
        ],
    );

    assert_eq!(
        expression.to_string(),
        "consume(object->value, (uint8_t)(left == right))"
    );
}

#[test]
fn renders_compound_literals_with_designators() {
    let expression = Expr::compound_literal(
        "Pair",
        [
            Initializer::designated("first", Expr::identifier("left")),
            Initializer::positional(Expr::identifier("right")),
        ],
    );

    assert_eq!(expression.to_string(), "(Pair){ .first = left, right }");
}
