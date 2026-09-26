use crate::backend::c::syntax::{
    Expr, FunctionDefinition, FunctionSignature, Parameter, TypeName, VariableDeclaration, c_block,
    c_expr,
};
use mal_frontend::check::ast::Type;

use super::body::types::Types;

pub(super) struct Output {
    pub(super) definition: FunctionDefinition,
    pub(super) uses_byte_runtime: bool,
}

pub(super) fn entry_main(parameter: &Type, types: Types, entry: &str) -> Option<Output> {
    if *parameter == Type::Unit {
        return Some(Output {
            definition: unit_main(entry),
            uses_byte_runtime: false,
        });
    }
    Some(Output {
        definition: argument_main(parameter, types, entry)?,
        uses_byte_runtime: true,
    })
}

fn unit_main(entry: &str) -> FunctionDefinition {
    FunctionDefinition::from_signature(
        FunctionSignature::new("int", "main", []),
        c_block!(
            (var ("MalContext") ("context") = (rust zero_initializer())),
            (var ("int32_t") ("result")),
            (call entry;
                (address (id "context")),
                (id "NULL"),
                (address (id "result")),
            ),
            (call "mal_control_destroy"; (address (id "context"))),
            (return (id "result")),
        ),
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

    let body = c_block!(
        (var ("MalContext") ("context") = (rust zero_initializer())),
        (var ("size_t") ("argument_count") = (conditional
                (greater (id "mal_argc"); (number 1));
                (cast "size_t"; (subtract (id "mal_argc"); (number 1)));
                (number 0)
        )),
        (declaration (
            VariableDeclaration::array("uint8_t", "argument", number(value.size))
                .aligned(number(value.alignment))
        ) = (rust zero_initializer())),
        (var (TypeName::named("void").pointer()) ("arguments") = (call "mal_runtime_buffer_from_arguments";
                (address (id "context")),
                (add (id "mal_argv"); (number 1)),
                (id "argument_count"),
                (number stride),
                (number data_offset),
                (number length_offset),
        )),
        (call "memcpy";
            (id "argument"),
            (address (id "arguments")),
            (sizeof (id "arguments")),
        ),
        (var ("int32_t") ("result")),
        (call entry;
            (address (id "context")),
            (id "argument"),
            (address (id "result")),
        ),
        // The entry only borrows its argument, so the shim drops the buffer and the Symbols it owns.
        (call "mal_runtime_environment_release"; (id "arguments")),
        (call "mal_control_destroy"; (address (id "context"))),
        (return (id "result")),
    );

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

fn number(value: impl ToString) -> Expr {
    Expr::number(value.to_string())
}

fn zero_initializer() -> Expr {
    c_expr!(initializer (number 0))
}
