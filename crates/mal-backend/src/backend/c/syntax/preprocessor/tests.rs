use crate::backend::c::syntax::{
    Block, Expr, FunctionDefinition, FunctionSignature, Initializer, MacroInvocation, Parameter,
    RecordDefinition, RecordField, Statement,
};

use super::{Directive, PreprocessorExpr};

#[test]
fn renders_defined_conditions() {
    assert_eq!(
        Directive::If(PreprocessorExpr::defined("FEATURE")).render(),
        "#if defined(FEATURE)\n"
    );
}

#[test]
fn validates_include_paths_at_the_dsl_boundary() {
    assert!(Directive::is_valid_quoted_include("generated/program.h"));
    assert!(!Directive::is_valid_quoted_include("bad\nheader.h"));
    assert!(!Directive::is_valid_quoted_include("bad\"header.h"));
}

#[test]
fn renders_multiple_structured_function_items_in_a_macro() {
    let body_signature = FunctionSignature::static_function(
        "int32_t",
        "mal_detail_body",
        [Parameter::named("int32_t", "value")],
    );
    let wrapper = FunctionDefinition::from_signature(
        FunctionSignature::new(
            "int32_t",
            "mal_ext_value",
            [Parameter::named("int32_t", "value")],
        ),
        Block::new([Statement::return_value(Expr::named_call(
            "mal_detail_body",
            [Expr::identifier("value")],
        ))]),
    );

    let directive = Directive::function_items_define(
        "MAL_DEFINE_value",
        ["value"],
        [body_signature.clone()],
        [wrapper],
        body_signature,
    );

    assert_eq!(
        directive.render(),
        "#define MAL_DEFINE_value(value) \\\nstatic int32_t mal_detail_body(int32_t value); \\\nint32_t mal_ext_value(int32_t value) { \\\n    return mal_detail_body(value); \\\n} \\\nstatic int32_t mal_detail_body( \\\n    int32_t value \\\n)\n"
    );
}

#[test]
fn renders_aggregate_templates_without_raw_c_fragments() {
    let directive = Directive::record_define(
        "DEFINE_PRODUCT",
        ["tag", "fields", "field"],
        RecordDefinition::structure(
            "tag",
            [RecordField::from(MacroInvocation::new(
                "fields",
                [Expr::identifier("field")],
            ))],
        ),
    );

    assert_eq!(
        directive.render(),
        "#define DEFINE_PRODUCT(tag, fields, field) \\\nstruct tag { \\\n    fields(field) \\\n};\n"
    );
}

#[test]
fn renders_expression_and_initializer_replacements() {
    assert_eq!(
        Directive::expression_define("IDENTITY", ["value"], Expr::identifier("value")).render(),
        "#define IDENTITY(value) value\n"
    );
    let initializer = Initializer::designated(
        "member",
        Expr::named_call("convert", [Expr::identifier("value")]),
    );
    assert_eq!(
        Directive::initializers_define("FIELD", ["member"], [initializer]).render(),
        "#define FIELD(member) \\\n.member = convert(value),\n"
    );
}
