use crate::backend::c::syntax::{TranslationUnit, TypeName, c_expr, c_invocation, c_items, c_type};
use crate::core::ast::TypeAlias;
use mal_frontend::check::ast::Type;

use super::super::{HostTypes, TypeRegistry, is_bool};

impl TypeRegistry {
    pub(in crate::backend::c) fn host_value_declarations(
        &self,
        host: &HostTypes,
        aliases: &[TypeAlias],
    ) -> TranslationUnit {
        let mut output = TranslationUnit::default();
        for name in &host.opaque_names {
            let alias = format!("mal_{name}_t");
            output.extend(c_items! {
                type { alias } = struct {
                    mal_detail_bits: uintptr_t,
                };
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
            let guard = format!("MAL_DETAIL_HOST_REPR_{id}_DECLARED");
            let alias = format!("mal_repr_{kind}_{id}_t");
            output.extend(c_items! {
                if !defined({ guard.clone() }) {
                    define!({ guard });
                    type { alias } = Struct<{ format!("mal_detail_repr_{kind}_{id}") }>;
                }
            });
        }
        for alias in aliases {
            if host.exposes_alias(alias) {
                output.extend(self.host_alias_declaration(alias));
            }
        }
        if !output.is_empty() {
            output.blank_line();
        }
        for ty in &self.aggregates {
            if !host.contains(ty) || is_bool(ty) {
                continue;
            }
            if !host.external_contains(ty) && matches!(ty, Type::Product(_) | Type::Sum(_)) {
                self.append_repr_descriptor(&mut output, ty);
            }
            let guard = format!("MAL_DETAIL_HOST_REPR_{}_DEFINED", self.index(ty));
            let mut guarded = c_items! { define!({ guard.clone() }); };
            match ty {
                Type::Product(_) => {
                    let tag = format!("mal_detail_repr_product_{}", self.index(ty));
                    guarded.push(c_invocation!(MAL_DETAIL_DEFINE_PRODUCT_REPR(
                        { tag },
                        { format!("MAL_DETAIL_REPR_FIELDS_{}", self.index(ty)) },
                        MAL_DETAIL_HOST_REPR_FIELD,
                    )));
                }
                Type::Sum(members) => {
                    let tag = format!("mal_detail_repr_sum_{}", self.index(ty));
                    let template = if members.is_empty() {
                        "MAL_DETAIL_DEFINE_EMPTY_SUM_REPR"
                    } else {
                        "MAL_DETAIL_DEFINE_SUM_REPR"
                    };
                    let mut arguments = vec![c_expr!({ tag })];
                    if !members.is_empty() {
                        arguments.push(c_expr!({
                            format!("MAL_DETAIL_REPR_FIELDS_{}", self.index(ty))
                        }));
                        arguments.push(c_expr!(MAL_DETAIL_HOST_REPR_FIELD));
                    }
                    guarded.push(c_invocation!({ template }(..{ arguments })));
                }
                Type::Function { .. } => continue,
                _ => unreachable!("only aggregate types have representation identities"),
            }
            output.extend(c_items! {
                if !defined({ guard }) {
                    ..{ guarded }
                }
            });
            output.blank_line();
        }
        output
    }

    fn host_alias_declaration(&self, alias: &TypeAlias) -> TranslationUnit {
        let name = format!("mal_{}_t", alias.name);
        let source = self.host_value_c_type(&alias.ty, None);
        c_items! { type { name } = { source }; }
    }

    pub(in crate::backend::c) fn header_declarations(&self, host: &HostTypes) -> TranslationUnit {
        let mut output = TranslationUnit::default();
        for name in &host.opaque_names {
            let alias = format!("MalType_{name}");
            output.extend(c_items! {
                type { alias } = struct {
                    bits: uintptr_t,
                };
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
                let source = self.c_type(&alias.ty);
                output.extend(c_items! { type { name } = { source }; });
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
            |alias| c_type!({ format!("MalType_{alias}") }),
        )
    }
}
