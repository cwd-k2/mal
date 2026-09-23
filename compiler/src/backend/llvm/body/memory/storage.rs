use crate::check::ast::Type;

use super::super::{EmittedValue, FunctionEmitter};

impl FunctionEmitter<'_> {
    pub(in crate::backend::llvm::body) fn emit_aligned_buffer_load_at(
        &mut self,
        pointer: &str,
        element: &Type,
    ) -> Option<EmittedValue> {
        self.emit_source_load_at_with_alignment(pointer, element, true, ", !tbaa !3")
    }

    fn emit_source_load_at_with_alignment(
        &mut self,
        pointer: &str,
        element: &Type,
        aligned: bool,
        metadata: &str,
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
                let field_pointer = self.source_pointer_offset(pointer, field.offset)?;
                let field_value = self.emit_source_load_at_with_alignment(
                    &field_pointer,
                    field_type,
                    aligned,
                    metadata,
                )?;
                let llvm_type = self.types.value(field_type)?;
                let inserted = self.register();
                self.line(format!(
                    "  {inserted} = insertvalue {} {product}, {} {}, {index}",
                    product_type.llvm, llvm_type.llvm, field_value.representation
                ));
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
            self.line(format!(
                "  {source_tag} = load i{}, ptr {pointer}, align {alignment}{metadata}",
                layout.tag_bits,
            ));
            if super::super::types::is_bool(element) {
                let value = self.register();
                self.line(format!("  {value} = trunc i8 {source_tag} to i1"));
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
                self.line(format!(
                    "  {extended} = zext i{} {source_tag} to i32",
                    layout.tag_bits
                ));
                extended
            } else {
                let narrowed = self.register();
                self.line(format!("  {narrowed} = trunc i64 {source_tag} to i32"));
                narrowed
            };
            let stem = self.register();
            let stem = stem.trim_start_matches('%').to_string();
            let runtime = self.types.value(element)?;
            let storage = self.entry_alloca(&runtime.llvm, runtime.alignment);
            let payload_pointer = self.source_pointer_offset(pointer, layout.payload_offset)?;
            let cases = variants
                .iter()
                .enumerate()
                .map(|(index, _)| format!("    i32 {index}, label %{stem}_variant_{index}"))
                .collect::<Vec<_>>()
                .join("\n");
            self.line(format!(
                "  switch i32 {tag}, label %{stem}_invalid [\n{cases}\n  ]"
            ));
            self.line(format!("{stem}_invalid:"));
            self.line("  unreachable");
            for (index, variant) in variants.iter().enumerate() {
                self.line(format!("{stem}_variant_{index}:"));
                let payload = self.emit_source_load_at_with_alignment(
                    &payload_pointer,
                    variant,
                    aligned,
                    metadata,
                )?;
                let sum = self.emit_sum_value(index, payload, element, false)?;
                self.line(format!(
                    "  store {} {}, ptr {storage}, align {}",
                    runtime.llvm, sum.representation, runtime.alignment
                ));
                self.line(format!("  br label %{stem}_loaded"));
            }
            self.line(format!("{stem}_loaded:"));
            let result = self.register();
            self.line(format!(
                "  {result} = load {}, ptr {storage}, align {}",
                runtime.llvm, runtime.alignment
            ));
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
        self.line(format!(
            "  {value} = load {}, ptr {pointer}, align {alignment}{metadata}",
            value_type.llvm,
        ));
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
        self.emit_source_store_at_with_alignment(pointer, value, true, "")
    }

    pub(in crate::backend::llvm::body) fn emit_aligned_buffer_store_at(
        &mut self,
        pointer: &str,
        value: &EmittedValue,
    ) -> Option<()> {
        self.emit_source_store_at_with_alignment(pointer, value, true, ", !tbaa !3")
    }

    fn emit_source_store_at_with_alignment(
        &mut self,
        pointer: &str,
        value: &EmittedValue,
        aligned: bool,
        metadata: &str,
    ) -> Option<()> {
        if value.ty == Type::Unit {
            return Some(());
        }
        if let Type::Product(elements) = &value.ty {
            let fields = self.source_layouts.product_fields(&value.ty)?;
            let runtime = self.types.value(&value.ty)?;
            for (index, (field, field_type)) in fields.iter().zip(elements.iter()).enumerate() {
                let field_value = self.register();
                self.line(format!(
                    "  {field_value} = extractvalue {} {}, {index}",
                    runtime.llvm, value.representation
                ));
                let field_pointer = self.source_pointer_offset(pointer, field.offset)?;
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
                self.line(format!("  {tag} = zext i1 {} to i8", value.representation));
                self.line(format!(
                    "  store i8 {tag}, ptr {pointer}, align 1{metadata}"
                ));
                return Some(());
            }
            let runtime = self.types.value(&value.ty)?;
            let tag = self.register();
            self.line(format!(
                "  {tag} = extractvalue {} {}, 0",
                runtime.llvm, value.representation
            ));
            let source_tag = if layout.tag_bits == 32 {
                tag.clone()
            } else if layout.tag_bits < 32 {
                let narrowed = self.register();
                self.line(format!(
                    "  {narrowed} = trunc i32 {tag} to i{}",
                    layout.tag_bits
                ));
                narrowed
            } else {
                let extended = self.register();
                self.line(format!("  {extended} = zext i32 {tag} to i64"));
                extended
            };
            let alignment = if aligned {
                self.source_layouts.layout(&value.ty)?.alignment
            } else {
                1
            };
            self.line(format!(
                "  store i{} {source_tag}, ptr {pointer}, align {alignment}{metadata}",
                layout.tag_bits,
            ));
            let stem = self.register();
            let stem = stem.trim_start_matches('%').to_string();
            let payload_pointer = self.source_pointer_offset(pointer, layout.payload_offset)?;
            let cases = variants
                .iter()
                .enumerate()
                .map(|(index, _)| format!("    i32 {index}, label %{stem}_variant_{index}"))
                .collect::<Vec<_>>()
                .join("\n");
            self.line(format!(
                "  switch i32 {tag}, label %{stem}_invalid [\n{cases}\n  ]"
            ));
            self.line(format!("{stem}_invalid:"));
            self.line("  unreachable");
            for (index, variant) in variants.iter().enumerate() {
                self.line(format!("{stem}_variant_{index}:"));
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
                self.line(format!("  br label %{stem}_stored"));
            }
            self.line(format!("{stem}_stored:"));
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
        self.line(format!(
            "  store {} {}, ptr {pointer}, align {alignment}{metadata}",
            value_type.llvm, value.representation
        ));
        Some(())
    }

    fn source_pointer_offset(&mut self, pointer: &str, offset: usize) -> Option<String> {
        if offset == 0 {
            return Some(pointer.to_string());
        }
        let field = self.register();
        self.line(format!(
            "  {field} = getelementptr i8, ptr {pointer}, {} {offset}",
            self.types.pointer_integer()?
        ));
        Some(field)
    }
}
