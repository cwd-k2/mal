//! Sum representation conversion templates.

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

fn sum_conversion_result(value: Expr) -> Expr {
    let initializers = c_initializers! {
        tag: UINT32_C(variant_tag),
        payload.member: { value },
    };
    c_expr!({ c_type!(result_type) } { ..{ initializers } })
}
