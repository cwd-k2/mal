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
    let descriptor_type = Type::Product(vec![Type::Address, Type::ByteSize].into());
    let descriptor = types.value(&descriptor_type)?;
    let descriptor_fields = types.product_fields(&descriptor_type)?;
    let descriptor_address_offset = descriptor_fields.first()?.offset;
    let descriptor_length_offset = descriptor_fields.get(1)?.offset;
    let descriptor_stride = descriptor.size;

    let count = identifier("argument_count");
    let index = identifier("index");
    let argv_slot = identifier("mal_argv").subscript(Expr::add(index.clone(), number(1)));
    let descriptor_slot = Expr::add(
        identifier("storage"),
        Expr::multiply(index.clone(), number(descriptor_stride)),
    );
    let descriptor_overflows = Expr::greater(
        count.clone(),
        Expr::divide(identifier("SIZE_MAX"), number(descriptor_stride)),
    );

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
        Statement::if_then(
            descriptor_overflows,
            Block::new([trap("argument descriptor size overflow")]),
        ),
        variable(
            "size_t",
            "storage_size",
            Some(Expr::conditional(
                Expr::equal(count.clone(), number(0)),
                number(1),
                Expr::multiply(count.clone(), number(descriptor_stride)),
            )),
        ),
        variable(
            TypeName::named("uint8_t").pointer(),
            "storage",
            Some(Expr::named_call(
                "mal_runtime_allocate",
                [
                    Expr::address_of(identifier("context")),
                    identifier("storage_size"),
                ],
            )),
        ),
        Statement::for_loop(
            VariableDeclaration::new("size_t", "index"),
            number(0),
            Expr::less(index.clone(), count.clone()),
            Expr::pre_increment(index),
            Block::new([
                variable(
                    "size_t",
                    "length",
                    Some(Expr::named_call("strlen", [argv_slot.clone()])),
                ),
                variable(
                    TypeName::named("uint8_t").pointer(),
                    "slot",
                    Some(descriptor_slot),
                ),
                variable(TypeName::named("void").pointer(), "data", Some(argv_slot)),
                call(
                    "memcpy",
                    [
                        Expr::add(identifier("slot"), number(descriptor_address_offset)),
                        Expr::address_of(identifier("data")),
                        Expr::sizeof_value(identifier("data")),
                    ],
                ),
                call(
                    "memcpy",
                    [
                        Expr::add(identifier("slot"), number(descriptor_length_offset)),
                        Expr::address_of(identifier("length")),
                        Expr::sizeof_value(identifier("length")),
                    ],
                ),
            ]),
        ),
        Statement::variable_declaration(
            VariableDeclaration::array("uint8_t", "argument", number(value.size))
                .aligned(number(value.alignment)),
            Some(zero_initializer()),
        ),
        variable("size_t", "count_usize", Some(count)),
        variable(
            TypeName::named("void").pointer(),
            "descriptor",
            Some(identifier("storage")),
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
                Expr::address_of(identifier("descriptor")),
                Expr::sizeof_value(identifier("descriptor")),
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
        call("mal_runtime_deallocate", [identifier("storage")]),
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

fn trap(message: &str) -> Statement {
    call(
        "mal_trap",
        [
            Expr::address_of(identifier("context")),
            Expr::string(message),
        ],
    )
}
