use super::{AggregateDefinition, AggregateField, AggregateKind, Comment};

#[test]
fn comments_cannot_terminate_their_own_delimiter() {
    assert_eq!(
        Comment::new("source */ injected").render(),
        "/* source * / injected */\n"
    );
}

#[test]
fn renders_nested_aggregate_definitions() {
    let definition = AggregateDefinition::structure(
        "Value",
        [
            AggregateField::variable("uint32_t", "tag"),
            AggregateField::aggregate(
                AggregateKind::Union,
                [AggregateField::variable("int32_t", "integer")],
                "payload",
            ),
        ],
    );

    assert_eq!(
        definition.render(),
        "struct Value {\n    uint32_t tag;\n    union {\n        int32_t integer;\n    } payload;\n};\n"
    );
}
