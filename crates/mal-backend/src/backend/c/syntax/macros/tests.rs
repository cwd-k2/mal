use std::cell::Cell;

use crate::backend::c::syntax::{Expr, FunctionSignature, Initializer, SwitchCase};

#[test]
fn embeds_rust_expressions_once_inside_structured_expressions() {
    let evaluations = Cell::new(0);
    let dynamic = || {
        evaluations.set(evaluations.get() + 1);
        Expr::identifier("dynamic")
    };

    let expression = super::c_expr!(conditional
        (greater (id "count"); (number 0));
        (add (rust dynamic()); (number 1));
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
        (extend middle),
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
        (rust dynamic),
        (extend trailing),
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
        (extend statements),
        (if (equal (id "value"); (number 0)); [
            (switch (id "value"); [
                (extend cases),
                (default; [(return (number 3))]),
            ]),
        ]),
    );
    let function = super::c_function!(signature
        FunctionSignature::new("int", "example", []);
        body body
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
