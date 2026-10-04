//! Stores of source values into their canonical memory layout.

use crate::backend::llvm::syntax::emit_instruction;
use crate::backend::llvm::syntax::emit_terminator;
use mal_frontend::check::ast::Type;

use super::super::super::{EmittedValue, FunctionEmitter};
use crate::backend::llvm::syntax::{CastOperator, MetadataAttachment, llvm_type};

impl FunctionEmitter<'_> {
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
                    let { field_value.clone() } = extract_value {
                        aggregate: ({ runtime.llvm.clone() }, { value.representation.clone() }),
                        indices: [{ index }],
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
            if super::super::super::types::is_bool(&value.ty) {
                let tag = self.register();
                emit_instruction! {
                    self;
                    let { tag.clone() } = cast {
                        operator: { CastOperator::ZExt },
                        value: (int(1_u16), { value.representation.clone() }),
                        to: int(8_u16),
                    };
                };
                emit_instruction! {
                    self;
                    store {
                        value: (int(8_u16), { &tag }),
                        pointer: { pointer },
                        alignment: 1,
                        metadata: { metadata.iter().copied() },
                    };
                };
                return Some(());
            }
            let runtime = self.types.value(&value.ty)?;
            let tag = self.register();
            emit_instruction! {
                self;
                let { tag.clone() } = extract_value {
                    aggregate: ({ runtime.llvm }, { value.representation.clone() }),
                    indices: [0],
                };
            };
            let source_tag = if layout.tag_bits == 32 {
                tag.clone()
            } else if layout.tag_bits < 32 {
                let narrowed = self.register();
                emit_instruction! {
                    self;
                    let { narrowed.clone() } = cast {
                        operator: { CastOperator::Trunc },
                        value: (int(32_u16), { &tag }),
                        to: { llvm_type!(int({ u16::try_from(layout.tag_bits).ok()? })) },
                    };
                };
                narrowed
            } else {
                let extended = self.register();
                emit_instruction! {
                    self;
                    let { extended.clone() } = cast {
                        operator: { CastOperator::ZExt },
                        value: (int(32_u16), { &tag }),
                        to: int(64_u16),
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
                    value: ({ llvm_type!(int({ u16::try_from(layout.tag_bits).ok()? })) }, { source_tag }),
                    pointer: { pointer },
                    alignment: { alignment },
                    metadata: { metadata.iter().copied() },
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
                switch (int(32_u16), { tag })  {
                    cases: [..{ cases }],
                    default: { format!("{stem}_invalid") },
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
                        target: { format!("{stem}_stored") },
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
                value: ({ value_type.llvm }, { value.representation.as_str() }),
                pointer: { pointer },
                alignment: { alignment },
                metadata: { metadata.iter().copied() },
            };
        };
        Some(())
    }
}
