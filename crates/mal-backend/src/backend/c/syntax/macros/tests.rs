use std::cell::Cell;

use crate::backend::c::syntax::{
    Expr, FunctionSignature, Initializer, SwitchCase, TranslationUnit,
};

#[test]
fn composes_types_parameters_and_signatures_with_template_interpolation() {
    let name = "convert";
    let result = super::c_type!(named("uint32_t"));
    let trailing = [super::c_parameter!("value" : named("uint8_t"))];
    let signature = super::c_signature!(static inline fn { name }(
        "call": ptr(named("mal_call_t")) [maybe_unused],
        {{ trailing }},
    ) -> { result });

    assert_eq!(
        signature.render(),
        "static inline uint32_t convert(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, uint8_t value)"
    );
}

#[test]
fn composes_declarations_and_nested_aggregate_fields() {
    let trailing = [super::c_aggregate_field!("next" : ptr(struct("Node")))];
    let mut unit = TranslationUnit::default();
    unit.push(super::c_declaration!(type "Word" = named("uint32_t")));
    unit.push(super::c_aggregate!(typedef struct "Node" => "Node"; [
        ("value": named("Word")),
        (union "payload"; [
            ("integer": named("int")),
            ("address": ptr(named("void"))),
        ]),
        {{ trailing }},
    ]));

    assert_eq!(
        unit.render(),
        concat!(
            "typedef uint32_t Word;\n",
            "typedef struct Node {\n",
            "    Word value;\n",
            "    union {\n",
            "        int integer;\n",
            "        void *address;\n",
            "    } payload;\n",
            "    struct Node *next;\n",
            "} Node;\n",
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

    let expression = super::c_expr!(conditional
        (greater (id "count"); (number 0));
        (add { dynamic() }; (number 1));
        (cast "size_t"; (number 0))
    );

    assert_eq!(evaluations.get(), 1);
    assert_eq!(
        expression.to_string(),
        "(count > 0) ? (dynamic + 1) : (size_t)0"
    );
}

#[test]
fn splices_runtime_call_arguments_in_order() {
    let middle = [Expr::identifier("second"), Expr::identifier("third")];
    let expression = super::c_expr!(call "observe";
        (id "first"),
        {{ middle }},
        (id "fourth"),
    );

    assert_eq!(
        expression.to_string(),
        "observe(first, second, third, fourth)"
    );
}

#[test]
fn constructs_compound_literals_from_static_and_rust_initializers() {
    let dynamic = Initializer::designated("second", Expr::number("2"));
    let trailing = [Initializer::positional(Expr::number("3"))];
    let expression = super::c_expr!(compound "Pair";
        (field "first"; (number 1)),
        { dynamic },
        {{ trailing }},
    );

    assert_eq!(
        expression.to_string(),
        "(Pair){ .first = 1, .second = 2, 3 }"
    );
}

#[test]
fn builds_nested_blocks_and_splices_runtime_node_sequences_in_order() {
    let statements = [super::c_statement!(call "observe"; (id "value"))];
    let cases = [SwitchCase::case(
        Expr::number("1"),
        super::c_block!((return (number 2))),
    )];
    let body = super::c_block!(
        (var ("int") ("value") = (number 0)),
        {{ statements }},
        (if (equal (id "value"); (number 0)); [
            (switch (id "value"); [
                {{ cases }},
                (default; [(return (number 3))]),
            ]),
        ]),
    );
    let function = super::c_function!(signature {
        FunctionSignature::new("int", "example", [])
    };
        body { body }
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
