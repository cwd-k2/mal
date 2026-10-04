use crate::backend::c::syntax::{Directive, Expr, TranslationUnit, c_invocation, c_items};
use crate::core::ast::TypeAlias;
use mal_frontend::check::ast::Type;

use super::super::{HostTypes, TypeRegistry, is_bool};

impl TypeRegistry {
    pub(in crate::backend::c) fn host_lifecycle_helpers(
        &self,
        host: &HostTypes,
        aliases: &[TypeAlias],
    ) -> TranslationUnit {
        let mut output = TranslationUnit::default();
        for ty in &self.aggregates {
            if !host.contains(ty) || is_bool(ty) || !self.has_managed_leaf(ty) {
                continue;
            }
            let id = self.index(ty);
            let guard = format!("MAL_DETAIL_REPR_{id}_LIFECYCLE");
            let kind = match ty {
                Type::Product(_) => "PRODUCT",
                Type::Sum(_) => "SUM",
                _ => unreachable!("only products and sums have host lifecycle"),
            };
            let host_type = self.host_value_c_type(ty, None);
            let host_type_name = host_type.to_string();
            let cleanup = format!("mal_detail_cleanup_{id}");
            let invocation = c_invocation!({ format!("MAL_DETAIL_DEFINE_{kind}_LIFECYCLE") }(
                { host_type_name },
                { format!("MAL_DETAIL_REPR_FIELDS_{id}") },
                { format!("mal_detail_storage_share_{id}") },
                { format!("mal_detail_storage_drop_{id}") },
                { cleanup },
            ));
            output.extend(c_items! {
                if !defined({ guard.clone() }) {
                    define!({ guard });
                    { invocation }
                }
            });
        }
        for alias in aliases.iter().filter(|alias| host.exposes_alias(alias)) {
            let Some(cleanup) = self.managed_cleanup_name(&alias.ty) else {
                continue;
            };
            output.push(Directive::define_expr(
                format!("MAL_DETAIL_CLEANUP_{}", alias.name),
                Expr::identifier(cleanup),
            ));
        }
        output
    }

    fn managed_cleanup_name(&self, ty: &Type) -> Option<String> {
        match ty {
            Type::Symbol => Some("mal_detail_cleanup_Symbol".into()),
            Type::Buffer(_) => Some("mal_detail_cleanup_Buffer".into()),
            Type::Product(_) | Type::Sum(_) if self.has_managed_leaf(ty) => {
                Some(format!("mal_detail_cleanup_{}", self.index(ty)))
            }
            _ => None,
        }
    }

    fn has_managed_leaf(&self, ty: &Type) -> bool {
        let mut pending = vec![ty];
        while let Some(ty) = pending.pop() {
            match ty {
                Type::Symbol | Type::Buffer(_) => return true,
                Type::Product(elements) | Type::Sum(elements) => pending.extend(elements.iter()),
                _ => {}
            }
        }
        false
    }
}
