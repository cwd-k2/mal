use crate::backend::c::syntax::{
    AggregateDefinition, AggregateField, AggregateKind, Block, Directive, FunctionDefinition,
    Initializer, MacroInvocation, Statement, SwitchCase, TranslationUnit, c_aggregate,
    c_aggregate_field, c_block, c_comment, c_declaration, c_directive, c_expr, c_function,
    c_initializer, c_signature, c_statement, c_type,
};
use crate::backend::c::types::TypeRegistry;

pub(super) fn emit() -> String {
    let mut output = TranslationUnit::default();
    for directive in [c_directive!(ifndef "MAL_H"), c_directive!(define "MAL_H";)] {
        output.push(directive);
    }
    output.blank_line();
    output.push(c_directive!(include(system "stddef.h")));
    output.push(c_directive!(include(system "stdint.h")));
    output.push(c_directive!(include(system "limits.h")));
    output.push(c_directive!(include(system "float.h")));
    output.push(c_directive!(include(system "string.h")));
    output.blank_line();
    output.push(c_directive!(define "MAL_C_ABI_VERSION" = (number("0x000900u"));));
    output.blank_line();
    output.push(c_directive!(if (defined("__clang__"))));
    output.push(c_directive!(define "MAL_DETAIL_MAYBE_UNUSED" = unused;));
    output.push(c_directive!(else));
    output.push(c_directive!(define "MAL_DETAIL_MAYBE_UNUSED";));
    output.push(c_directive!(endif));
    output.blank_line();
    output.push(c_comment!("Runtime API"));
    output.blank_line();
    output.push(c_declaration!(type "MalContext" = struct("MalContext")));
    output.push(c_aggregate! {
        type "MalType_Unit" = struct {
            "unused": named("uint8_t"),
        }
    });
    for (source, alias) in [
        ("uint8_t", "MalType_Bool"),
        ("int8_t", "MalType_Int8"),
        ("int16_t", "MalType_Int16"),
        ("int32_t", "MalType_Int32"),
        ("int64_t", "MalType_Int64"),
        ("uint8_t", "MalType_UInt8"),
        ("uint16_t", "MalType_UInt16"),
        ("uint32_t", "MalType_UInt32"),
        ("uint64_t", "MalType_UInt64"),
        ("float", "MalType_Float32"),
        ("double", "MalType_Float64"),
        ("size_t", "MalType_ByteSize"),
        ("size_t", "MalType_USize"),
    ] {
        output.push(c_declaration!(type #{ alias } = #{ c_type!(named(#{ source })) }));
    }
    output.push(c_declaration!(type "MalType_Address" = ptr(named("void"))));
    let width_of = |ty: &str, bits: u32| {
        c_expr! {
            equal(
                (multiply(
                    (sizeof((cast(#{ c_type!(named(#{ ty })) }, (number(0)))))),
                    (id("CHAR_BIT"))
                )),
                (number(#{ bits }))
            )
        }
    };
    let macro_equals =
        |name: &str, value: &str| c_expr!(equal((id(#{ name })), (number(#{ value }))));
    for (condition, message) in [
        (
            c_expr! {
                logical_and(#{ width_of("float", 32) }, #{ macro_equals("FLT_MANT_DIG", "24") })
            },
            "float is not IEEE 754 binary32",
        ),
        (
            c_expr! {
                logical_and(#{ width_of("double", 64) }, #{ macro_equals("DBL_MANT_DIG", "53") })
            },
            "double is not IEEE 754 binary64",
        ),
        (
            c_expr! {
                logical_and(#{ macro_equals("FLT_HAS_SUBNORM", "1") }, #{ macro_equals("DBL_HAS_SUBNORM", "1") })
            },
            "the target does not preserve subnormal floating-point values",
        ),
        (
            macro_equals("FLT_EVAL_METHOD", "0"),
            "floating-point expressions are evaluated with extra precision",
        ),
    ] {
        output.push(c_declaration!(static_assert(#{ condition }, #{ message });));
    }
    for (source, alias) in [
        ("MalType_Unit", "mal_Unit_t"),
        ("MalType_Bool", "mal_Bool_t"),
        ("MalType_Int8", "mal_Int8_t"),
        ("MalType_Int16", "mal_Int16_t"),
        ("MalType_Int32", "mal_Int32_t"),
        ("MalType_Int64", "mal_Int64_t"),
        ("MalType_UInt8", "mal_UInt8_t"),
        ("MalType_UInt16", "mal_UInt16_t"),
        ("MalType_UInt32", "mal_UInt32_t"),
        ("MalType_UInt64", "mal_UInt64_t"),
        ("MalType_Float32", "mal_Float32_t"),
        ("MalType_Float64", "mal_Float64_t"),
        ("MalType_Address", "mal_Address_t"),
        ("MalType_ByteSize", "mal_ByteSize_t"),
        ("MalType_USize", "mal_USize_t"),
    ] {
        output.push(c_declaration!(type #{ alias } = #{ c_type!(named(#{ source })) }));
    }
    output.push(c_aggregate! {
        type "mal_call_t" = struct {
            "mal_detail_context": ptr(named("MalContext")),
        }
    });
    output.blank_line();
    output.push(c_directive! {
        define "mal_false" =
            (cast((named("mal_Bool_t")), (call("UINT8_C", [number(0)]))));
    });
    output.push(c_directive! {
        define "mal_true" =
            (cast((named("mal_Bool_t")), (call("UINT8_C", [number(1)]))));
    });
    output.blank_line();
    output.push(c_declaration! {
        fn #{
            c_signature! {
                #[noreturn] fn "mal_trap"(
                    "context": ptr(named("MalContext")),
                    "message": ptr(const(named("char"))),
                ) -> named("void")
            }
        };
    });
    output.push(c_function! {
        #[static] #[inline] #[noreturn] fn "mal_call_trap"(
            "call": ptr(named("mal_call_t")),
            "message": ptr(const(named("char"))),
        ) -> named("void") {
            call("mal_trap", [
                pointer_field((id("call")), "mal_detail_context"),
                id("message"),
            ]);
        }
    });
    append_builtin_returns(&mut output);
    output.push(c_function! {
        #[static] #[inline] fn "mal_detail_convert_Unit"(
            #[maybe_unused] "call": ptr(named("mal_call_t")),
            #[maybe_unused] "value": named("mal_Unit_t"),
        ) -> named("MalType_Unit") {
            return (compound((named("MalType_Unit")), [positional((number(0)))]));
        }
    });
    output.blank_line();
    output.push(c_comment!("Canonical scalar memory access"));
    output.blank_line();
    output.push(c_function! {
        #[static] #[inline] fn "mal_detail_memory_read_Unit"(
            #[maybe_unused] "call": ptr(named("mal_call_t")),
            #[maybe_unused] "source": ptr(const(named("uint8_t"))),
        ) -> named("mal_Unit_t") {
            return (compound((named("mal_Unit_t")), [positional((number(0)))]));
        }
    });
    output.push(c_function! {
        #[static] #[inline] fn "mal_detail_memory_write_Unit"(
            #[maybe_unused] "call": ptr(named("mal_call_t")),
            #[maybe_unused] "destination": ptr(named("uint8_t")),
            "value": named("mal_Unit_t"),
        ) -> named("void") {
            cast((named("void")), (id("value")));
        }
    });
    output.blank_line();
    output.extend(TypeRegistry::default().common_scalar_memory_helpers());
    output.blank_line();
    append_generated_header_templates(&mut output);
    output.blank_line();
    output.push(c_directive!(endif));
    output.render()
}

fn append_generated_header_templates(output: &mut TranslationUnit) {
    output.push(c_comment!("Generated header templates"));
    output.blank_line();
    append_aggregate_templates(output);
    let conversion = c_function! {
        #[static] #[inline] fn "function_name"(
            #[maybe_unused] "call": ptr(named("mal_call_t")),
            "value": named("value_type"),
        ) -> named("result_type") {
            return (id("conversion"));
        }
    };
    output.push(c_directive! {
        define_functions "MAL_DETAIL_DEFINE_CONVERSION" {
            parameters: #{ ["function_name", "result_type", "value_type", "conversion"] },
            definitions: #{ [conversion] },
        }
    });
    let memory_read = c_function! {
        #[static] #[inline] fn "read_name"(
            "call": ptr(named("mal_call_t")),
            "address": named("mal_Address_t"),
            "index": named("mal_USize_t"),
        ) -> named("value_type") {
            call("mal_Address_return", [id("call"), id("address")]);
            return (call("reader", [
                id("call"),
                add(
                    (cast(
                        (ptr(const(named("uint8_t")))),
                        (id("address"))
                    )),
                    (multiply((id("index")), (id("stride"))))
                ),
            ]));
        }
    };
    let memory_write = c_function! {
        #[static] #[inline] fn "write_name"(
            "call": ptr(named("mal_call_t")),
            "address": named("mal_Address_t"),
            "index": named("mal_USize_t"),
            "value": named("value_type"),
        ) -> named("void") {
            call("mal_Address_return", [id("call"), id("address")]);
            call("writer", [
                id("call"),
                add(
                    (cast((ptr(named("uint8_t"))), (id("address")))),
                    (multiply((id("index")), (id("stride"))))
                ),
                id("value"),
            ]);
        }
    };
    output.push(c_directive! {
        define_functions "MAL_DETAIL_DEFINE_MEMORY_ALIAS" {
            parameters: #{ [
                "read_name", "write_name", "value_type", "stride", "reader", "writer",
            ] },
            definitions: #{ [memory_read, memory_write] },
        }
    });
    append_product_memory_template(output);
    append_sum_memory_template(output);
    append_sum_conversion_template(output);
    append_sum_api_templates(output);
}

fn append_aggregate_templates(output: &mut TranslationUnit) {
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

fn append_product_memory_template(output: &mut TranslationUnit) {
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

fn append_sum_memory_template(output: &mut TranslationUnit) {
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

fn append_sum_conversion_template(output: &mut TranslationUnit) {
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

fn append_sum_api_templates(output: &mut TranslationUnit) {
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

fn append_builtin_returns(output: &mut TranslationUnit) {
    output.push(c_function! {
        #[static] #[inline] fn "mal_Unit_return"(
            #[maybe_unused] "call": ptr(named("mal_call_t")),
        ) -> named("MalType_Unit") {
            return (compound((named("MalType_Unit")), [
                field("unused", (call("UINT8_C", [number(0)]))),
            ]));
        }
    });
    for (raw, host, name) in [
        ("MalType_Int8", "mal_Int8_t", "Int8"),
        ("MalType_Int16", "mal_Int16_t", "Int16"),
        ("MalType_Int32", "mal_Int32_t", "Int32"),
        ("MalType_Int64", "mal_Int64_t", "Int64"),
        ("MalType_UInt8", "mal_UInt8_t", "UInt8"),
        ("MalType_UInt16", "mal_UInt16_t", "UInt16"),
        ("MalType_UInt32", "mal_UInt32_t", "UInt32"),
        ("MalType_UInt64", "mal_UInt64_t", "UInt64"),
        ("MalType_Float32", "mal_Float32_t", "Float32"),
        ("MalType_Float64", "mal_Float64_t", "Float64"),
        ("MalType_ByteSize", "mal_ByteSize_t", "ByteSize"),
        ("MalType_USize", "mal_USize_t", "USize"),
    ] {
        output.push(c_function! {
            #[static] #[inline] fn #{ format!("mal_{name}_return") }(
                #[maybe_unused] "call": ptr(named("mal_call_t")),
                "value": named(#{ host }),
            ) -> named(#{ raw }) {
                return (id("value"));
            }
        });
    }
    output.push(c_function! {
        #[static] #[inline] fn "mal_Address_return"(
            "call": ptr(named("mal_call_t")),
            "value": named("mal_Address_t"),
        ) -> named("MalType_Address") {
            if (equal((id("value")), (number(0)))) {
                call("mal_call_trap", [
                    id("call"),
                    string("invalid Address result"),
                ]);
            }
            return (id("value"));
        }
    });
    output.push(c_function! {
        #[static] #[inline] fn "mal_Bool_return"(
            "call": ptr(named("mal_call_t")),
            "value": named("mal_Bool_t"),
        ) -> named("MalType_Bool") {
            if (logical_and(
                (not_equal((id("value")), (id("mal_false")))),
                (not_equal((id("value")), (id("mal_true"))))
            )) {
                call("mal_call_trap", [
                    id("call"),
                    string("invalid Bool result"),
                ]);
            }
            return (id("value"));
        }
    });
}
