//! Field descriptors shared by generated aggregate declarations.

use crate::backend::c::syntax::{Directive, TranslationUnit, c_invocation, c_items};
use mal_frontend::check::ast::Type;

use super::TypeRegistry;

impl TypeRegistry {
    pub(super) fn append_repr_descriptor(&self, output: &mut TranslationUnit, ty: &Type) {
        const FIELDS_PER_CHUNK: usize = 32;

        let (member_prefix, elements) = match ty {
            Type::Product(elements) => ("field", elements.as_ref()),
            Type::Sum(elements) => ("variant", elements.as_ref()),
            _ => unreachable!("only products and sums have field descriptors"),
        };
        let id = self.index(ty);
        let descriptor = format!("MAL_DETAIL_REPR_FIELDS_{id}");
        let guard = format!("{descriptor}_DEFINED");
        let mut guarded = c_items! { define!({ guard.clone() }); };
        let fields = elements
            .iter()
            .enumerate()
            .map(|(index, element)| {
                c_invocation!(field(
                    context,
                    { index },
                    { format!("{member_prefix}_{index}") },
                    { self.host_value_c_type(element, None).to_string() },
                ))
            })
            .collect::<Vec<_>>();

        if fields.len() <= FIELDS_PER_CHUNK {
            guarded.push(Directive::invocations_define(
                descriptor,
                ["field", "context"],
                fields,
            ));
        } else {
            let mut chunks = Vec::new();
            for (chunk_index, fields) in fields.chunks(FIELDS_PER_CHUNK).enumerate() {
                let chunk = format!("{descriptor}_{chunk_index}");
                guarded.push(Directive::invocations_define(
                    chunk.clone(),
                    ["field", "context"],
                    fields.iter().cloned(),
                ));
                chunks.push(c_invocation!({ chunk }(field, context)));
            }
            guarded.push(Directive::invocations_define(
                descriptor,
                ["field", "context"],
                chunks,
            ));
        }
        output.extend(c_items! {
            if !defined({ guard }) {
                ..{ guarded }
            }
        });
    }
}
