use super::{Block, FunctionDefinition, Statement};
use crate::c_emit::syntax::{Expr, FunctionSignature};

#[test]
fn renders_function_statements_and_blocks() {
    let definition = FunctionDefinition::from_signature(
        FunctionSignature::new("int", "choose", []),
        Block::new([
            Statement::variable("int", "result", Some(Expr::number("0"))),
            Statement::if_then(
                Expr::identifier("ready"),
                Block::new([Statement::assignment(
                    Expr::identifier("result"),
                    Expr::number("42"),
                )]),
            ),
            Statement::return_value(Expr::identifier("result")),
        ]),
    );

    assert_eq!(
        definition.render(),
        "int choose(void) {\n    int result = 0;\n    if (ready) {\n        result = 42;\n    }\n    return result;\n}\n"
    );
}
