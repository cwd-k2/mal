//! Aggregate representation and product conversion templates.

use crate::backend::c::syntax::{
    AggregateDefinition, AggregateField, AggregateKind, Directive, FunctionDefinition, Initializer,
    MacroInvocation, TranslationUnit, c_aggregate_field, c_block, c_expr, c_function,
    c_initializer, c_signature, c_type,
};

pub(super) fn append_aggregate_templates(output: &mut TranslationUnit) {
    output.push(Directive::aggregate_fields_define(
        "MAL_DETAIL_RAW_REPR_FIELD",
        [
            "context",
            "index",
            "member",
            "raw_type",
            "host_type",
            "to_host",
            "to_raw",
        ],
        [c_aggregate_field!("member": named("raw_type"))],
    ));
    output.push(Directive::aggregate_fields_define(
        "MAL_DETAIL_HOST_REPR_FIELD",
        [
            "context",
            "index",
            "member",
            "raw_type",
            "host_type",
            "to_host",
            "to_raw",
        ],
        [c_aggregate_field!("member": named("host_type"))],
    ));

    let descriptor_fields = || {
        [AggregateField::macro_invocation(MacroInvocation::new(
            "fields",
            [c_expr!(id("field")), c_expr!(id("type_tag"))],
        ))]
    };
    output.push(Directive::aggregate_define(
        "MAL_DETAIL_DEFINE_PRODUCT_REPR",
        ["type_tag", "fields", "field"],
        AggregateDefinition::structure("type_tag", descriptor_fields()),
    ));
    output.push(Directive::aggregate_define(
        "MAL_DETAIL_DEFINE_SUM_REPR",
        ["type_tag", "members", "member"],
        AggregateDefinition::structure(
            "type_tag",
            [
                c_aggregate_field!("tag": named("uint32_t")),
                AggregateField::aggregate(
                    AggregateKind::Union,
                    [AggregateField::macro_invocation(MacroInvocation::new(
                        "members",
                        [c_expr!(id("member")), c_expr!(id("type_tag"))],
                    ))],
                    "payload",
                ),
            ],
        ),
    ));
    output.push(Directive::aggregate_define(
        "MAL_DETAIL_DEFINE_EMPTY_SUM_REPR",
        ["type_tag"],
        AggregateDefinition::structure("type_tag", [c_aggregate_field!("tag": named("uint32_t"))]),
    ));
    append_product_conversion_template(output);
    output.blank_line();
}

fn append_product_conversion_template(output: &mut TranslationUnit) {
    output.push(Directive::expression_define(
        "MAL_DETAIL_REPR_IDENTITY",
        ["call", "value"],
        c_expr!(id("value")),
    ));
    let converted = |converter: &str| {
        c_initializer! {
            field("member", (call(#{ converter }, [id("call"), field((id("value")), "member")])))
        }
    };
    output.push(Directive::initializers_define(
        "MAL_DETAIL_PRODUCT_TO_HOST_FIELD",
        [
            "context",
            "index",
            "member",
            "raw_type",
            "host_type",
            "to_host",
            "to_raw",
        ],
        [converted("to_host")],
    ));
    output.push(Directive::initializers_define(
        "MAL_DETAIL_PRODUCT_TO_RAW_FIELD",
        [
            "context",
            "index",
            "member",
            "raw_type",
            "host_type",
            "to_host",
            "to_raw",
        ],
        [converted("to_raw")],
    ));
    let conversion = |name: &str, result: &str, value: &str, field: &str| {
        let initializer = Initializer::macro_invocation(MacroInvocation::new(
            "fields",
            [c_expr!(id(#{ field })), c_expr!(id(#{ result }))],
        ));
        FunctionDefinition::from_signature(
            c_signature! {
                #[static] #[inline] fn #{ name }(
                    #[maybe_unused] "call": ptr(named("mal_call_t")),
                    "value": named(#{ value }),
                ) -> named(#{ result })
            },
            c_block! {
                return #{ crate::backend::c::syntax::Expr::compound_literal(
                    c_type!(named(#{ result })),
                    [initializer],
                ) };
            },
        )
    };
    output.push(Directive::function_definitions_define(
        "MAL_DETAIL_DEFINE_PRODUCT_CONVERSIONS",
        [
            "to_host_name",
            "to_raw_name",
            "raw_type",
            "host_type",
            "fields",
        ],
        [
            conversion(
                "to_host_name",
                "host_type",
                "raw_type",
                "MAL_DETAIL_PRODUCT_TO_HOST_FIELD",
            ),
            conversion(
                "to_raw_name",
                "raw_type",
                "host_type",
                "MAL_DETAIL_PRODUCT_TO_RAW_FIELD",
            ),
        ],
    ));
    let converting_return = c_function! {
        #[static] #[inline] fn "function_name"(
            #[maybe_unused] "call": ptr(named("mal_call_t")),
            "value": named("value_type"),
        ) -> named("result_type") {
            return (call("converter", [id("call"), id("value")]));
        }
    };
    output.push(Directive::function_definitions_define(
        "MAL_DETAIL_DEFINE_CONVERTING_RETURN",
        ["function_name", "result_type", "value_type", "converter"],
        [converting_return],
    ));
}
