//! Canonical-memory read and write helpers for scalars.

use super::*;

impl TypeRegistry {
    pub(super) fn append_scalar_memory_helpers(&self, output: &mut TranslationUnit, ty: &Type) {
        let name = scalar_name(ty);
        let host_type = self.host_value_c_type(ty, None);
        let call = if matches!(ty, Type::Address) || is_bool(ty) {
            c_parameter!("call": ptr(named("mal_call_t")))
        } else {
            c_parameter!(#[maybe_unused] "call": ptr(named("mal_call_t")))
        };
        let value = if matches!(ty, Type::Address) {
            c_expr!(call("mal_Address_return", [id("call"), id("value")]))
        } else if is_bool(ty) {
            c_expr!(call("mal_Bool_return", [id("call"), id("value")]))
        } else {
            c_expr!(id("value"))
        };
        let read_body = c_block! {
            let "value": #{ host_type.clone() };
            call("memcpy", [
                address((id("value"))),
                id("source"),
                sizeof((id("value"))),
            ]);
            return #{ value };
        };
        append_function(
            output,
            c_signature! {
                #[static] #[inline] fn #{ format!("mal_detail_memory_read_{name}") }(
                    #{ call.clone() },
                    "source": ptr(const(named("uint8_t"))),
                ) -> #{ host_type.clone() }
            },
            read_body,
        );

        let validation = if matches!(ty, Type::Address) {
            Some(c_statement!(call("mal_Address_return", [id("call"), id("value")]);))
        } else if is_bool(ty) {
            Some(c_statement!(call("mal_Bool_return", [id("call"), id("value")]);))
        } else {
            None
        };
        let write_body = c_block! {
            ...#{ validation }
            call("memcpy", [
                id("destination"),
                address((id("value"))),
                sizeof((id("value"))),
            ]);
        };
        append_function(
            output,
            c_signature! {
                #[static] #[inline] fn #{ format!("mal_detail_memory_write_{name}") }(
                    #{ call },
                    "destination": ptr(named("uint8_t")),
                    "value": #{ host_type },
                ) -> named("void")
            },
            write_body,
        );
    }
}
