//! Canonical-memory read and write helpers for scalars.

use super::*;

impl TypeRegistry {
    pub(super) fn append_scalar_memory_helpers(&self, output: &mut TranslationUnit, ty: &Type) {
        let name = scalar_name(ty);
        let host_type = self.host_value_c_type(ty, None);
        let call = if matches!(ty, Type::Address) || is_bool(ty) {
            c_parameter!(call: *mut mal_call_t)
        } else {
            c_parameter!(#[maybe_unused] call: *mut mal_call_t)
        };
        let value = if matches!(ty, Type::Address) {
            c_expr!(mal_Address_return(call, value))
        } else if is_bool(ty) {
            c_expr!(mal_Bool_return(call, value))
        } else {
            c_expr!(value)
        };
        let read_body = c_block! {
            let value: { host_type.clone() };
            memcpy(&value, source, sizeof(value));
            return { value };
        };
        append_function(
            output,
            c_signature! {
                #[static] #[inline] fn { format!("mal_detail_memory_read_{name}") }(
                    { call.clone() },
                    source: *const uint8_t,
                ) -> { host_type.clone() }
            },
            read_body,
        );

        let validation = if matches!(ty, Type::Address) {
            Some(c_statement!(mal_Address_return(call, value);))
        } else if is_bool(ty) {
            Some(c_statement!(mal_Bool_return(call, value);))
        } else {
            None
        };
        let write_body = c_block! {
            ..{ validation }
            memcpy(destination, &value, sizeof(value));
        };
        append_function(
            output,
            c_signature! {
                #[static] #[inline] fn { format!("mal_detail_memory_write_{name}") }(
                    { call },
                    destination: *mut uint8_t,
                    value: { host_type },
                ) -> void
            },
            write_body,
        );
    }
}
