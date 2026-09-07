use crate::c_emit::syntax::{
    Block, Expr, FunctionDefinition, FunctionSignature, Initializer, Parameter, Statement,
    SwitchCase, TranslationUnit, TypeName,
};
use crate::check::ast::Type;
use crate::core::ast::TypeAlias;

use super::super::{HostTypes, TypeRegistry};

impl TypeRegistry {
    pub(in crate::c_emit) fn header_lifetime_helpers(
        &self,
        host: &HostTypes,
        aliases: &[TypeAlias],
    ) -> TranslationUnit {
        let mut output = TranslationUnit::default();
        for (index, ty) in self.aggregates.iter().enumerate() {
            if host.contains(ty) && self.contains_managed(ty) {
                self.emit_lifetime_helpers(
                    &mut output,
                    ty,
                    representation_owner(ty, index),
                    self.c_type(ty),
                );
            }
        }
        for alias in aliases {
            if host.contains(&alias.ty) && self.contains_managed(&alias.ty) {
                self.emit_alias_lifetime_helpers(&mut output, alias);
            }
        }
        output
    }

    fn emit_lifetime_helpers(
        &self,
        output: &mut TranslationUnit,
        ty: &Type,
        owner: String,
        c_type: TypeName,
    ) {
        output.push(self.host_clone_definition(ty, &owner, c_type.clone()));
        output.blank_line();
        output.push(take_definition(&owner, c_type.clone()));
        output.blank_line();
        output.push(self.host_drop_definition(ty, &owner, c_type));
        output.blank_line();
    }

    fn emit_alias_lifetime_helpers(&self, output: &mut TranslationUnit, alias: &TypeAlias) {
        let owner = &alias.name;
        let target_owner = self.host_owner(&alias.ty);
        let c_type = TypeName::named(format!("MalType_{}", alias.name));
        output.push(FunctionDefinition::from_signature(
            FunctionSignature::static_inline(
                c_type.clone(),
                format!("mal_{owner}_clone"),
                [
                    context_parameter(),
                    Parameter::named(c_type.clone(), "value"),
                ],
            ),
            Block::new([Statement::return_value(Expr::named_call(
                format!("mal_{target_owner}_clone"),
                [Expr::identifier("context"), Expr::identifier("value")],
            ))]),
        ));
        output.blank_line();
        output.push(FunctionDefinition::from_signature(
            FunctionSignature::static_inline(
                c_type.clone(),
                format!("mal_{owner}_take"),
                [Parameter::named(c_type.clone().pointer(), "value")],
            ),
            Block::new([Statement::return_value(Expr::named_call(
                format!("mal_{target_owner}_take"),
                [Expr::identifier("value")],
            ))]),
        ));
        output.blank_line();
        output.push(FunctionDefinition::from_signature(
            FunctionSignature::static_inline(
                "void",
                format!("mal_{owner}_drop"),
                [
                    context_parameter(),
                    Parameter::named(c_type.pointer(), "value"),
                ],
            ),
            Block::new([Statement::call(
                format!("mal_{target_owner}_drop"),
                [Expr::identifier("context"), Expr::identifier("value")],
            )]),
        ));
        output.blank_line();
    }

    fn host_clone_definition(
        &self,
        ty: &Type,
        owner: &str,
        c_type: TypeName,
    ) -> FunctionDefinition {
        let mut body = Block::default();
        match ty {
            Type::Product(elements) => {
                for (index, element) in elements.iter().enumerate() {
                    if self.contains_managed(element) {
                        let field = Expr::identifier("value").field(format!("field_{index}"));
                        body.push(Statement::assignment(
                            field.clone(),
                            self.host_clone_value(element, field),
                        ));
                    }
                }
            }
            Type::Sum(members) => {
                let mut cases = Vec::new();
                for (index, member) in members.iter().enumerate() {
                    let mut case = Block::default();
                    if self.contains_managed(member) {
                        let payload = Expr::identifier("value")
                            .field("payload")
                            .field(format!("variant_{index}"));
                        case.push(Statement::assignment(
                            payload.clone(),
                            self.host_clone_value(member, payload),
                        ));
                    }
                    case.push(Statement::Break);
                    cases.push(SwitchCase::case(uint32(index), case));
                }
                cases.push(invalid_sum_default());
                body.push(Statement::switch(
                    Expr::identifier("value").field("tag"),
                    cases,
                ));
            }
            _ => unreachable!("only aggregate representations need generated helpers"),
        }
        body.push(Statement::return_value(Expr::identifier("value")));
        FunctionDefinition::from_signature(
            FunctionSignature::static_inline(
                c_type.clone(),
                format!("mal_{owner}_clone"),
                [context_parameter(), Parameter::named(c_type, "value")],
            ),
            body,
        )
    }

    fn host_drop_definition(&self, ty: &Type, owner: &str, c_type: TypeName) -> FunctionDefinition {
        let mut body = Block::default();
        match ty {
            Type::Product(elements) => {
                for (index, element) in elements.iter().enumerate().rev() {
                    if self.contains_managed(element) {
                        self.host_drop_value(
                            &mut body,
                            element,
                            Expr::identifier("value").pointer_field(format!("field_{index}")),
                        );
                    }
                }
            }
            Type::Sum(members) => {
                let mut cases = Vec::new();
                for (index, member) in members.iter().enumerate() {
                    let mut case = Block::default();
                    if self.contains_managed(member) {
                        self.host_drop_value(
                            &mut case,
                            member,
                            Expr::identifier("value")
                                .pointer_field("payload")
                                .field(format!("variant_{index}")),
                        );
                    }
                    case.push(Statement::Break);
                    cases.push(SwitchCase::case(uint32(index), case));
                }
                cases.push(invalid_sum_default());
                body.push(Statement::switch(
                    Expr::identifier("value").pointer_field("tag"),
                    cases,
                ));
            }
            _ => unreachable!("only aggregate representations need generated helpers"),
        }
        body.push(Statement::assignment(
            Expr::dereference(Expr::identifier("value")),
            Expr::compound_literal(c_type.clone(), [Initializer::positional(Expr::number("0"))]),
        ));
        FunctionDefinition::from_signature(
            FunctionSignature::static_inline(
                "void",
                format!("mal_{owner}_drop"),
                [
                    context_parameter(),
                    Parameter::named(c_type.pointer(), "value"),
                ],
            ),
            body,
        )
    }

    fn host_clone_value(&self, ty: &Type, value: Expr) -> Expr {
        Expr::named_call(
            format!("mal_{}_clone", self.host_owner(ty)),
            [Expr::identifier("context"), value],
        )
    }

    fn host_drop_value(&self, body: &mut Block, ty: &Type, value: Expr) {
        body.push(Statement::call(
            format!("mal_{}_drop", self.host_owner(ty)),
            [Expr::identifier("context"), Expr::address_of(value)],
        ));
    }

    fn host_owner(&self, ty: &Type) -> String {
        match ty {
            Type::Symbol => "Symbol".into(),
            Type::Product(_) | Type::Sum(_) => representation_owner(ty, self.index(ty)),
            _ => unreachable!("only managed host values have lifecycle owners"),
        }
    }
}

fn take_definition(owner: &str, c_type: TypeName) -> FunctionDefinition {
    FunctionDefinition::from_signature(
        FunctionSignature::static_inline(
            c_type.clone(),
            format!("mal_{owner}_take"),
            [Parameter::named(c_type.clone().pointer(), "value")],
        ),
        Block::new([
            Statement::variable(
                c_type.clone(),
                "result",
                Some(Expr::dereference(Expr::identifier("value"))),
            ),
            Statement::assignment(
                Expr::dereference(Expr::identifier("value")),
                Expr::compound_literal(c_type, [Initializer::positional(Expr::number("0"))]),
            ),
            Statement::return_value(Expr::identifier("result")),
        ]),
    )
}

fn representation_owner(ty: &Type, index: usize) -> String {
    match ty {
        Type::Product(_) => format!("Repr_Product_{index}"),
        Type::Sum(_) => format!("Repr_Sum_{index}"),
        _ => unreachable!("only aggregate types have representation owners"),
    }
}

fn context_parameter() -> Parameter {
    Parameter::named(TypeName::named("MalContext").pointer(), "context")
}

fn uint32(value: usize) -> Expr {
    Expr::named_call("UINT32_C", [Expr::number(value.to_string())])
}

fn invalid_sum_default() -> SwitchCase {
    SwitchCase::default(Block::new([Statement::call(
        "mal_trap",
        [Expr::identifier("context"), Expr::string("invalid sum tag")],
    )]))
}
