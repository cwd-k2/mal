use super::{AggregateDefinition, AggregateField, AggregateKind, Comment};
use crate::c_emit::syntax::{Expr, TypeName};

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

#[test]
fn renders_array_fields_without_smuggling_syntax_through_identifiers() {
    let definition = AggregateDefinition::typedef_structure(
        None,
        [AggregateField::array(
            TypeName::const_named("uint8_t"),
            "bytes",
            Expr::number("256"),
        )],
        "Buffer",
    );

    assert_eq!(
        definition.render(),
        "typedef struct { const uint8_t bytes[256]; } Buffer;\n"
    );
}
