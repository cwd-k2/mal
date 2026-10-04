//! Aggregate representation and product conversion templates.

use crate::backend::c::syntax::{
    Directive, FunctionDefinition, TranslationUnit, c_block, c_expr, c_initializers, c_invocation,
    c_items, c_record, c_record_fields, c_signature, c_type,
};

pub(super) fn append_aggregate_templates(output: &mut TranslationUnit) {
    output.push(Directive::record_fields_define(
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
        c_record_fields! { member: raw_type },
    ));
    output.push(Directive::record_fields_define(
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
        c_record_fields! { member: host_type },
    ));

    let descriptor_field = || c_invocation!(fields(field, type_tag));
    output.push(Directive::record_define(
        "MAL_DETAIL_DEFINE_PRODUCT_REPR",
        ["type_tag", "fields", "field"],
        c_record! {
            struct type_tag {
                { descriptor_field() },
            }
        },
    ));
    let members = c_invocation!(members(member, type_tag));
    output.push(Directive::record_define(
        "MAL_DETAIL_DEFINE_SUM_REPR",
        ["type_tag", "members", "member"],
        c_record! {
            struct type_tag {
                tag: uint32_t,
                payload: union {
                    { members },
                },
            }
        },
    ));
    output.push(Directive::record_define(
        "MAL_DETAIL_DEFINE_EMPTY_SUM_REPR",
        ["type_tag"],
        c_record! {
            struct type_tag {
                tag: uint32_t,
            }
        },
    ));
    append_product_conversion_template(output);
    output.blank_line();
}

fn append_product_conversion_template(output: &mut TranslationUnit) {
    output.extend(c_items! { define!(MAL_DETAIL_REPR_IDENTITY(call, value) = value); });
    let converted = |converter: &str| {
        c_initializers! { member: { c_expr!({ converter }(call, value.member)) } }
            .into_iter()
            .next()
            .expect("one product conversion initializer")
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
        let initializers = c_initializers! {
            { c_invocation!(fields({ field }, { result })) },
        };
        FunctionDefinition::from_signature(
            c_signature! {
                #[static] #[inline] fn { name }(
                    #[maybe_unused] call: *mut mal_call_t,
                    value: { value },
                ) -> { result }
            },
            c_block! {
                return { c_expr!({ c_type!({ result }) } { ..{ initializers } }) };
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
}
