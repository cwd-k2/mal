use super::expression::{Expr, Initializer};

#[test]
fn renders_composed_expressions() {
    let expression = Expr::named_call(
        "consume",
        [
            Expr::identifier("object").pointer_field("value"),
            Expr::cast(
                "uint64_t",
                Expr::add(Expr::identifier("left"), Expr::identifier("right")),
            ),
        ],
    );

    assert_eq!(
        expression.to_string(),
        "consume(object->value, (uint64_t)(left + right))"
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
fn renders_character_literals() {
    assert_eq!(Expr::character('\n').to_string(), "'\\n'");
    assert_eq!(Expr::character('\'').to_string(), "'\\''");
}

#[test]
fn preserves_nested_unary_and_conditional_structure() {
    assert_eq!(
        Expr::negate(Expr::negate(Expr::identifier("value"))).to_string(),
        "-(-value)"
    );
    assert_eq!(
        Expr::conditional(
            Expr::conditional(
                Expr::identifier("first"),
                Expr::identifier("second"),
                Expr::identifier("third"),
            ),
            Expr::identifier("fourth"),
            Expr::identifier("fifth"),
        )
        .to_string(),
        "(first ? second : third) ? fourth : fifth"
    );
}
