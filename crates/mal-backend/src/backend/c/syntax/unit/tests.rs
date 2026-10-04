use super::{Comment, RecordDefinition, RecordField, RecordKind};
use crate::backend::c::syntax::{MacroInvocation, c_expr};

#[test]
fn comments_cannot_terminate_their_own_delimiter() {
    assert_eq!(
        Comment::new("source */ injected").render(),
        "/* source * / injected */\n"
    );
}

#[test]
fn renders_nested_aggregate_definitions() {
    let definition = RecordDefinition::structure(
        "Value",
        [
            RecordField::variable("uint32_t", "tag"),
            RecordField::record(
                RecordKind::Union,
                [RecordField::variable("int32_t", "integer")],
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
fn renders_macro_invocations_as_aggregate_fields() {
    let definition = RecordDefinition::structure(
        "Value",
        [RecordField::from(MacroInvocation::new(
            "fields",
            [c_expr!(field)],
        ))],
    );

    assert_eq!(
        definition.render(),
        "struct Value {\n    fields(field)\n};\n"
    );
}
