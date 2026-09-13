use crate::backend::c::syntax::{
    Block, Expr, FunctionDefinition, FunctionSignature, Initializer, Parameter, Statement,
    TranslationUnit, TypeName, VariableDeclaration,
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

pub(super) fn symbol_bridge_runtime() -> String {
    let context = Parameter::named(TypeName::named("MalContext").pointer(), "context");
    let value = Parameter::named("MalType_Symbol", "value");
    let mut unit = TranslationUnit::default();
    unit.push(FunctionDefinition::from_signature(
        FunctionSignature::new(
            "MalType_Symbol",
            "mal_symbol_materialize",
            [context.clone(), value.clone()],
        ),
        Block::new([
            Statement::expression(Expr::assign(
                identifier("value").field("data"),
                Expr::named_call(
                    "mal_runtime_symbol_data",
                    [
                        identifier("context"),
                        identifier("value").field("ownership"),
                    ],
                ),
            )),
            Statement::return_value(identifier("value")),
        ]),
    ));
    unit.blank_line();
    unit.push(FunctionDefinition::from_signature(
        FunctionSignature::new(
            "MalType_Symbol",
            "mal_symbol_copy_from_bytes",
            [
                context.clone(),
                Parameter::named(TypeName::const_named("uint8_t").pointer(), "data"),
                Parameter::named("uint64_t", "length"),
            ],
        ),
        Block::new([
            variable(
                TypeName::named("void").pointer(),
                "ownership",
                Some(Expr::named_call(
                    "mal_runtime_symbol_read",
                    [
                        identifier("context"),
                        identifier("data"),
                        identifier("length"),
                    ],
                )),
            ),
            Statement::return_value(Expr::compound_literal(
                "MalType_Symbol",
                [
                    Initializer::designated(
                        "data",
                        Expr::named_call(
                            "mal_runtime_symbol_data",
                            [identifier("context"), identifier("ownership")],
                        ),
                    ),
                    Initializer::designated("length", identifier("length")),
                    Initializer::designated("ownership", identifier("ownership")),
                ],
            )),
        ]),
    ));
    unit.blank_line();
    unit.push(FunctionDefinition::from_signature(
        FunctionSignature::new("MalType_Symbol", "mal_symbol_retain", [context, value]),
        Block::new([
            Statement::expression(Expr::assign(
                identifier("value").field("ownership"),
                Expr::named_call(
                    "mal_runtime_symbol_retain",
                    [
                        identifier("context"),
                        identifier("value").field("ownership"),
                    ],
                ),
            )),
            Statement::return_value(identifier("value")),
        ]),
    ));
    unit.render()
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
    let descriptor_stride = types
        .value(&Type::Ptr)?
        .size
        .checked_add(types.value(&Type::UInt64)?.size)?;

    let count = identifier("argument_count");
    let index = identifier("index");
    let argv_slot = identifier("mal_argv").subscript(Expr::add(index.clone(), number(1)));
    let descriptor_slot = Expr::add(
        identifier("storage"),
        Expr::multiply(index.clone(), number(descriptor_stride)),
    );
    let count_does_not_fit = Expr::not_equal(
        Expr::cast("size_t", Expr::cast("uint64_t", count.clone())),
        count.clone(),
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
            Expr::logical_or(count_does_not_fit, descriptor_overflows),
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
                Statement::if_then(
                    Expr::not_equal(
                        Expr::cast("size_t", Expr::cast("uint64_t", identifier("length"))),
                        identifier("length"),
                    ),
                    Block::new([trap("argument length overflow")]),
                ),
                variable(
                    TypeName::named("uint8_t").pointer(),
                    "slot",
                    Some(descriptor_slot),
                ),
                variable(TypeName::named("void").pointer(), "data", Some(argv_slot)),
                variable(
                    "uint64_t",
                    "length_u64",
                    Some(Expr::cast("uint64_t", identifier("length"))),
                ),
                call(
                    "memcpy",
                    [
                        identifier("slot"),
                        Expr::address_of(identifier("data")),
                        Expr::sizeof_value(identifier("data")),
                    ],
                ),
                call(
                    "memcpy",
                    [
                        Expr::add(identifier("slot"), Expr::sizeof_value(identifier("data"))),
                        Expr::address_of(identifier("length_u64")),
                        Expr::sizeof_value(identifier("length_u64")),
                    ],
                ),
            ]),
        ),
        Statement::variable_declaration(
            VariableDeclaration::array("uint8_t", "argument", number(value.size))
                .aligned(number(value.alignment)),
            Some(zero_initializer()),
        ),
        variable("uint64_t", "count_u64", Some(Expr::cast("uint64_t", count))),
        variable(
            TypeName::named("void").pointer(),
            "descriptor",
            Some(identifier("storage")),
        ),
        call(
            "memcpy",
            [
                Expr::add(identifier("argument"), number(count_offset)),
                Expr::address_of(identifier("count_u64")),
                Expr::sizeof_value(identifier("count_u64")),
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
