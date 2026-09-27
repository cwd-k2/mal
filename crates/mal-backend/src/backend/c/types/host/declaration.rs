use crate::backend::c::syntax::{
    Declaration, TranslationUnit, TypeName, c_aggregate, c_aggregate_field, c_declaration, c_type,
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
            let alias = format!("mal_{name}_t");
            output.push(c_aggregate! {
                type #{ alias } = struct {
                    "mal_detail_bits": named("uintptr_t"),
                }
            });
        }
        for ty in &self.aggregates {
            if !host.contains(ty) || is_bool(ty) {
                continue;
            }
            let kind = match ty {
                Type::Product(_) => "product",
                Type::Sum(_) => "sum",
                Type::Function { .. } => continue,
                _ => unreachable!("only aggregate types have representation identities"),
            };
            let id = self.index(ty);
            let alias = format!("mal_repr_{kind}_{id}_t");
            output.push(c_declaration! {
                type #{ alias } =
                    struct(#{ format!("mal_detail_repr_{kind}_{id}") })
            });
        }
        for alias in aliases {
            if host.exposes_alias(alias) {
                output.push(self.host_alias_declaration(alias));
            }
        }
        if !output.is_empty() {
            output.blank_line();
        }
        for ty in &self.aggregates {
            if !host.contains(ty) || is_bool(ty) {
                continue;
            }
            match ty {
                Type::Product(elements) => {
                    let fields = elements.iter().enumerate().map(|(field, ty)| {
                        let name = format!("field_{field}");
                        c_aggregate_field!(#{ name } : #{ self.host_value_c_type(ty, None) })
                    });
                    let tag = format!("mal_detail_repr_product_{}", self.index(ty));
                    output.push(c_aggregate!(struct #{ tag } { ...#{ fields } }));
                }
                Type::Sum(members) => {
                    let fields = sum_representation_fields(members, |member| {
                        self.host_value_c_type(member, None)
                    });
                    let tag = format!("mal_detail_repr_sum_{}", self.index(ty));
                    output.push(c_aggregate!(struct #{ tag } { ...#{ fields } }));
                }
                Type::Function { .. } => continue,
                _ => unreachable!("only aggregate types have representation identities"),
            }
            output.blank_line();
        }
        output
    }

    fn host_alias_declaration(&self, alias: &TypeAlias) -> Declaration {
        let name = format!("mal_{}_t", alias.name);
        c_declaration! {
            type #{ name } = #{
                self.host_value_c_type(&alias.ty, None)
            }
        }
    }

    pub(in crate::backend::c) fn header_declarations(&self, host: &HostTypes) -> TranslationUnit {
        let mut output = TranslationUnit::default();
        for name in &host.opaque_names {
            let alias = format!("MalType_{name}");
            output.push(c_aggregate! {
                type #{ alias } = struct {
                    "bits": named("uintptr_t"),
                }
            });
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
                let name = format!("MalType_{}", alias.name);
                output.push(c_declaration! {
                    type #{ name } = #{
                        self.c_type(&alias.ty)
                    }
                });
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
            |alias| c_type!(named(#{ format!("MalType_{alias}") })),
        )
    }
}
