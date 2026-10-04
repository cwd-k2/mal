//! Canonical-memory read and write helpers for type aliases the checker admitted for host memory access.

use super::*;

impl TypeRegistry {
    pub(super) fn append_alias_memory_helpers(
        &self,
        output: &mut TranslationUnit,
        alias: &TypeAlias,
        layouts: SourceLayouts,
    ) {
        let stride = layouts
            .layout(&alias.ty)
            .expect("checker-approved memory alias has a layout")
            .stride;
        if !matches!(alias.ty, Type::Unit) {
            let helper = match &alias.ty {
                Type::Product(_) | Type::Sum(_) if !is_bool(&alias.ty) => {
                    self.index(&alias.ty).to_string()
                }
                _ => scalar_name(&alias.ty).into(),
            };
            output.push(c_invocation!(MAL_DETAIL_DEFINE_MEMORY_ALIAS(
                { format!("mal_{}_read", alias.name) },
                { format!("mal_{}_write", alias.name) },
                { format!("mal_{}_t", alias.name) },
                { stride },
                { format!("mal_detail_memory_read_{helper}") },
                { format!("mal_detail_memory_write_{helper}") },
            )));
            return;
        }
        let source = offset(
            c_expr!(address as *const uint8_t),
            c_expr!(index * { stride }),
        );
        let unused_index = (stride == 0).then(|| c_statement!(index as void;));
        let read_value = self.memory_read_value(&alias.ty, c_expr!(call), source);
        let read_body = c_block! {
            ..{ unused_index }
            mal_Address_return(call, address);
            return { read_value };
        };
        append_function(
            output,
            c_signature! {
                #[static] #[inline] fn { format!("mal_{}_read", alias.name) }(
                    call: *mut mal_call_t,
                    address: mal_Address_t,
                    index: mal_USize_t,
                ) -> { format!("mal_{}_t", alias.name) }
            },
            read_body,
        );

        let destination = offset(
            c_expr!(address as *mut uint8_t),
            c_expr!(index * { stride }),
        );
        let unused_index = (stride == 0).then(|| c_statement!(index as void;));
        let write_value =
            self.memory_write_statement(&alias.ty, c_expr!(call), destination, c_expr!(value));
        let write_body = c_block! {
            ..{ unused_index }
            mal_Address_return(call, address);
            { write_value }
        };
        append_function(
            output,
            c_signature! {
                #[static] #[inline] fn { format!("mal_{}_write", alias.name) }(
                    call: *mut mal_call_t,
                    address: mal_Address_t,
                    index: mal_USize_t,
                    value: { format!("mal_{}_t", alias.name) },
                ) -> void
            },
            write_body,
        );
    }
}
