use super::{Directive, PastePart, PreprocessorExpr};

#[test]
fn renders_token_pasting_function_aliases() {
    let directive = Directive::function_alias(
        "MAL_TYPE",
        ["name"],
        [PastePart::text("MalType_"), PastePart::parameter("name")],
    );
    assert_eq!(
        directive.render(),
        "#define MAL_TYPE(name) MalType_##name\n"
    );
}

#[test]
fn renders_typed_condition_operators() {
    let condition = PreprocessorExpr::logical_and(
        PreprocessorExpr::defined("FEATURE"),
        PreprocessorExpr::not_equal(
            PreprocessorExpr::identifier("FEATURE"),
            PreprocessorExpr::integer(1),
        ),
    );

    assert_eq!(
        Directive::If(condition).render(),
        "#if defined(FEATURE) && FEATURE != 1\n"
    );
}

#[test]
fn validates_include_paths_at_the_dsl_boundary() {
    assert!(Directive::is_valid_quoted_include("generated/program.h"));
    assert!(!Directive::is_valid_quoted_include("bad\nheader.h"));
    assert!(!Directive::is_valid_quoted_include("bad\"header.h"));
}

#[test]
fn escapes_error_messages_as_string_literals() {
    assert_eq!(
        Directive::error("unsupported \"target\"\n").render(),
        "#error \"unsupported \\\"target\\\"\\n\"\n"
    );
}
