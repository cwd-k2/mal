//! Entry allocas that operations in the emitted states share: host bridge operands, a Symbol result, and one
//! Buffer element value.

use super::*;
use crate::backend::llvm::syntax::llvm_type;

/// Sizes of the shared scratch storage, each `None` or `false` when no state needs it.
pub(super) struct Scratch {
    /// Size and alignment of the largest external argument or result.
    pub(super) external: Option<(usize, usize)>,
    /// Size and alignment of the largest element a `new` or `fill` stores.
    pub(super) buffer_value: Option<(usize, usize)>,
    /// Whether a Symbol operation returns through memory.
    pub(super) symbol_result: bool,
}

impl Scratch {
    pub(super) fn measure(
        execution: &crate::execution::Program,
        index: &ProgramIndex<'_>,
        states: &[StateId],
        types: &Types,
        source_layouts: &crate::backend::source_layout::SourceLayouts,
    ) -> Option<Self> {
        let bindings = || {
            states
                .iter()
                .flat_map(|state| &execution.control.states[state.0].bindings)
        };
        let symbol_result = bindings().any(|binding| {
            matches!(
                &binding.operation,
                Operation::Symbol {
                    primitive: mal_frontend::check::ast::SymbolPrimitive::Concatenate
                        | mal_frontend::check::ast::SymbolPrimitive::Prefix
                        | mal_frontend::check::ast::SymbolPrimitive::Suffix,
                    ..
                }
            )
        });
        let buffer_value = largest(
            bindings()
                .filter_map(|binding| match &binding.operation {
                    Operation::Buffer {
                        operation:
                            crate::core::ast::BufferOperation::New
                            | crate::core::ast::BufferOperation::Fill,
                        element,
                        ..
                    } => source_layouts
                        .layout(element)
                        .map(|layout| (layout.stride, layout.alignment))
                        .or_else(|| {
                            let value = types.value(element)?;
                            Some((value.size, value.alignment))
                        }),
                    _ => None,
                })
                .filter(|(size, _)| *size != 0),
        );
        let mut externals = Vec::new();
        for binding in bindings() {
            if let Operation::ExternalCall { id, .. } = binding.operation {
                let external = *index.externals.get(&id)?;
                for ty in [&external.parameter, &external.result] {
                    let value = types.value(ty)?;
                    externals.push((value.size, value.alignment));
                }
            }
        }
        Some(Self {
            external: largest(externals.into_iter()),
            buffer_value,
            symbol_result,
        })
    }
}

/// The largest size and the largest alignment among `layouts`, or `None` when there are none.
fn largest(layouts: impl Iterator<Item = (usize, usize)>) -> Option<(usize, usize)> {
    layouts.fold(None, |storage, (size, alignment)| {
        let (current_size, current_alignment) = storage.unwrap_or((0usize, 1usize));
        Some((current_size.max(size), current_alignment.max(alignment)))
    })
}

impl FunctionEmitter<'_> {
    pub(super) fn emit_scratch_allocas(&mut self) -> Option<()> {
        if let Some((size, alignment)) = self.external_storage {
            let storage_type = llvm_type!(array(#{ size }, int(8)));
            emit_instruction! {
                self;
                let "%mal_bridge_argument" = alloca {
                    ty: #{ storage_type.clone() },
                    alignment: #{ alignment },
                };
            };
            emit_instruction! {
                self;
                let "%mal_bridge_result" = alloca {
                    ty: #{ storage_type },
                    alignment: #{ alignment },
                };
            };
        }
        if self.needs_symbol_result_slot {
            let symbol = self.types.value(&Type::Symbol)?;
            emit_instruction! {
                self;
                let "%mal_symbol_result" = alloca {
                    ty: #{ symbol.llvm },
                    alignment: #{ symbol.alignment },
                };
            };
        }
        if let Some((size, alignment)) = self.buffer_value_storage {
            emit_instruction! {
                self;
                let "%mal_buffer_value" = alloca {
                    ty: #{ llvm_type!(array(#{ size }, int(8_u16))) },
                    alignment: #{ alignment },
                };
            };
        }
        Some(())
    }
}
