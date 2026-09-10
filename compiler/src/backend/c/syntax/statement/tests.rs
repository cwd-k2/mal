use super::{Block, FunctionDefinition, Statement};
use crate::backend::c::syntax::{Expr, FunctionSignature};

#[test]
fn renders_function_statements_and_blocks() {
    let definition = FunctionDefinition::from_signature(
        FunctionSignature::new("int", "choose", []),
        Block::new([
            Statement::if_then(
                Expr::identifier("ready"),
                Block::new([Statement::return_value(Expr::number("42"))]),
            ),
            Statement::return_value(Expr::number("0")),
        ]),
    );

    assert_eq!(
        definition.render(),
        "int choose(void) {\n    if (ready) {\n        return 42;\n    }\n    return 0;\n}\n"
    );
}
