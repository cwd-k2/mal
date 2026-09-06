use crate::c_emit::syntax::{
    AggregateDefinition, AggregateField, Block, Declaration, Expr, FunctionDefinition,
    FunctionSignature, Initializer, Parameter, Statement, TranslationUnit, TypeName,
};
use crate::check::ast::Type;
use crate::core::ast::TypeAlias;

use super::{HostTypes, TypeRegistry, is_bool};

mod product;
mod sum;

impl TypeRegistry {
    pub(in crate::c_emit) fn header_declarations(&self, host: &HostTypes) -> TranslationUnit {
        let mut output = TranslationUnit::default();
        for name in &host.opaque_names {
            output.push(AggregateDefinition::typedef_structure(
                None,
                [AggregateField::variable("uintptr_t", "bits")],
                format!("MalType_{name}"),
            ));
        }
        if !host.opaque_names.is_empty() {
            output.blank_line();
        }
        output.extend(self.declarations(host, true));
        output
    }

    pub(in crate::c_emit) fn header_alias_declarations(
        &self,
        host: &HostTypes,
        aliases: &[TypeAlias],
    ) -> TranslationUnit {
        let mut output = TranslationUnit::default();
        for alias in aliases {
            if host.contains(&alias.ty) {
                output.push(Declaration::type_alias(
                    self.c_type(&alias.ty),
                    format!("MalType_{}", alias.name),
                ));
            }
        }
        if !output.is_empty() {
            output.blank_line();
        }
        output
    }

    pub(in crate::c_emit) fn header_opaque_helpers(&self, host: &HostTypes) -> TranslationUnit {
        let mut output = TranslationUnit::default();
        for name in &host.opaque_names {
            append_function(
                &mut output,
                FunctionSignature::static_inline(
                    format!("MalType_{name}"),
                    format!("mal_{name}_from_bits"),
                    [Parameter::named("uintptr_t", "bits")],
                ),
                Block::new([Statement::return_value(Expr::compound_literal(
                    format!("MalType_{name}"),
                    [Initializer::designated("bits", Expr::identifier("bits"))],
                ))]),
            );
            append_function(
                &mut output,
                FunctionSignature::static_inline(
                    "uintptr_t",
                    format!("mal_{name}_bits"),
                    [Parameter::named(format!("MalType_{name}"), "value")],
                ),
                Block::new([Statement::return_value(
                    Expr::identifier("value").field("bits"),
                )]),
            );
        }
        output
    }

    pub(in crate::c_emit) fn header_alias_helpers(
        &self,
        host: &HostTypes,
        aliases: &[TypeAlias],
    ) -> TranslationUnit {
        let mut output = TranslationUnit::default();
        for alias in aliases {
            if !host.contains(&alias.ty) {
                continue;
            }
            match &alias.ty {
                Type::Product(elements) => {
                    self.emit_product_constructor(&mut output, alias, elements);
                    self.emit_product_accessors(&mut output, alias, elements);
                }
                Type::Sum(members) if !is_bool(&alias.ty) => {
                    self.emit_sum_helpers(&mut output, alias, members);
                }
                _ => {}
            }
        }
        output
    }

    pub(in crate::c_emit) fn header_c_type(&self, ty: &Type, alias: Option<&str>) -> TypeName {
        alias.map_or_else(
            || self.c_type(ty),
            |alias| TypeName::named(format!("MalType_{alias}")),
        )
    }

    fn parameters(&self, elements: &[Type]) -> Vec<Parameter> {
        elements
            .iter()
            .enumerate()
            .map(|(index, element)| {
                Parameter::named(self.c_type(element), format!("value_{index}"))
            })
            .collect()
    }
}

fn append_function(output: &mut TranslationUnit, signature: FunctionSignature, body: Block) {
    output.push(FunctionDefinition::from_signature(signature, body));
    output.blank_line();
}
