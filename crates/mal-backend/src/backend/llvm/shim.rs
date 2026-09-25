use crate::backend::c::syntax::{
    Block, Expr, FunctionDefinition, FunctionSignature, Parameter, Statement, TypeName,
    VariableDeclaration,
};
use mal_frontend::check::ast::Type;

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
    let Type::Buffer(element) = parameter else {
        return None;
    };
    if **element != Type::Symbol {
        return None;
    }
    // A Symbol view is an owner, a data address, and a byte count, laid out like `(Address, Address, USize)`. The
    // runtime callbacks read the owner as the first field.
    let view = Type::Product(vec![Type::Address, Type::Address, Type::USize].into());
    let view_fields = types.product_fields(&view)?;
    if view_fields.first()?.offset != 0 {
        return None;
    }
    let data_offset = view_fields.get(1)?.offset;
    let length_offset = view_fields.get(2)?.offset;
    let stride = types.value(element)?.size;
    let value = types.value(parameter)?;

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
        variable(
            TypeName::named("void").pointer(),
            "arguments",
            Some(Expr::named_call(
                "mal_runtime_buffer_from_arguments",
                [
                    Expr::address_of(identifier("context")),
                    Expr::add(identifier("mal_argv"), number(1)),
                    identifier("argument_count"),
                    number(stride),
                    number(data_offset),
                    number(length_offset),
                ],
            )),
        ),
        call(
            "memcpy",
            [
                identifier("argument"),
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
        // The entry only borrows its argument, so the shim drops the buffer and the Symbols it owns.
        call("mal_runtime_environment_release", [identifier("arguments")]),
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
