use crate::backend::c::syntax::{TranslationUnit, c_function, c_invocation, c_items};
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
            let invocation = c_invocation!({ format!("MAL_DETAIL_DEFINE_{kind}_LIFECYCLE") }(
                { host_type_name },
                { format!("MAL_DETAIL_REPR_FIELDS_{id}") },
                { format!("mal_detail_storage_share_{id}") },
                { format!("mal_detail_storage_drop_{id}") },
            ));
            output.extend(c_items! {
                if !defined({ guard.clone() }) {
                    define!({ guard });
                    { invocation }
                }
            });
        }
        for name in host.opaque_names.iter().chain(
            aliases
                .iter()
                .filter(|alias| host.exposes_alias(alias))
                .map(|alias| &alias.name),
        ) {
            output.push(c_function! {
                #[static] #[inline] fn { format!("mal_detail_cleanup_{name}") }(
                    value: *mut { format!("mal_{name}_t") },
                ) -> void {
                    mal_detail_release(value);
                    memset(value, 0, sizeof(*value));
                }
            });
        }
        output
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
