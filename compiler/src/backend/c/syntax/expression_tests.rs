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

#[test]
fn renders_storage_and_control_expressions() {
    let slot = Expr::add(
        Expr::identifier("storage"),
        Expr::multiply(Expr::identifier("index"), Expr::number("16")),
    );
    let expression = Expr::conditional(
        Expr::greater(Expr::identifier("count"), Expr::number("0")),
        slot.subscript(Expr::number("1")),
        Expr::sizeof_value(Expr::identifier("fallback")),
    );

    assert_eq!(
        expression.to_string(),
        "(count > 0) ? (storage + (index * 16))[1] : sizeof(fallback)"
    );
}
