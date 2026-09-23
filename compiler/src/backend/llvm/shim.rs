use crate::backend::c::syntax::{
    Block, Expr, FunctionDefinition, FunctionSignature, Parameter, Statement, TypeName,
    VariableDeclaration,
};
use crate::check::ast::Type;

use super::body::types::Types;

pub(super) fn entry_main(
    parameter: &Type,
    types: Types,
    entry: &str,
) -> Option<FunctionDefinition> {
    if *parameter == Type::Unit {
        return Some(unit_main(entry));
    }
    argument_main(parameter, types, entry)
}

fn unit_main(entry: &str) -> FunctionDefinition {
    FunctionDefinition::from_signature(
        FunctionSignature::new("int", "main", []),
        Block::new([
            variable("MalContext", "context", Some(zero_initializer())),
            variable("int32_t", "result", None),
            call(
                entry,
                [
                    Expr::address_of(identifier("context")),
                    identifier("NULL"),
                    Expr::address_of(identifier("result")),
                ],
            ),
            call(
                "mal_control_destroy",
                [Expr::address_of(identifier("context"))],
            ),
            Statement::return_value(identifier("result")),
        ]),
    )
}

fn argument_main(parameter: &Type, types: Types, entry: &str) -> Option<FunctionDefinition> {
    let fields = types.product_fields(parameter)?;
    let count_offset = fields.first()?.offset;
    let pointer_offset = fields.get(1)?.offset;
    let value = types.value(parameter)?;
    let count = identifier("argument_count");

    let body = Block::new([
        variable("MalContext", "context", Some(zero_initializer())),
        variable(
            "size_t",
            "argument_count",
            Some(Expr::conditional(
                Expr::greater(identifier("mal_argc"), number(1)),
                Expr::cast("size_t", Expr::subtract(identifier("mal_argc"), number(1))),
                number(0),
            )),
        ),
        Statement::variable_declaration(
            VariableDeclaration::array("uint8_t", "argument", number(value.size))
                .aligned(number(value.alignment)),
            Some(zero_initializer()),
        ),
        variable("size_t", "count_usize", Some(count)),
        variable(
            TypeName::named("void").pointer(),
            "arguments",
            Some(Expr::add(identifier("mal_argv"), number(1))),
        ),
        call(
            "memcpy",
            [
                Expr::add(identifier("argument"), number(count_offset)),
                Expr::address_of(identifier("count_usize")),
                Expr::sizeof_value(identifier("count_usize")),
            ],
        ),
        call(
            "memcpy",
            [
                Expr::add(identifier("argument"), number(pointer_offset)),
                Expr::address_of(identifier("arguments")),
                Expr::sizeof_value(identifier("arguments")),
            ],
        ),
        variable("int32_t", "result", None),
        call(
            entry,
            [
                Expr::address_of(identifier("context")),
                identifier("argument"),
                Expr::address_of(identifier("result")),
            ],
        ),
        call(
            "mal_control_destroy",
            [Expr::address_of(identifier("context"))],
        ),
        Statement::return_value(identifier("result")),
    ]);

    Some(FunctionDefinition::from_signature(
        FunctionSignature::new(
            "int",
            "main",
            [
                Parameter::named("int", "mal_argc"),
                Parameter::named(TypeName::named("char").pointer().pointer(), "mal_argv"),
            ],
        ),
        body,
    ))
}

fn identifier(name: impl Into<crate::backend::c::syntax::Identifier>) -> Expr {
    Expr::identifier(name)
}

fn number(value: impl ToString) -> Expr {
    Expr::number(value.to_string())
}

fn variable(
    ty: impl Into<TypeName>,
    name: impl Into<crate::backend::c::syntax::Identifier>,
    initializer: Option<Expr>,
) -> Statement {
    Statement::variable(ty, name, initializer)
}

fn call(
    name: impl Into<crate::backend::c::syntax::Identifier>,
    arguments: impl IntoIterator<Item = Expr>,
) -> Statement {
    Statement::call(name, arguments)
}

fn zero_initializer() -> Expr {
    Expr::initializer_list([number(0)])
}
