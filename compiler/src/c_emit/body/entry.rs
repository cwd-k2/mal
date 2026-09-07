use crate::c_emit::syntax::{
    Block as CBlock, Expr as CExpr, FunctionDefinition, FunctionSignature, Initializer, Parameter,
    Statement, TranslationUnit, TypeName,
};
use crate::check::ast::Type;
use crate::closure::ast::{self as closure, TopLevelPattern};

use super::{BodyEmitter, value_name};

#[path = "entry/arguments.rs"]
mod arguments;

impl BodyEmitter<'_> {
    pub(super) fn emit_initializer(&mut self) -> TranslationUnit {
        let mut body = CBlock::default();
        body.push(Statement::expression(CExpr::cast(
            "void",
            CExpr::identifier("mal_context"),
        )));
        for binding in &self.program.bindings {
            self.emit_block_bindings(&mut body, &binding.value);
            match &binding.pattern {
                TopLevelPattern::Binding { id, .. } => {
                    body.push(Statement::assignment(
                        CExpr::identifier(value_name(*id)),
                        self.types.copy_value(
                            &binding.value.result.ty,
                            self.emit_atom(&binding.value.result),
                        ),
                    ));
                    body.push(Statement::expression(CExpr::cast(
                        "void",
                        CExpr::identifier(value_name(*id)),
                    )));
                }
                TopLevelPattern::Wildcard { .. } => {
                    body.push(Statement::expression(CExpr::cast(
                        "void",
                        self.emit_atom(&binding.value.result),
                    )));
                }
                TopLevelPattern::Product { .. } => self.emit_top_level_pattern(
                    &mut body,
                    &binding.pattern,
                    self.emit_atom(&binding.value.result),
                ),
            }
            self.emit_block_cleanup(&mut body, &binding.value);
        }
        let definition = FunctionDefinition::from_signature(
            FunctionSignature::static_function(
                "void",
                "mal_program_initialize",
                [Parameter::named(
                    TypeName::named("MalContext").pointer(),
                    "mal_context",
                )],
            ),
            body,
        );
        let mut output = TranslationUnit::new([definition.into()]);
        output.blank_line();
        output
    }

    pub(super) fn emit_program_destroy(&self) -> TranslationUnit {
        let mut body = CBlock::default();
        for binding in self.program.bindings.iter().rev() {
            self.destroy_top_level_pattern(&mut body, &binding.pattern);
        }
        body.push(Statement::expression(CExpr::cast(
            "void",
            CExpr::identifier("mal_context"),
        )));
        let definition = FunctionDefinition::from_signature(
            FunctionSignature::static_function(
                "void",
                "mal_program_destroy",
                [Parameter::named(
                    TypeName::named("MalContext").pointer(),
                    "mal_context",
                )],
            ),
            body,
        );
        let mut output = TranslationUnit::new([definition.into()]);
        output.blank_line();
        output
    }

    pub(super) fn emit_main(&self, main: &closure::TopLevelBinding) -> TranslationUnit {
        let TopLevelPattern::Binding { id, ref ty, .. } = main.pattern else {
            unreachable!()
        };
        let name = value_name(id);
        let Type::Function { parameter, .. } = ty else {
            unreachable!("entry point validation requires a function")
        };
        if **parameter == Type::Unit {
            return TranslationUnit::new([FunctionDefinition::from_signature(
                FunctionSignature::new("int", "main", []),
                CBlock::new([
                    Statement::variable(
                        "MalContext",
                        "mal_context",
                        Some(CExpr::initializer_list([Initializer::positional(
                            CExpr::number("0"),
                        )])),
                    ),
                    Statement::variable(
                        "MalType_Unit",
                        "mal_unit",
                        Some(CExpr::initializer_list([Initializer::positional(
                            CExpr::named_call("UINT8_C", [CExpr::number("0")]),
                        )])),
                    ),
                    Statement::call(
                        "mal_program_initialize",
                        [CExpr::address_of(CExpr::identifier("mal_context"))],
                    ),
                    Statement::variable(
                        "int32_t",
                        "mal_result",
                        Some(CExpr::call(
                            CExpr::identifier(name.clone()).field("call"),
                            [
                                CExpr::address_of(CExpr::identifier("mal_context")),
                                CExpr::identifier(name.clone()).field("environment"),
                                CExpr::identifier("mal_unit"),
                            ],
                        )),
                    ),
                    Statement::call(
                        "mal_program_destroy",
                        [CExpr::address_of(CExpr::identifier("mal_context"))],
                    ),
                    Statement::call(
                        "mal_context_destroy",
                        [CExpr::address_of(CExpr::identifier("mal_context"))],
                    ),
                    Statement::return_value(CExpr::cast("int", CExpr::identifier("mal_result"))),
                ]),
            )
            .into()]);
        }

        self.emit_argument_main(name, parameter)
    }
}
