use crate::backend::c::syntax::{
    AggregateDefinition, AggregateField, Declaration, TranslationUnit, TypeName,
};
use crate::core::ast::TypeAlias;
use mal_frontend::check::ast::Type;

use super::super::{HostTypes, TypeRegistry, is_bool, sum_representation_fields};

impl TypeRegistry {
    pub(in crate::backend::c) fn host_value_declarations(
        &self,
        host: &HostTypes,
        aliases: &[TypeAlias],
    ) -> TranslationUnit {
        let mut output = TranslationUnit::default();
        for name in &host.opaque_names {
            output.push(AggregateDefinition::typedef_structure(
                None,
                [AggregateField::variable("uintptr_t", "mal_detail_bits")],
                format!("mal_{name}_t"),
            ));
        }
        for (index, ty) in self.aggregates.iter().enumerate() {
            if !host.contains(ty) || is_bool(ty) {
                continue;
            }
            let kind = match ty {
                Type::Product(_) => "product",
                Type::Sum(_) => "sum",
                Type::Function { .. } => continue,
                _ => unreachable!("only aggregate types have representation identities"),
            };
            output.push(Declaration::type_alias(
                TypeName::structure(format!("mal_detail_repr_{kind}_{index}")),
                format!("mal_repr_{kind}_{index}_t"),
            ));
        }
        for alias in aliases {
            if host.exposes_alias(alias) {
                output.push(self.host_alias_declaration(alias));
            }
        }
        if !output.is_empty() {
            output.blank_line();
        }
        for (index, ty) in self.aggregates.iter().enumerate() {
            if !host.contains(ty) || is_bool(ty) {
                continue;
            }
            match ty {
                Type::Product(elements) => output.push(AggregateDefinition::structure(
                    format!("mal_detail_repr_product_{index}"),
                    elements.iter().enumerate().map(|(field, ty)| {
                        AggregateField::variable(
                            self.host_value_c_type(ty, None),
                            format!("field_{field}"),
                        )
                    }),
                )),
                Type::Sum(members) => {
                    output.push(AggregateDefinition::structure(
                        format!("mal_detail_repr_sum_{index}"),
                        sum_representation_fields(members, |member| {
                            self.host_value_c_type(member, None)
                        }),
                    ));
                }
                Type::Function { .. } => continue,
                _ => unreachable!("only aggregate types have representation identities"),
            }
            output.blank_line();
        }
        output
    }

    fn host_alias_declaration(&self, alias: &TypeAlias) -> Declaration {
        Declaration::type_alias(
            self.host_value_c_type(&alias.ty, None),
            format!("mal_{}_t", alias.name),
        )
    }

    pub(in crate::backend::c) fn header_declarations(&self, host: &HostTypes) -> TranslationUnit {
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

    pub(in crate::backend::c) fn header_alias_declarations(
        &self,
        host: &HostTypes,
        aliases: &[TypeAlias],
    ) -> TranslationUnit {
        let mut output = TranslationUnit::default();
        for alias in aliases {
            if host.exposes_external_alias(alias) {
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

    pub(in crate::backend::c) fn header_c_type(&self, ty: &Type, alias: Option<&str>) -> TypeName {
        alias.map_or_else(
            || self.c_type(ty),
            |alias| TypeName::named(format!("MalType_{alias}")),
        )
    }
}
