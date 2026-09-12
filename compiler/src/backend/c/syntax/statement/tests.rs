use super::{Block, FunctionDefinition, Statement};
use crate::backend::c::syntax::{Expr, FunctionSignature, VariableDeclaration};

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

#[test]
fn renders_aligned_arrays_and_for_loops() {
    let definition = FunctionDefinition::from_signature(
        FunctionSignature::new("void", "fill", []),
        Block::new([
            Statement::variable_declaration(
                VariableDeclaration::array("uint8_t", "storage", Expr::number("32"))
                    .aligned(Expr::number("8")),
                Some(Expr::initializer_list([Expr::number("0")])),
            ),
            Statement::for_loop(
                VariableDeclaration::new("size_t", "index"),
                Expr::number("0"),
                Expr::greater(Expr::identifier("count"), Expr::identifier("index")),
                Expr::pre_increment(Expr::identifier("index")),
                Block::new([Statement::expression(Expr::assign(
                    Expr::identifier("storage").subscript(Expr::identifier("index")),
                    Expr::number("1"),
                ))]),
            ),
        ]),
    );

    assert_eq!(
        definition.render(),
        "void fill(void) {\n    _Alignas(8) uint8_t storage[32] = { 0 };\n    for (size_t index = 0; count > index; ++index) {\n        storage[index] = 1;\n    }\n}\n"
    );
}
