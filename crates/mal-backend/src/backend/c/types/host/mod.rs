use crate::backend::c::syntax::{
    Block, FunctionDefinition, FunctionSignature, TranslationUnit, c_block, c_signature,
};

use super::{HostTypes, TypeRegistry};

mod declaration;
mod lifecycle;

impl TypeRegistry {
    pub(in crate::backend::c) fn host_value_helpers(&self, host: &HostTypes) -> TranslationUnit {
        let mut output = TranslationUnit::default();
        for name in &host.opaque_names {
            let ty = format!("mal_{name}_t");
            append_function(
                &mut output,
                c_signature! {
                    #[static] #[inline] #[overloadable] fn mal_detail_from_bits(
                        #[maybe_unused] type_marker: *mut { ty.clone() },
                        bits: uintptr_t,
                    ) -> { ty.clone() }
                },
                c_block! { return { ty.clone() } { mal_detail_bits: bits }; },
            );
            append_function(
                &mut output,
                c_signature! {
                    #[static] #[inline] #[overloadable] fn mal_detail_bits(
                        value: { ty.clone() },
                    ) -> uintptr_t
                },
                c_block! { return value.mal_detail_bits; },
            );
        }
        output
    }
}

fn append_function(output: &mut TranslationUnit, signature: FunctionSignature, body: Block) {
    output.push(FunctionDefinition::from_signature(signature, body));
    output.blank_line();
}
