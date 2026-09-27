use mal_frontend::check::ast::Type;

use super::super::{EmittedValue, FunctionEmitter};
use crate::backend::llvm::syntax::{CastOperator, MetadataAttachment, llvm_type};

impl FunctionEmitter<'_> {
    pub(in crate::backend::llvm::body) fn emit_aligned_buffer_load_at(
        &mut self,
        pointer: &str,
        element: &Type,
    ) -> Option<EmittedValue> {
        self.emit_source_load_at_with_alignment(
            pointer,
            element,
            true,
            &[MetadataAttachment::Tbaa(3), MetadataAttachment::NoAlias(6)],
        )
    }

    fn emit_source_load_at_with_alignment(
        &mut self,
        pointer: &str,
        element: &Type,
        aligned: bool,
        metadata: &[MetadataAttachment],
    ) -> Option<EmittedValue> {
        if *element == Type::Unit {
            return Some(EmittedValue {
                ty: Type::Unit,
                representation: "0".into(),
                owned: false,
            });
        }
        if let Type::Product(elements) = element {
            let fields = self.source_layouts.product_fields(element)?;
            let product_type = self.types.value(element)?;
            let mut product = "poison".to_string();
            for (index, (field, field_type)) in fields.iter().zip(elements.iter()).enumerate() {
                let field_pointer = self.source_pointer_offset(pointer, field.offset);
                let field_value = self.emit_source_load_at_with_alignment(
                    &field_pointer,
                    field_type,
                    aligned,
                    metadata,
                )?;
                let llvm_type = self.types.value(field_type)?;
                let inserted = self.register();
                emit_instruction! {
                    self;
                    let {{ inserted.clone() }} = insert_value {
                        aggregate: typed({{ product_type.llvm.clone() }}, {{ product }}),
                        element: typed({{ llvm_type.llvm }}, {{ field_value.representation }}),
                        indices: [{{ index }}],
                    };
                };
                product = inserted;
            }
            return Some(EmittedValue {
                ty: element.clone(),
                representation: product,
                owned: false,
            });
        }
        if let Type::Sum(variants) = element {
            let layout = self.source_layouts.sum(element)?;
            let source_tag = self.register();
            let alignment = if aligned {
                self.source_layouts.layout(element)?.alignment
            } else {
                1
            };
            emit_instruction! {
                self;
                let {{ source_tag.clone() }} = load {
                    ty: {{ llvm_type!(int({{ u16::try_from(layout.tag_bits).ok()? }})) }},
                    pointer: {{ pointer }},
                    alignment: {{ alignment }},
                    metadata: {{ metadata.iter().copied() }},
                };
            };
            if super::super::types::is_bool(element) {
                let value = self.register();
                emit_instruction! {
                    self;
                    let {{ value.clone() }} = cast {
                        operator: {{ CastOperator::Trunc }},
                        value: typed((int(8_u16)), {{ source_tag }}),
                        to: (int(1_u16)),
                    };
                };
                return Some(EmittedValue {
                    ty: element.clone(),
                    representation: value,
                    owned: false,
                });
            }
            let tag = if layout.tag_bits == 32 {
                source_tag
            } else if layout.tag_bits < 32 {
                let extended = self.register();
                emit_instruction! {
                    self;
                    let {{ extended.clone() }} = cast {
                        operator: {{ CastOperator::ZExt }},
                        value: typed(
                            {{ llvm_type!(int({{ u16::try_from(layout.tag_bits).ok()? }})) }},
                            {{ source_tag }},
                        ),
                        to: (int(32_u16)),
                    };
                };
                extended
            } else {
                let narrowed = self.register();
                emit_instruction! {
                    self;
                    let {{ narrowed.clone() }} = cast {
                        operator: {{ CastOperator::Trunc }},
                        value: typed((int(64_u16)), {{ source_tag }}),
                        to: (int(32_u16)),
                    };
                };
                narrowed
            };
            let stem = self.register();
            let stem = stem.trim_start_matches('%').to_string();
            let runtime = self.types.value(element)?;
            let storage = self.entry_alloca(&runtime.llvm, runtime.alignment);
            let payload_pointer = self.source_pointer_offset(pointer, layout.payload_offset);
            let cases = variants
                .iter()
                .enumerate()
                .map(|(index, _)| (index.to_string(), format!("{stem}_variant_{index}")));
            emit_terminator! {
                self;
                switch typed((int(32_u16)), {{ tag }})  {
                    cases: [...{{ cases }}],
                    default: {{ format!("{stem}_invalid") }},
                };
            };
            self.block(format!("{stem}_invalid"));
            emit_terminator! {
                self;
                unreachable;
            };
            for (index, variant) in variants.iter().enumerate() {
                self.block(format!("{stem}_variant_{index}"));
                let payload = self.emit_source_load_at_with_alignment(
                    &payload_pointer,
                    variant,
                    aligned,
                    metadata,
                )?;
                let sum = self.emit_sum_value(index, payload, element, false)?;
                emit_instruction! {
                    self;
                    store {
                        value: typed({{ runtime.llvm.clone() }}, {{ sum.representation }}),
                        pointer: {{ storage.as_str() }},
                        alignment: {{ runtime.alignment }},
                        metadata: [],
                    };
                };
                emit_terminator! {
                    self;
                    branch {
                        target: {{ format!("{stem}_loaded") }},
                    };
                };
            }
            self.block(format!("{stem}_loaded"));
            let result = self.register();
            emit_instruction! {
                self;
                let {{ result.clone() }} = load {
                    ty: {{ runtime.llvm }},
                    pointer: {{ storage }},
                    alignment: {{ runtime.alignment }},
                    metadata: [],
                };
            };
            return Some(EmittedValue {
                ty: element.clone(),
                representation: result,
                owned: false,
            });
        }
        if !matches!(
            element,
            Type::Int8
                | Type::Int16
                | Type::Int32
                | Type::Int64
                | Type::UInt8
                | Type::UInt16
                | Type::UInt32
                | Type::UInt64
                | Type::Float32
                | Type::Float64
                | Type::Address
                | Type::ByteSize
                | Type::USize
        ) {
            return None;
        }
        let value_type = self.types.value(element)?;
        let alignment = if aligned {
            self.source_layouts.layout(element)?.alignment
        } else {
            1
        };
        let value = self.register();
        emit_instruction! {
            self;
            let {{ value.clone() }} = load {
                ty: {{ value_type.llvm }},
                pointer: {{ pointer }},
                alignment: {{ alignment }},
                metadata: {{ metadata.iter().copied() }},
            };
        };
        Some(EmittedValue {
            ty: element.clone(),
            representation: value,
            owned: false,
        })
    }

    pub(in crate::backend::llvm::body) fn emit_aligned_source_store_at(
        &mut self,
        pointer: &str,
        value: &EmittedValue,
    ) -> Option<()> {
        self.emit_source_store_at_with_alignment(pointer, value, true, &[])
    }

    pub(in crate::backend::llvm::body) fn emit_aligned_buffer_store_at(
        &mut self,
        pointer: &str,
        value: &EmittedValue,
    ) -> Option<()> {
        self.emit_source_store_at_with_alignment(
            pointer,
            value,
            true,
            &[MetadataAttachment::Tbaa(3), MetadataAttachment::NoAlias(6)],
        )
    }

    fn emit_source_store_at_with_alignment(
        &mut self,
        pointer: &str,
        value: &EmittedValue,
        aligned: bool,
        metadata: &[MetadataAttachment],
    ) -> Option<()> {
        if value.ty == Type::Unit {
            return Some(());
        }
        if let Type::Product(elements) = &value.ty {
            let fields = self.source_layouts.product_fields(&value.ty)?;
            let runtime = self.types.value(&value.ty)?;
            for (index, (field, field_type)) in fields.iter().zip(elements.iter()).enumerate() {
                let field_value = self.register();
                emit_instruction! {
                    self;
                    let {{ field_value.clone() }} = extract_value {
                        aggregate: typed({{ runtime.llvm.clone() }}, {{ value.representation.clone() }}),
                        indices: [{{ index }}],
                    };
                };
                let field_pointer = self.source_pointer_offset(pointer, field.offset);
                self.emit_source_store_at_with_alignment(
                    &field_pointer,
                    &EmittedValue {
                        ty: field_type.clone(),
                        representation: field_value,
                        owned: false,
                    },
                    aligned,
                    metadata,
                )?;
            }
            return Some(());
        }
        if let Type::Sum(variants) = &value.ty {
            let layout = self.source_layouts.sum(&value.ty)?;
            if super::super::types::is_bool(&value.ty) {
                let tag = self.register();
                emit_instruction! {
                    self;
                    let {{ tag.clone() }} = cast {
                        operator: {{ CastOperator::ZExt }},
                        value: typed((int(1_u16)), {{ value.representation.clone() }}),
                        to: (int(8_u16)),
                    };
                };
                emit_instruction! {
                    self;
                    store {
                        value: typed((int(8_u16)), {{ &tag }}),
                        pointer: {{ pointer }},
                        alignment: 1,
                        metadata: {{ metadata.iter().copied() }},
                    };
                };
                return Some(());
            }
            let runtime = self.types.value(&value.ty)?;
            let tag = self.register();
            emit_instruction! {
                self;
                let {{ tag.clone() }} = extract_value {
                    aggregate: typed({{ runtime.llvm }}, {{ value.representation.clone() }}),
                    indices: [0],
                };
            };
            let source_tag = if layout.tag_bits == 32 {
                tag.clone()
            } else if layout.tag_bits < 32 {
                let narrowed = self.register();
                emit_instruction! {
                    self;
                    let {{ narrowed.clone() }} = cast {
                        operator: {{ CastOperator::Trunc }},
                        value: typed((int(32_u16)), {{ &tag }}),
                        to: {{ llvm_type!(int({{ u16::try_from(layout.tag_bits).ok()? }})) }},
                    };
                };
                narrowed
            } else {
                let extended = self.register();
                emit_instruction! {
                    self;
                    let {{ extended.clone() }} = cast {
                        operator: {{ CastOperator::ZExt }},
                        value: typed((int(32_u16)), {{ &tag }}),
                        to: (int(64_u16)),
                    };
                };
                extended
            };
            let alignment = if aligned {
                self.source_layouts.layout(&value.ty)?.alignment
            } else {
                1
            };
            emit_instruction! {
                self;
                store {
                    value: typed({{ llvm_type!(int({{ u16::try_from(layout.tag_bits).ok()? }})) }}, {{ source_tag }}),
                    pointer: {{ pointer }},
                    alignment: {{ alignment }},
                    metadata: {{ metadata.iter().copied() }},
                };
            };
            let stem = self.register();
            let stem = stem.trim_start_matches('%').to_string();
            let payload_pointer = self.source_pointer_offset(pointer, layout.payload_offset);
            let cases = variants
                .iter()
                .enumerate()
                .map(|(index, _)| (index.to_string(), format!("{stem}_variant_{index}")));
            emit_terminator! {
                self;
                switch typed((int(32_u16)), {{ tag }})  {
                    cases: [...{{ cases }}],
                    default: {{ format!("{stem}_invalid") }},
                };
            };
            self.block(format!("{stem}_invalid"));
            emit_terminator! {
                self;
                unreachable;
            };
            for (index, variant) in variants.iter().enumerate() {
                self.block(format!("{stem}_variant_{index}"));
                let payload = self.emit_sum_payload(&value.ty, variant, &value.representation)?;
                self.emit_source_store_at_with_alignment(
                    &payload_pointer,
                    &EmittedValue {
                        ty: variant.clone(),
                        representation: payload,
                        owned: false,
                    },
                    aligned,
                    metadata,
                )?;
                emit_terminator! {
                    self;
                    branch {
                        target: {{ format!("{stem}_stored") }},
                    };
                };
            }
            self.block(format!("{stem}_stored"));
            return Some(());
        }
        if !matches!(
            value.ty,
            Type::Int8
                | Type::Int16
                | Type::Int32
                | Type::Int64
                | Type::UInt8
                | Type::UInt16
                | Type::UInt32
                | Type::UInt64
                | Type::Float32
                | Type::Float64
                | Type::Address
                | Type::ByteSize
                | Type::USize
        ) {
            return None;
        }
        let value_type = self.types.value(&value.ty)?;
        let alignment = if aligned {
            self.source_layouts.layout(&value.ty)?.alignment
        } else {
            1
        };
        emit_instruction! {
            self;
            store {
                value: typed({{ value_type.llvm }}, {{ value.representation.as_str() }}),
                pointer: {{ pointer }},
                alignment: {{ alignment }},
                metadata: {{ metadata.iter().copied() }},
            };
        };
        Some(())
    }

    fn source_pointer_offset(&mut self, pointer: &str, offset: usize) -> String {
        if offset == 0 {
            return pointer.to_string();
        }
        let field = self.register();
        emit_instruction! {
            self;
            let {{ field.clone() }} = get_element_ptr {
                inbounds: false,
                element_type: (int(8_u16)),
                pointer: {{ pointer }},
                indices: [typed({{ self.types.index_llvm_type() }}, {{ offset.to_string() }})],
            };
        };
        field
    }
}
