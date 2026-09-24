use crate::backend::c::syntax::{
    Block, Expr, FunctionDefinition, FunctionSignature, Parameter, Statement,
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
