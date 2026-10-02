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
            output.push(c_macro_invocation! {
                "MAL_DETAIL_DEFINE_MEMORY_ALIAS"([
                    id(#{ format!("mal_{}_read", alias.name) }),
                    id(#{ format!("mal_{}_write", alias.name) }),
                    id(#{ format!("mal_{}_t", alias.name) }),
                    number(#{ stride }),
                    id(#{ format!("mal_detail_memory_read_{helper}") }),
                    id(#{ format!("mal_detail_memory_write_{helper}") }),
                ])
            });
            return;
        }
        let source = offset(
            c_expr! {
                cast(
                    #{ c_type!(ptr(const(named("uint8_t")))) },
                    (id("address"))
                )
            },
            c_expr!(multiply((id("index")), (number(#{ stride })))),
        );
        let unused_index =
            (stride == 0).then(|| c_statement!(cast((named("void")), (id("index")));));
        let read_value = self.memory_read_value(&alias.ty, c_expr!(id("call")), source);
        let read_body = c_block! {
            ...#{ unused_index }
            call("mal_Address_return", [id("call"), id("address")]);
            return #{ read_value };
        };
        append_function(
            output,
            c_signature! {
                #[static] #[inline] fn #{ format!("mal_{}_read", alias.name) }(
                    "call": ptr(named("mal_call_t")),
                    "address": named("mal_Address_t"),
                    "index": named("mal_USize_t"),
                ) -> named(#{ format!("mal_{}_t", alias.name) })
            },
            read_body,
        );

        let destination = offset(
            c_expr! {
                cast(
                    #{ c_type!(ptr(named("uint8_t"))) },
                    (id("address"))
                )
            },
            c_expr!(multiply((id("index")), (number(#{ stride })))),
        );
        let unused_index =
            (stride == 0).then(|| c_statement!(cast((named("void")), (id("index")));));
        let write_value = self.memory_write_statement(
            &alias.ty,
            c_expr!(id("call")),
            destination,
            c_expr!(id("value")),
        );
        let write_body = c_block! {
            ...#{ unused_index }
            call("mal_Address_return", [id("call"), id("address")]);
            #{ write_value }
        };
        append_function(
            output,
            c_signature! {
                #[static] #[inline] fn #{ format!("mal_{}_write", alias.name) }(
                    "call": ptr(named("mal_call_t")),
                    "address": named("mal_Address_t"),
                    "index": named("mal_USize_t"),
                    "value": named(#{ format!("mal_{}_t", alias.name) }),
                ) -> named("void")
            },
            write_body,
        );
    }
}
