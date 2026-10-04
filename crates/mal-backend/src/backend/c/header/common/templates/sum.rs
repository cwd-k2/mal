//! Sum representation conversion and public constructor templates.

use crate::backend::c::syntax::{
    Directive, Expr, FunctionDefinition, SwitchCase, TranslationUnit, c_block, c_expr,
    c_initializers, c_invocation, c_signature, c_switch_cases, c_type,
};

pub(super) fn append_sum_conversion_template(output: &mut TranslationUnit) {
    let converted = |converter: &str| c_expr!({ converter }(call, value.payload.member));
    output.push(Directive::switch_cases_define(
        "MAL_DETAIL_SUM_TO_HOST_CASE",
        [
            "result_type",
            "variant_tag",
            "member",
            "raw_type",
            "host_type",
            "to_host",
            "to_raw",
        ],
        c_switch_cases! {
            UINT32_C(variant_tag) => {
                return { sum_conversion_result(converted("to_host")) };
            },
        },
    ));
    output.push(Directive::switch_cases_define(
        "MAL_DETAIL_SUM_TO_RAW_CASE",
        [
            "result_type",
            "variant_tag",
            "member",
            "raw_type",
            "host_type",
            "to_host",
            "to_raw",
        ],
        c_switch_cases! {
            UINT32_C(variant_tag) => {
                return { sum_conversion_result(converted("to_raw")) };
            },
        },
    ));
    let conversion = |name: &str, result: &str, value: &str, case: &str| {
        let cases: [SwitchCase; 1] = [c_invocation!(members({ case }, { result })).into()];
        let body = c_block! {
            match value.tag {
                ..{ cases },
                _ => { mal_call_trap(call, "invalid sum tag"); },
            }
        };
        FunctionDefinition::from_signature(
            c_signature! {
                #[static] #[inline] fn { name }(
                    call: *mut mal_call_t,
                    value: { value },
                ) -> { result }
            },
            body,
        )
    };
    let to_host = conversion(
        "to_host_name",
        "host_type",
        "raw_type",
        "MAL_DETAIL_SUM_TO_HOST_CASE",
    );
    let to_raw = conversion(
        "to_raw_name",
        "raw_type",
        "host_type",
        "MAL_DETAIL_SUM_TO_RAW_CASE",
    );
    output.push(Directive::function_definitions_define(
        "MAL_DETAIL_DEFINE_SUM_CONVERSIONS",
        [
            "to_host_name",
            "to_raw_name",
            "raw_type",
            "host_type",
            "members",
        ],
        [to_host, to_raw],
    ));
}

pub(super) fn append_sum_api_templates(output: &mut TranslationUnit) {
    let unit_value = sum_api_value(c_expr!(mal_Unit_t { _0: 0 }));
    let unit_make = FunctionDefinition::from_signature(
        c_signature!(#[static] #[inline] fn { "make_name" }() -> host_type),
        c_block!(return { unit_value };),
    );
    let unit_return = FunctionDefinition::from_signature(
        c_signature! {
            #[static] #[inline] fn { "return_name" }(
                call: *mut mal_call_t,
            ) -> raw_type
        },
        c_block!(return to_raw(call, make_name());),
    );
    output.push(Directive::function_definitions_define(
        "MAL_DETAIL_DEFINE_SUM_UNIT_API",
        [
            "make_name",
            "return_name",
            "host_type",
            "raw_type",
            "tag_name",
            "member",
            "to_raw",
        ],
        [unit_make, unit_return],
    ));

    let value_value = sum_api_value(c_expr!(value));
    let value_make = FunctionDefinition::from_signature(
        c_signature! {
            #[static] #[inline] fn { "make_name" }(
                value: value_type,
            ) -> host_type
        },
        c_block!(return { value_value };),
    );
    let value_return = FunctionDefinition::from_signature(
        c_signature! {
            #[static] #[inline] fn { "return_name" }(
                call: *mut mal_call_t,
                value: value_type,
            ) -> raw_type
        },
        c_block! { return to_raw(call, make_name(value)); },
    );
    output.push(Directive::function_definitions_define(
        "MAL_DETAIL_DEFINE_SUM_VALUE_API",
        [
            "make_name",
            "return_name",
            "host_type",
            "raw_type",
            "value_type",
            "tag_name",
            "member",
            "to_raw",
        ],
        [value_make, value_return],
    ));
}

fn sum_conversion_result(value: Expr) -> Expr {
    let initializers = c_initializers! {
        tag: UINT32_C(variant_tag),
        payload.member: { value },
    };
    c_expr!({ c_type!(result_type) } { ..{ initializers } })
}

fn sum_api_value(value: Expr) -> Expr {
    let initializers = c_initializers! {
        tag: tag_name,
        payload.member: { value },
    };
    c_expr!({ c_type!(host_type) } { ..{ initializers } })
}
