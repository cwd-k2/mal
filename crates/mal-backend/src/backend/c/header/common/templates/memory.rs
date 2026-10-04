//! Canonical product and sum memory-access templates.

use crate::backend::c::syntax::{
    Directive, Expr, FunctionDefinition, Statement, SwitchCase, TranslationUnit, c_block, c_expr,
    c_initializers, c_invocation, c_signature, c_statement, c_switch_cases, c_type,
};

pub(super) fn append_product_memory_template(output: &mut TranslationUnit) {
    let member = c_expr!(value.member);
    let unit = c_expr!(mal_Unit_t { _0: 0 });
    output.push(Directive::statements_define(
        "MAL_DETAIL_MEMORY_PRODUCT_READ_UNIT",
        ["member"],
        [c_statement!({ member.clone() } = { unit };)],
    ));
    output.push(Directive::statements_define(
        "MAL_DETAIL_MEMORY_PRODUCT_READ_VALUE",
        ["member", "reader", "writer", "offset"],
        [c_statement!({ member.clone() } = reader(call, source + offset);)],
    ));
    output.push(Directive::statements_define(
        "MAL_DETAIL_MEMORY_PRODUCT_WRITE_UNIT",
        ["member"],
        [c_statement!({ member.clone() } as void;)],
    ));
    output.push(Directive::statements_define(
        "MAL_DETAIL_MEMORY_PRODUCT_WRITE_VALUE",
        ["member", "reader", "writer", "offset"],
        [c_statement!(writer(call, destination + offset, { member });)],
    ));

    let read_fields: Statement = c_invocation!(fields(
        MAL_DETAIL_MEMORY_PRODUCT_READ_UNIT,
        MAL_DETAIL_MEMORY_PRODUCT_READ_VALUE,
    ))
    .into();
    let read_body = c_block! {
        let value: value_type;
        { read_fields };
        return value;
    };
    let read = FunctionDefinition::from_signature(
        c_signature! {
            #[static] #[inline] fn { "read_name" }(
                #[maybe_unused] call: *mut mal_call_t,
                #[maybe_unused] source: *const uint8_t,
            ) -> value_type
        },
        read_body,
    );
    let write_fields: Statement = c_invocation!(fields(
        MAL_DETAIL_MEMORY_PRODUCT_WRITE_UNIT,
        MAL_DETAIL_MEMORY_PRODUCT_WRITE_VALUE,
    ))
    .into();
    let write_body = c_block! { { write_fields }; };
    let write = FunctionDefinition::from_signature(
        c_signature! {
            #[static] #[inline] fn { "write_name" }(
                #[maybe_unused] call: *mut mal_call_t,
                #[maybe_unused] destination: *mut uint8_t,
                value: value_type,
            ) -> void
        },
        write_body,
    );
    output.push(Directive::function_definitions_define(
        "MAL_DETAIL_DEFINE_MEMORY_PRODUCT",
        ["read_name", "write_name", "value_type", "fields"],
        [read, write],
    ));
}

pub(super) fn append_sum_memory_template(output: &mut TranslationUnit) {
    let tag = c_expr!(UINT32_C(variant_tag));
    let payload = c_expr!(reader(call, source + offset));
    output.push(Directive::switch_cases_define(
        "MAL_DETAIL_MEMORY_SUM_READ_CASE",
        [
            "value_type",
            "tag_type",
            "tag_writer",
            "variant_tag",
            "member",
            "reader",
            "writer",
            "offset",
        ],
        c_switch_cases! {
            variant_tag => { return { sum_memory_value(tag.clone(), payload) }; },
        },
    ));
    let host_tag = c_expr!(value.tag);
    let host_payload = c_expr!(value.payload.member);
    output.push(Directive::switch_cases_define(
        "MAL_DETAIL_MEMORY_SUM_WRITE_CASE",
        [
            "value_type",
            "tag_type",
            "tag_writer",
            "variant_tag",
            "member",
            "reader",
            "writer",
            "offset",
        ],
        c_switch_cases! {
            { tag.clone() } => {
                tag_writer(call, destination, { host_tag } as tag_type);
                writer(call, destination + offset, { host_payload });
                return;
            },
        },
    ));

    let read_cases: [SwitchCase; 1] =
        [c_invocation!(members(MAL_DETAIL_MEMORY_SUM_READ_CASE)).into()];
    let read_body = c_block! {
        match tag_reader(call, source) {
            ..{ read_cases },
            _ => {
                mal_call_trap(call, "invalid canonical sum tag");
            },
        }
    };
    let read = FunctionDefinition::from_signature(
        c_signature! {
            #[static] #[inline] fn { "read_name" }(
                #[maybe_unused] call: *mut mal_call_t,
                #[maybe_unused] source: *const uint8_t,
            ) -> value_type
        },
        read_body,
    );
    let write_cases: [SwitchCase; 1] =
        [c_invocation!(members(MAL_DETAIL_MEMORY_SUM_WRITE_CASE)).into()];
    let write_body = c_block! {
        match value.tag {
            ..{ write_cases },
            _ => { mal_call_trap(call, "invalid sum tag"); },
        }
    };
    let write = FunctionDefinition::from_signature(
        c_signature! {
            #[static] #[inline] fn { "write_name" }(
                #[maybe_unused] call: *mut mal_call_t,
                #[maybe_unused] destination: *mut uint8_t,
                value: value_type,
            ) -> void
        },
        write_body,
    );
    output.push(Directive::function_definitions_define(
        "MAL_DETAIL_DEFINE_MEMORY_SUM",
        [
            "read_name",
            "write_name",
            "value_type",
            "tag_reader",
            "members",
        ],
        [read, write],
    ));
}

fn sum_memory_value(tag: Expr, payload: Expr) -> Expr {
    let initializers = c_initializers! {
        tag: { tag },
        payload.member: { payload },
    };
    c_expr!({ c_type!(value_type) } { ..{ initializers } })
}
