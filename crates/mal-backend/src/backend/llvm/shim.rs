use crate::backend::c::syntax::{
    Expr, FunctionDefinition, Statement, VariableDeclaration, c_block, c_expr, c_function,
    c_signature,
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
    c_function! {
        fn main() -> int {
            let context: MalContext = { zero_initializer() };
            let result: int32_t;
            { entry }(&context, NULL, &result);
            mal_control_destroy(&context);
            return result;
        }
    }
}

fn argument_main(parameter: &Type, types: Types, entry: &str) -> Option<FunctionDefinition> {
    let Type::Buffer(element) = parameter else {
        return None;
    };
    if **element != Type::Symbol {
        return None;
    }
    // A Symbol view stores its owner, data pointer, and byte count. Runtime callbacks read the owner first.
    let view_fields = types.symbol_fields()?;
    if view_fields.first()?.offset != 0 {
        return None;
    }
    let data_offset = view_fields.get(1)?.offset;
    let length_offset = view_fields.get(2)?.offset;
    let stride = types.value(element)?.size;
    let value = types.value(parameter)?;

    let argument_declaration = Statement::variable_declaration(
        VariableDeclaration::array("uint8_t", "argument", number(value.size))
            .aligned(number(value.alignment)),
        Some(zero_initializer()),
    );
    let body = c_block! {
        let context: MalContext = { zero_initializer() };
        let argument_count: size_t = if mal_argc > 1 { (mal_argc - 1) as size_t } else { 0 };
        { argument_declaration };
        let arguments: *mut void = mal_runtime_buffer_from_arguments(
            &context,
            mal_argv + 1,
            argument_count,
            { number(stride) },
            { number(data_offset) },
            { number(length_offset) },
        );
        memcpy(argument, &arguments, sizeof(arguments));
        let result: int32_t;
        { entry }(&context, argument, &result);
        // The entry only borrows its argument, so the shim drops the buffer and the Symbols it owns.
        mal_runtime_owner_release(arguments);
        mal_control_destroy(&context);
        return result;
    };

    Some(FunctionDefinition::from_signature(
        c_signature! {
            fn main(
                mal_argc: int,
                mal_argv: *mut *mut char,
            ) -> int
        },
        body,
    ))
}

fn number(value: impl ToString) -> Expr {
    Expr::number(value.to_string())
}

fn zero_initializer() -> Expr {
    c_expr!([0])
}
