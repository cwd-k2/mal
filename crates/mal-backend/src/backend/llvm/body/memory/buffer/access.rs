//! Direct element access: the active data pointer, element addresses, and the runtime-owned get and put paths.

use super::*;

impl FunctionEmitter<'_> {
    pub(in crate::backend::llvm::body) fn active_buffer_data(
        &mut self,
        buffer: &EmittedValue,
    ) -> String {
        let slot = self.register();
        emit_instruction! {
            self;
            let #{ slot.clone() } = call {
                tail: false,
                result_type: (ptr),
                callee: direct("mal_runtime_buffer_data_slot"),
                arguments: [typed((ptr), #{ buffer.representation.clone() })],
            };
        };
        let data = self.register();
        emit_instruction! {
            self;
            let #{ data.clone() } = load {
                ty: (ptr),
                pointer: #{ slot },
                alignment: #{ self.types.pointer_alignment() },
                metadata: #{ [
                    MetadataAttachment::Tbaa(8),
                    MetadataAttachment::AliasScope(6),
                ] },
            };
        };
        data
    }

    pub(super) fn buffer_value_pointer(
        &mut self,
        value: &EmittedValue,
        element_storage: ElementStorage,
    ) -> Option<String> {
        if element_storage.stride() == 0 {
            return Some("null".into());
        }
        let storage = "%mal_buffer_value";
        match element_storage {
            ElementStorage::Canonical { stride } => {
                let layout = self.source_layouts.layout(&value.ty)?;
                emit_instruction! {
                    self;
                    store {
                        value: typed(#{ llvm_type!(array(#{ stride }, int(8_u16))) }, "zeroinitializer"),
                        pointer: #{ storage },
                        alignment: #{ layout.alignment },
                        metadata: [],
                    };
                };
                self.emit_aligned_source_store_at(storage, value)?;
            }
            ElementStorage::Runtime { alignment, .. } => {
                let value_type = self.types.value(&value.ty)?;
                emit_instruction! {
                    self;
                    store {
                        value: typed(#{ value_type.llvm }, #{ value.representation.as_str() }),
                        pointer: #{ storage },
                        alignment: #{ alignment },
                        metadata: [],
                    };
                };
            }
        }
        Some(storage.into())
    }

    /// Loads an internal carrier and creates the result responsibility only when the stored lifecycle owns one.
    pub(super) fn emit_runtime_element_get(
        &mut self,
        pointer: &str,
        element: &Type,
        alignment: usize,
        lifecycle: Lifecycle,
    ) -> Option<EmittedValue> {
        let value_type = self.types.value(element)?;
        let loaded = self.register();
        emit_instruction! {
            self;
            let #{ loaded.clone() } = load {
                ty: #{ value_type.llvm },
                pointer: #{ pointer },
                alignment: #{ alignment },
                metadata: #{ buffer_element_metadata() },
            };
        };
        if lifecycle == Lifecycle::Owned {
            self.retain_value(element, &loaded)?;
        }
        Some(EmittedValue {
            ty: element.clone(),
            representation: loaded,
            owned: lifecycle == Lifecycle::Owned,
        })
    }

    /// Stores an internal carrier. Owned values transfer a responsibility into the place; trivial values need only a
    /// typed store because replacing their bits has no lifecycle effect.
    pub(super) fn emit_runtime_element_put(
        &mut self,
        pointer: &str,
        value: &EmittedValue,
        alignment: usize,
        lifecycle: Lifecycle,
    ) -> Option<()> {
        if lifecycle == Lifecycle::Owned {
            return self.move_into_managed_place(
                pointer,
                value,
                alignment,
                &buffer_element_metadata(),
            );
        }
        let value_type = self.types.value(&value.ty)?;
        emit_instruction! {
            self;
            store {
                value: typed(#{ value_type.llvm }, #{ value.representation.clone() }),
                pointer: #{ pointer },
                alignment: #{ alignment },
                metadata: #{ buffer_element_metadata() },
            };
        };
        Some(())
    }

    pub(super) fn buffer_element_pointer(
        &mut self,
        data: &str,
        index: &EmittedValue,
        stride: usize,
    ) -> Option<String> {
        if index.ty != Type::USize {
            return None;
        }
        let offset = self.register();
        emit_instruction! {
            self;
            let #{ offset.clone() } = binary {
                operator: #{ BinaryOperator::Mul },
                ty: #{ self.types.index_llvm_type() },
                left: #{ index.representation.clone() },
                right: #{ stride.to_string() },
            };
        };
        let pointer = self.register();
        emit_instruction! {
            self;
            let #{ pointer.clone() } = get_element_ptr {
                inbounds: false,
                element_type: (int(8_u16)),
                pointer: #{ data },
                indices: [typed(#{ self.types.index_llvm_type() }, #{ offset })],
            };
        };
        Some(pointer)
    }
}

fn buffer_element_metadata() -> [MetadataAttachment; 2] {
    [MetadataAttachment::Tbaa(3), MetadataAttachment::NoAlias(6)]
}
