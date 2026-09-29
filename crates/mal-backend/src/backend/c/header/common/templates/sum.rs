//! Sum representation conversion and public constructor templates.

use crate::backend::c::syntax::{
    Block, Directive, FunctionDefinition, MacroInvocation, Statement, SwitchCase, TranslationUnit,
    c_block, c_expr, c_signature,
};

pub(super) fn append_sum_conversion_template(output: &mut TranslationUnit) {
    let converted = |converter: &str| {
        c_expr! {
            call(#{ converter }, [
                id("call"),
                field((field((id("value")), "payload")), "member"),
            ])
        }
    };
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
        [SwitchCase::case(
            c_expr!(call("UINT32_C", [id("variant_tag")])),
            c_block! {
                return (compound((named("result_type")), [
                    field("tag", (call("UINT32_C", [id("variant_tag")]))),
                    path(
                        #{ ["payload".to_string(), "member".to_string()] },
                        #{ converted("to_host") }
                    ),
                ]));
            },
        )],
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
        [SwitchCase::case(
            c_expr!(call("UINT32_C", [id("variant_tag")])),
            c_block! {
                return (compound((named("result_type")), [
                    field("tag", (call("UINT32_C", [id("variant_tag")]))),
                    path(
                        #{ ["payload".to_string(), "member".to_string()] },
                        #{ converted("to_raw") }
                    ),
                ]));
            },
        )],
    ));
    let conversion = |name: &str, result: &str, value: &str, case: &str| {
        let mut body = Block::default();
        body.push(Statement::switch(
            c_expr!(field((id("value")), "tag")),
            [
                SwitchCase::macro_invocation(MacroInvocation::new(
                    "members",
                    [c_expr!(id(#{ case })), c_expr!(id(#{ result }))],
                )),
                SwitchCase::default(c_block! {
                    call("mal_call_trap", [id("call"), string("invalid sum tag")]);
                }),
            ]
            .into(),
        ));
        FunctionDefinition::from_signature(
            c_signature! {
                #[static] #[inline] fn #{ name }(
                    "call": ptr(named("mal_call_t")),
                    "value": named(#{ value }),
                ) -> named(#{ result })
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
    let unit_value = c_expr! {
        compound((named("host_type")), [
            field("tag", (id("tag_name"))),
            path(
                #{ ["payload".to_string(), "member".to_string()] },
                (compound((named("mal_Unit_t")), [positional((number(0)))]))
            ),
        ])
    };
    let unit_make = FunctionDefinition::from_signature(
        c_signature!(#[static] #[inline] fn "make_name"() -> named("host_type")),
        c_block!(return #{ unit_value };),
    );
    let unit_return = FunctionDefinition::from_signature(
        c_signature! {
            #[static] #[inline] fn "return_name"(
                "call": ptr(named("mal_call_t")),
            ) -> named("raw_type")
        },
        c_block!(return (call("to_raw", [id("call"), call("make_name", [])]));),
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

    let value_value = c_expr! {
        compound((named("host_type")), [
            field("tag", (id("tag_name"))),
            path(
                #{ ["payload".to_string(), "member".to_string()] },
                (id("value"))
            ),
        ])
    };
    let value_make = FunctionDefinition::from_signature(
        c_signature! {
            #[static] #[inline] fn "make_name"(
                "value": named("value_type"),
            ) -> named("host_type")
        },
        c_block!(return #{ value_value };),
    );
    let value_return = FunctionDefinition::from_signature(
        c_signature! {
            #[static] #[inline] fn "return_name"(
                "call": ptr(named("mal_call_t")),
                "value": named("value_type"),
            ) -> named("raw_type")
        },
        c_block! {
            return (call("to_raw", [id("call"), call("make_name", [id("value")])]));
        },
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
