use crate::c_emit::syntax::{
    Block as CBlock, Expr as CExpr, ForInitializer, FunctionDefinition, FunctionSignature,
    Initializer, Parameter, Statement, TranslationUnit, TypeName,
};
use crate::check::ast::Type;

use super::BodyEmitter;

impl BodyEmitter<'_> {
    pub(super) fn emit_argument_main(&self, name: String, parameter: &Type) -> TranslationUnit {
        let parameter_type = self.types.c_type(parameter);
        let argv = || {
            CExpr::identifier("mal_argv").index(CExpr::add(
                CExpr::identifier("mal_index"),
                CExpr::number("1"),
            ))
        };
        let trap = |message: &'static str| {
            Statement::call(
                "mal_trap",
                [
                    CExpr::address_of(CExpr::identifier("mal_context")),
                    CExpr::string(message),
                ],
            )
        };
        TranslationUnit::new([FunctionDefinition::from_signature(
            FunctionSignature::new(
                "int",
                "main",
                [
                    Parameter::named("int", "mal_argc"),
                    Parameter::named(TypeName::named("char").pointer().pointer(), "mal_argv"),
                ],
            ),
            CBlock::new([
                Statement::variable(
                    "MalContext",
                    "mal_context",
                    Some(CExpr::initializer_list([Initializer::positional(
                        CExpr::identifier("NULL"),
                    )])),
                ),
                Statement::call(
                    "mal_program_initialize",
                    [CExpr::address_of(CExpr::identifier("mal_context"))],
                ),
                Statement::variable(
                    "size_t",
                    "mal_argument_count",
                    Some(CExpr::conditional(
                        CExpr::greater(CExpr::identifier("mal_argc"), CExpr::number("1")),
                        CExpr::cast(
                            "size_t",
                            CExpr::subtract(CExpr::identifier("mal_argc"), CExpr::number("1")),
                        ),
                        CExpr::number("0"),
                    )),
                ),
                Statement::variable(
                    TypeName::const_named("size_t"),
                    "mal_argument_stride",
                    Some(CExpr::add(
                        CExpr::sizeof_type("MalType_Ptr"),
                        CExpr::sizeof_type("uint64_t"),
                    )),
                ),
                Statement::if_then(
                    CExpr::greater(
                        CExpr::identifier("mal_argument_count"),
                        CExpr::divide(
                            CExpr::identifier("SIZE_MAX"),
                            CExpr::identifier("mal_argument_stride"),
                        ),
                    ),
                    CBlock::new([trap("argument descriptor size overflow")]),
                ),
                Statement::variable(
                    TypeName::named("uint8_t").pointer(),
                    "mal_argument_storage",
                    Some(CExpr::cast(
                        TypeName::named("uint8_t").pointer(),
                        CExpr::named_call(
                            "mal_allocate",
                            [
                                CExpr::address_of(CExpr::identifier("mal_context")),
                                CExpr::multiply(
                                    CExpr::identifier("mal_argument_count"),
                                    CExpr::identifier("mal_argument_stride"),
                                ),
                            ],
                        ),
                    )),
                ),
                Statement::for_loop(
                    ForInitializer::variable("size_t", "mal_index", CExpr::number("0")),
                    CExpr::less(
                        CExpr::identifier("mal_index"),
                        CExpr::identifier("mal_argument_count"),
                    ),
                    CExpr::pre_increment(CExpr::identifier("mal_index")),
                    CBlock::new([
                        Statement::variable(
                            "size_t",
                            "mal_length",
                            Some(CExpr::named_call("strlen", [argv()])),
                        ),
                        Statement::if_then(
                            CExpr::not_equal(
                                CExpr::cast("uint64_t", CExpr::identifier("mal_length")),
                                CExpr::identifier("mal_length"),
                            ),
                            CBlock::new([trap("argument length overflow")]),
                        ),
                        Statement::variable(
                            "MalType_Ptr",
                            "mal_data",
                            Some(CExpr::named_call(
                                "mal_Ptr_from_address",
                                [CExpr::cast(TypeName::named("uint8_t").pointer(), argv())],
                            )),
                        ),
                        Statement::variable(
                            "uint64_t",
                            "mal_length_u64",
                            Some(CExpr::cast("uint64_t", CExpr::identifier("mal_length"))),
                        ),
                        Statement::variable(
                            TypeName::named("uint8_t").pointer(),
                            "mal_slot",
                            Some(CExpr::add(
                                CExpr::identifier("mal_argument_storage"),
                                CExpr::multiply(
                                    CExpr::identifier("mal_index"),
                                    CExpr::identifier("mal_argument_stride"),
                                ),
                            )),
                        ),
                        Statement::call(
                            "memcpy",
                            [
                                CExpr::identifier("mal_slot"),
                                CExpr::address_of(CExpr::identifier("mal_data")),
                                CExpr::sizeof_expr(CExpr::identifier("mal_data")),
                            ],
                        ),
                        Statement::call(
                            "memcpy",
                            [
                                CExpr::add(
                                    CExpr::identifier("mal_slot"),
                                    CExpr::sizeof_expr(CExpr::identifier("mal_data")),
                                ),
                                CExpr::address_of(CExpr::identifier("mal_length_u64")),
                                CExpr::sizeof_expr(CExpr::identifier("mal_length_u64")),
                            ],
                        ),
                    ]),
                ),
                Statement::variable(
                    parameter_type.clone(),
                    "mal_arguments",
                    Some(CExpr::compound_literal(
                        parameter_type,
                        [
                            Initializer::designated(
                                "field_0",
                                CExpr::cast("uint64_t", CExpr::identifier("mal_argument_count")),
                            ),
                            Initializer::designated(
                                "field_1",
                                CExpr::named_call(
                                    "mal_Ptr_from_address",
                                    [CExpr::identifier("mal_argument_storage")],
                                ),
                            ),
                        ],
                    )),
                ),
                Statement::variable(
                    "int32_t",
                    "mal_result",
                    Some(CExpr::call(
                        CExpr::identifier(name.clone()).field("call"),
                        [
                            CExpr::address_of(CExpr::identifier("mal_context")),
                            CExpr::identifier(name).field("environment"),
                            CExpr::identifier("mal_arguments"),
                        ],
                    )),
                ),
                Statement::call(
                    "mal_context_destroy",
                    [CExpr::address_of(CExpr::identifier("mal_context"))],
                ),
                Statement::return_value(CExpr::cast("int", CExpr::identifier("mal_result"))),
            ]),
        )
        .into()])
    }
}
