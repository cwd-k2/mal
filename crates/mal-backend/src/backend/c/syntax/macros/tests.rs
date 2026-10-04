use std::cell::Cell;

use crate::backend::c::syntax::{Expr, FunctionSignature, Initializer};

#[test]
fn composes_types_parameters_and_signatures_with_template_interpolation() {
    let name = "convert";
    let result = super::c_type!(uint32_t);
    let trailing = [super::c_parameter!(value: uint8_t)];
    let signature = super::c_signature! {
        #[static] #[inline] fn { name }(
            #[maybe_unused] call: *mut mal_call_t,
            ..{ trailing },
        ) -> { result }
    };

    assert_eq!(
        signature.render(),
        "static inline uint32_t convert(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, uint8_t value)"
    );
}

#[test]
fn parses_translation_unit_items_as_a_rust_subset() {
    let trailing = super::c_record_fields! {
        next: *mut Struct<Node>,
    };
    let unit = super::c_items! {
        include_system!(stdint.h);
        define!(WORD_BITS = 32);
        define!(IDENTITY(value) = value);
        define!(EMPTY());
        comment!("public representation");
        type Word = uint32_t;
        type Node = struct Node {
            value: Word,
            payload: union {
                integer: int,
                address: *mut void,
            },
            ..{ trailing },
        };
        type Choice = union Choice {
            integer: int,
        };
        struct Wrapper {
            callback: fn(value: Word) -> Word,
        }
        union Scalar {
            integer: int,
        }
        fn observe(value: Word) -> void;
        #[static] fn identity(value: Word) -> Word {
            return value;
        }
        assert!(sizeof(0 as Word) == 4, "Word width");
    };

    assert_eq!(
        unit.render(),
        concat!(
            "#include <stdint.h>\n",
            "#define WORD_BITS 32\n",
            "#define IDENTITY(value) value\n",
            "#define EMPTY()\n",
            "/* public representation */\n",
            "typedef uint32_t Word;\n",
            "typedef struct Node {\n",
            "    Word value;\n",
            "    union {\n",
            "        int integer;\n",
            "        void *address;\n",
            "    } payload;\n",
            "    struct Node *next;\n",
            "} Node;\n",
            "typedef union Choice {\n",
            "    int integer;\n",
            "} Choice;\n",
            "struct Wrapper {\n",
            "    Word (*callback)(Word value);\n",
            "};\n",
            "union Scalar {\n",
            "    int integer;\n",
            "};\n",
            "void observe(Word value);\n",
            "static Word identity(Word value) {\n",
            "    return value;\n",
            "}\n",
            "_Static_assert(sizeof((Word)0) == 4, \"Word width\");\n",
        )
    );
}

#[test]
fn composes_conditional_items_invocations_and_nested_initializers() {
    let guard = "EXAMPLE_H";
    let arguments = [Expr::identifier("first"), Expr::identifier("second")];
    let invocation = super::c_invocation!({ "FIELDS" }(..{ arguments }));
    let initializers = super::c_initializers! {
        tag: 1,
        payload.member: value,
    };
    let value = Expr::compound_literal(super::c_type!(Value), initializers);
    let unit = super::c_items! {
        if !defined({ guard }) {
            define!({ guard });
            { invocation };
            fn read() -> Value {
                return { value };
            }
        }
    };

    assert_eq!(
        unit.render(),
        concat!(
            "#ifndef EXAMPLE_H\n",
            "#define EXAMPLE_H\n",
            "FIELDS(first, second)\n",
            "Value read(void) {\n",
            "    return (Value){ .tag = 1, .payload.member = value };\n",
            "}\n",
            "#endif\n",
        )
    );
}

#[test]
fn embeds_rust_expressions_once_inside_structured_expressions() {
    let evaluations = Cell::new(0);
    let dynamic = || {
        evaluations.set(evaluations.get() + 1);
        Expr::identifier("dynamic")
    };

    let expression = super::c_expr! {
        if count > 0 { { dynamic() } + 1 } else { 0 as size_t }
    };

    assert_eq!(evaluations.get(), 1);
    assert_eq!(
        expression.to_string(),
        "(count > 0) ? (dynamic + 1) : (size_t)0"
    );
}

#[test]
fn parses_rust_expression_syntax() {
    let argument = Expr::identifier("dynamic");
    let expression = super::c_expr! {
        count > 0 && observe(&value.member, { argument }) == expected
    };

    assert_eq!(
        expression.to_string(),
        "(count > 0) && (observe(&value.member, dynamic) == expected)"
    );
    assert_eq!(
        super::c_expr!(Pair {
            first: 1,
            second: value
        })
        .to_string(),
        "(Pair){ .first = 1, .second = value }"
    );
}

#[test]
fn parses_rust_type_syntax() {
    assert_eq!(super::c_type!(size_t).to_string(), "size_t");
    assert_eq!(
        super::c_type!(*const uint8_t).to_string(),
        "const uint8_t *"
    );
    assert_eq!(
        super::c_type!(*mut Struct<Node>).to_string(),
        "struct Node *"
    );
    assert_eq!(
        super::c_type!(*const *mut uint8_t).to_string(),
        "uint8_t *const *"
    );
    assert_eq!(
        super::c_type!(*mut *const uint8_t).to_string(),
        "const uint8_t **"
    );
}

#[test]
fn ordinary_identifiers_never_select_an_alternate_grammar() {
    assert_eq!(super::c_expr!(call(value)).to_string(), "call(value)");
    assert_eq!(
        super::c_expr!(named_value + pointer_count).to_string(),
        "named_value + pointer_count"
    );
}

#[test]
fn parses_rust_statement_syntax() {
    let dynamic = Expr::identifier("dynamic");
    let block = super::c_block! {
        let value: size_t = 0;
        observe(value, { dynamic });
        if value > 0 {
            return value;
        }
    };

    let function = crate::backend::c::syntax::FunctionDefinition::from_signature(
        FunctionSignature::new("size_t", "example", []),
        block,
    );
    assert_eq!(
        function.render(),
        concat!(
            "size_t example(void) {\n",
            "    size_t value = 0;\n",
            "    observe(value, dynamic);\n",
            "    if (value > 0) {\n",
            "        return value;\n",
            "    }\n",
            "}\n",
        )
    );
}

#[test]
fn parses_rust_function_syntax() {
    let function = super::c_function! {
        #[static]
        #[inline]
        fn identity(#[maybe_unused] call: *mut mal_call_t, value: uint32_t) -> uint32_t {
            observe(call);
            return value;
        }
    };

    assert_eq!(
        function.render(),
        concat!(
            "static inline uint32_t identity(",
            "mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, uint32_t value) {\n",
            "    observe(call);\n",
            "    return value;\n",
            "}\n",
        )
    );
}

#[test]
fn splices_runtime_call_arguments_in_order() {
    let middle = [Expr::identifier("second"), Expr::identifier("third")];
    let expression = super::c_expr!(observe(first, ..{ middle }, fourth));

    assert_eq!(
        expression.to_string(),
        "observe(first, second, third, fourth)"
    );
}

#[test]
fn constructs_compound_literals_from_static_and_rust_initializers() {
    let dynamic = Initializer::designated("second", Expr::number("2"));
    let trailing = [dynamic, Initializer::positional(Expr::number("3"))];
    let expression = super::c_expr!(Pair {
        first: 1,
        ..{ trailing }
    });

    assert_eq!(
        expression.to_string(),
        "(Pair){ .first = 1, .second = 2, 3 }"
    );
}

#[test]
fn builds_nested_blocks_and_splices_runtime_node_sequences_in_order() {
    let statements = [super::c_statement!(observe(value);)];
    let cases = super::c_switch_cases! {
        1 => { return 2; },
    };
    let body = super::c_block! {
        let value: int = 0;
        ..{ statements }
        if value == 0 {
            match value {
                ..{ cases },
                _ => { return 3; },
            }
        }
    };
    let function = crate::backend::c::syntax::FunctionDefinition::from_signature(
        FunctionSignature::new("int", "example", []),
        body,
    );

    assert_eq!(
        function.render(),
        concat!(
            "int example(void) {\n",
            "    int value = 0;\n",
            "    observe(value);\n",
            "    if (value == 0) {\n",
            "        switch (value) {\n",
            "            case 1: {\n",
            "                return 2;\n",
            "            }\n",
            "            default: {\n",
            "                return 3;\n",
            "            }\n",
            "        }\n",
            "    }\n",
            "}\n",
        )
    );
}

#[test]
fn function_macro_embeds_signature_grammar_directly() {
    let function = super::c_function! {
        #[static] #[inline] fn identity(
            value: uint32_t,
        ) -> uint32_t {
            return value;
        }
    };

    assert_eq!(
        function.render(),
        "static inline uint32_t identity(uint32_t value) {\n    return value;\n}\n"
    );
}
