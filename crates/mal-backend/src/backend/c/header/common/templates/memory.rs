//! Canonical product and sum memory-access templates.

use crate::backend::c::syntax::{
    Block, Directive, FunctionDefinition, MacroInvocation, Statement, SwitchCase, TranslationUnit,
    c_block, c_expr, c_signature, c_statement,
};

pub(super) fn append_product_memory_template(output: &mut TranslationUnit) {
    let member = c_expr!(field((id("value")), "member"));
    let unit = c_expr!(compound((named("mal_Unit_t")), [positional((number(0)))]));
    output.push(Directive::statements_define(
        "MAL_DETAIL_MEMORY_PRODUCT_READ_UNIT",
        ["member"],
        [c_statement!(assign(#{ member.clone() }, #{ unit });)],
    ));
    output.push(Directive::statements_define(
        "MAL_DETAIL_MEMORY_PRODUCT_READ_VALUE",
        ["member", "reader", "writer", "offset"],
        [c_statement! {
            assign(
                #{ member.clone() },
                (call("reader", [
                    id("call"),
                    add((id("source")), (id("offset"))),
                ]))
            );
        }],
    ));
    output.push(Directive::statements_define(
        "MAL_DETAIL_MEMORY_PRODUCT_WRITE_UNIT",
        ["member"],
        [c_statement!(cast((named("void")), #{ member.clone() });)],
    ));
    output.push(Directive::statements_define(
        "MAL_DETAIL_MEMORY_PRODUCT_WRITE_VALUE",
        ["member", "reader", "writer", "offset"],
        [c_statement! {
            call("writer", [
                id("call"),
                add((id("destination")), (id("offset"))),
                #{ member },
            ]);
        }],
    ));

    let mut read_body = Block::default();
    read_body.push(c_statement!(let "value": named("value_type");));
    read_body.push(Statement::macro_invocation(MacroInvocation::new(
        "fields",
        [
            c_expr!(id("MAL_DETAIL_MEMORY_PRODUCT_READ_UNIT")),
            c_expr!(id("MAL_DETAIL_MEMORY_PRODUCT_READ_VALUE")),
        ],
    )));
    read_body.push(c_statement!(return (id("value"));));
    let read = FunctionDefinition::from_signature(
        c_signature! {
            #[static] #[inline] fn "read_name"(
                #[maybe_unused] "call": ptr(named("mal_call_t")),
                #[maybe_unused] "source": ptr(const(named("uint8_t"))),
            ) -> named("value_type")
        },
        read_body,
    );
    let mut write_body = Block::default();
    write_body.push(Statement::macro_invocation(MacroInvocation::new(
        "fields",
        [
            c_expr!(id("MAL_DETAIL_MEMORY_PRODUCT_WRITE_UNIT")),
            c_expr!(id("MAL_DETAIL_MEMORY_PRODUCT_WRITE_VALUE")),
        ],
    )));
    let write = FunctionDefinition::from_signature(
        c_signature! {
            #[static] #[inline] fn "write_name"(
                #[maybe_unused] "call": ptr(named("mal_call_t")),
                #[maybe_unused] "destination": ptr(named("uint8_t")),
                "value": named("value_type"),
            ) -> named("void")
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
    let tag = c_expr!(call("UINT32_C", [id("variant_tag")]));
    let payload = c_expr! {
        call("reader", [
            id("call"),
            add((id("source")), (id("offset"))),
        ])
    };
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
        [SwitchCase::case(
            c_expr!(id("variant_tag")),
            c_block! {
                return (compound((named("value_type")), [
                    field("tag", #{ tag.clone() }),
                    path(#{ ["payload".to_string(), "member".to_string()] }, #{ payload }),
                ]));
            },
        )],
    ));
    let host_tag = c_expr!(field((id("value")), "tag"));
    let host_payload = c_expr!(field((field((id("value")), "payload")), "member"));
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
        [SwitchCase::case(
            tag.clone(),
            c_block! {
                call("tag_writer", [
                    id("call"),
                    id("destination"),
                    cast((named("tag_type")), #{ host_tag }),
                ]);
                call("writer", [
                    id("call"),
                    add((id("destination")), (id("offset"))),
                    #{ host_payload },
                ]);
                return;
            },
        )],
    ));

    let mut read_body = Block::default();
    read_body.push(Statement::switch(
        c_expr!(call("tag_reader", [id("call"), id("source")])),
        [
            SwitchCase::macro_invocation(MacroInvocation::new(
                "members",
                [c_expr!(id("MAL_DETAIL_MEMORY_SUM_READ_CASE"))],
            )),
            SwitchCase::default(c_block! {
                call("mal_call_trap", [
                    id("call"),
                    string("invalid canonical sum tag"),
                ]);
            }),
        ]
        .into(),
    ));
    let read = FunctionDefinition::from_signature(
        c_signature! {
            #[static] #[inline] fn "read_name"(
                #[maybe_unused] "call": ptr(named("mal_call_t")),
                #[maybe_unused] "source": ptr(const(named("uint8_t"))),
            ) -> named("value_type")
        },
        read_body,
    );
    let mut write_body = Block::default();
    write_body.push(Statement::switch(
        c_expr!(field((id("value")), "tag")),
        [
            SwitchCase::macro_invocation(MacroInvocation::new(
                "members",
                [c_expr!(id("MAL_DETAIL_MEMORY_SUM_WRITE_CASE"))],
            )),
            SwitchCase::default(c_block! {
                call("mal_call_trap", [id("call"), string("invalid sum tag")]);
            }),
        ]
        .into(),
    ));
    let write = FunctionDefinition::from_signature(
        c_signature! {
            #[static] #[inline] fn "write_name"(
                #[maybe_unused] "call": ptr(named("mal_call_t")),
                #[maybe_unused] "destination": ptr(named("uint8_t")),
                "value": named("value_type"),
            ) -> named("void")
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
