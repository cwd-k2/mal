use crate::c_emit::syntax::{
    Block, Expr, FunctionDefinition, FunctionSignature, Parameter, Statement, SwitchCase,
    TranslationUnit,
};
use crate::check::ast::Type;

use super::{TypeRegistry, is_bool};

impl TypeRegistry {
    pub(in crate::c_emit) fn contains_managed(&self, ty: &Type) -> bool {
        crate::execution::ownership::is_managed(ty)
    }

    pub(in crate::c_emit) fn copy_value(&self, ty: &Type, value: Expr) -> Expr {
        if !self.contains_managed(ty) {
            return value;
        }
        match ty {
            Type::Symbol => Expr::named_call(
                "mal_symbol_retain",
                [Expr::identifier("mal_context"), value],
            ),
            Type::Product(_) | Type::Sum(_) | Type::Function { .. } => Expr::named_call(
                copy_name(self.index(ty)),
                [Expr::identifier("mal_context"), value],
            ),
            _ => unreachable!("only managed types require copy operations"),
        }
    }

    pub(in crate::c_emit) fn destroy_value(&self, block: &mut Block, ty: &Type, value: Expr) {
        if !self.contains_managed(ty) {
            return;
        }
        match ty {
            Type::Symbol => block.push(Statement::call("mal_symbol_release", [value])),
            Type::Product(_) | Type::Sum(_) | Type::Function { .. } => block.push(Statement::call(
                destroy_name(self.index(ty)),
                [Expr::identifier("mal_context"), value],
            )),
            _ => unreachable!("only managed types require destroy operations"),
        }
    }

    pub(in crate::c_emit) fn lifetime_definitions(&self) -> TranslationUnit {
        let mut output = TranslationUnit::default();
        for (index, ty) in self.aggregates.iter().enumerate() {
            if !self.contains_managed(ty) {
                continue;
            }
            output.push(self.copy_definition(index, ty));
            output.blank_line();
            output.push(self.destroy_definition(index, ty));
            output.blank_line();
        }
        output
    }

    fn copy_definition(&self, index: usize, ty: &Type) -> FunctionDefinition {
        let mut body = Block::default();
        body.push(Statement::expression(Expr::cast(
            "void",
            Expr::identifier("mal_context"),
        )));
        match ty {
            Type::Product(elements) => {
                for (field, element) in elements.iter().enumerate() {
                    if self.contains_managed(element) {
                        let target = Expr::identifier("value").field(format!("field_{field}"));
                        body.push(Statement::assignment(
                            target.clone(),
                            self.copy_value(element, target),
                        ));
                    }
                }
            }
            Type::Sum(members) if !is_bool(ty) => {
                let mut cases = Vec::new();
                for (variant, member) in members.iter().enumerate() {
                    let mut case = Block::default();
                    if self.contains_managed(member) {
                        let target = Expr::identifier("value")
                            .field("payload")
                            .field(format!("variant_{variant}"));
                        case.push(Statement::assignment(
                            target.clone(),
                            self.copy_value(member, target),
                        ));
                    }
                    case.push(Statement::Break);
                    cases.push(SwitchCase::case(uint32(variant), case));
                }
                cases.push(invalid_sum_default());
                body.push(Statement::switch(
                    Expr::identifier("value").field("tag"),
                    cases,
                ));
            }
            Type::Function { .. } => {
                body.push(Statement::assignment(
                    Expr::identifier("value").field("environment"),
                    Expr::named_call(
                        "mal_retain",
                        [
                            Expr::identifier("mal_context"),
                            Expr::identifier("value").field("environment"),
                        ],
                    ),
                ));
            }
            Type::Sum(_) => unreachable!("Bool does not have managed fields"),
            _ => unreachable!("only aggregate types have generated lifetime operations"),
        }
        body.push(Statement::return_value(Expr::identifier("value")));
        FunctionDefinition::from_signature(
            FunctionSignature::static_inline(
                self.c_type(ty),
                copy_name(index),
                [
                    Parameter::named(
                        crate::c_emit::syntax::TypeName::named("MalContext").pointer(),
                        "mal_context",
                    ),
                    Parameter::named(self.c_type(ty), "value"),
                ],
            )
            .maybe_unused(),
            body,
        )
    }

    fn destroy_definition(&self, index: usize, ty: &Type) -> FunctionDefinition {
        let mut body = Block::default();
        body.push(Statement::expression(Expr::cast(
            "void",
            Expr::identifier("mal_context"),
        )));
        match ty {
            Type::Product(elements) => {
                for (field, element) in elements.iter().enumerate().rev() {
                    self.destroy_value(
                        &mut body,
                        element,
                        Expr::identifier("value").field(format!("field_{field}")),
                    );
                }
            }
            Type::Sum(members) if !is_bool(ty) => {
                let mut cases = Vec::new();
                for (variant, member) in members.iter().enumerate() {
                    let mut case = Block::default();
                    self.destroy_value(
                        &mut case,
                        member,
                        Expr::identifier("value")
                            .field("payload")
                            .field(format!("variant_{variant}")),
                    );
                    case.push(Statement::Break);
                    cases.push(SwitchCase::case(uint32(variant), case));
                }
                cases.push(invalid_sum_default());
                body.push(Statement::switch(
                    Expr::identifier("value").field("tag"),
                    cases,
                ));
            }
            Type::Function { .. } => {
                let environment = Expr::identifier("value").field("environment");
                body.push(Statement::if_then(
                    Expr::not_equal(environment.clone(), Expr::identifier("NULL")),
                    Block::new([Statement::if_then(
                        Expr::not_equal(
                            Expr::named_call("mal_release", [environment.clone()]),
                            uint8(0),
                        ),
                        Block::new([Statement::expression(Expr::call(
                            Expr::identifier("value").field("destroy_environment"),
                            [Expr::identifier("mal_context"), environment],
                        ))]),
                    )]),
                ));
            }
            Type::Sum(_) => unreachable!("Bool does not have managed fields"),
            _ => unreachable!("only aggregate types have generated lifetime operations"),
        }
        FunctionDefinition::from_signature(
            FunctionSignature::static_inline(
                "void",
                destroy_name(index),
                [
                    Parameter::named(
                        crate::c_emit::syntax::TypeName::named("MalContext").pointer(),
                        "mal_context",
                    ),
                    Parameter::named(self.c_type(ty), "value"),
                ],
            )
            .maybe_unused(),
            body,
        )
    }
}

fn copy_name(index: usize) -> String {
    format!("mal_copy_value_{index}")
}

fn destroy_name(index: usize) -> String {
    format!("mal_destroy_value_{index}")
}

fn uint8(value: u8) -> Expr {
    Expr::named_call("UINT8_C", [Expr::number(value.to_string())])
}

fn uint32(value: usize) -> Expr {
    Expr::named_call("UINT32_C", [Expr::number(value.to_string())])
}

fn invalid_sum_default() -> SwitchCase {
    SwitchCase::default(Block::new([Statement::call(
        "mal_trap",
        [
            Expr::identifier("mal_context"),
            Expr::string("invalid sum tag"),
        ],
    )]))
}
