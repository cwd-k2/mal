//! Canonical memory layout access shared by Buffer elements and host memory: loads, stores, and field addresses.

mod load;
mod store;

use super::super::FunctionEmitter;

impl FunctionEmitter<'_> {
    /// The address `offset` bytes past `pointer`.
    pub(super) fn source_pointer_offset(&mut self, pointer: &str, offset: usize) -> String {
        if offset == 0 {
            return pointer.to_string();
        }
        let field = self.register();
        emit_instruction! {
            self;
            let #{ field.clone() } = get_element_ptr {
                inbounds: false,
                element_type: (int(8_u16)),
                pointer: #{ pointer },
                indices: [typed(#{ self.types.index_llvm_type() }, #{ offset.to_string() })],
            };
        };
        field
    }
}
