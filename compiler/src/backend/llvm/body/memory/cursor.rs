use crate::check::ast::Type;

use super::super::{EmittedValue, FunctionEmitter};

impl FunctionEmitter<'_> {
    pub(in crate::backend::llvm::body) fn emit_align(
        &mut self,
        value: EmittedValue,
        result_type: &Type,
    ) -> Option<EmittedValue> {
        if value.ty != *result_type {
            return None;
        }
        let element = match &value.ty {
            Type::Cursor(element) | Type::Region(element) => element,
            _ => return None,
        };
        let alignment = self.source_layouts.layout(element)?.alignment;
        if alignment == 1 {
            return Some(value);
        }
        let (address, count) = match &value.ty {
            Type::Cursor(_) => (value.representation.clone(), None),
            Type::Region(_) => {
                let runtime = self.types.value(&value.ty)?;
                let address = self.register();
                self.line(format!(
                    "  {address} = extractvalue {} {}, 0",
                    runtime.llvm, value.representation
                ));
                let count = self.register();
                self.line(format!(
                    "  {count} = extractvalue {} {}, 1",
                    runtime.llvm, value.representation
                ));
                (address, Some(count))
            }
            _ => return None,
        };
        let advanced = self.register();
        self.line(format!(
            "  {advanced} = getelementptr i8, ptr {address}, {} {}",
            self.types.pointer_integer()?,
            alignment - 1
        ));
        let aligned = self.register();
        let bits = self.types.pointer_size().checked_mul(8)?;
        self.line(format!(
            "  {aligned} = call ptr @llvm.ptrmask.p0.i{bits}(ptr {advanced}, i{bits} -{alignment})"
        ));
        let Some(count) = count else {
            return Some(EmittedValue {
                ty: value.ty,
                representation: aligned,
                owned: false,
            });
        };
        let runtime = self.types.value(&value.ty)?;
        let with_address = self.register();
        self.line(format!(
            "  {with_address} = insertvalue {} poison, ptr {aligned}, 0",
            runtime.llvm
        ));
        let result = self.register();
        self.line(format!(
            "  {result} = insertvalue {} {with_address}, {} {count}, 1",
            runtime.llvm,
            self.types.pointer_integer()?
        ));
        Some(EmittedValue {
            ty: value.ty,
            representation: result,
            owned: false,
        })
    }

    pub(in crate::backend::llvm::body) fn emit_cursor_load(
        &mut self,
        cursor: &EmittedValue,
        result_type: &Type,
    ) -> Option<EmittedValue> {
        let Type::Cursor(element) = &cursor.ty else {
            return None;
        };
        let expected = Type::Product(vec![(**element).clone(), cursor.ty.clone()].into());
        if *result_type != expected {
            return None;
        }
        let value = self.emit_source_load(cursor, element)?;
        let next = self.offset_cursor(cursor, element)?;
        let product_type = self.types.value(result_type)?;
        let value_type = self.types.value(element)?;
        let with_value = self.register();
        self.line(format!(
            "  {with_value} = insertvalue {} poison, {} {}, 0",
            product_type.llvm, value_type.llvm, value.representation
        ));
        let result = self.register();
        self.line(format!(
            "  {result} = insertvalue {} {with_value}, ptr {}, 1",
            product_type.llvm, next.representation
        ));
        Some(EmittedValue {
            ty: result_type.clone(),
            representation: result,
            owned: false,
        })
    }

    pub(in crate::backend::llvm::body) fn emit_cursor_store(
        &mut self,
        argument: &EmittedValue,
        result_type: &Type,
    ) -> Option<EmittedValue> {
        let Type::Product(fields) = &argument.ty else {
            return None;
        };
        let [cursor_type, element] = fields.as_ref() else {
            return None;
        };
        let Type::Cursor(cursor_element) = cursor_type else {
            return None;
        };
        if cursor_element.as_ref() != element || *result_type != *cursor_type {
            return None;
        }
        let [cursor, value] = self.product_fields(argument, [cursor_type, element])?;
        self.emit_source_store(&cursor, &value)?;
        self.offset_cursor(&cursor, element)
    }

    fn offset_cursor(&mut self, cursor: &EmittedValue, element: &Type) -> Option<EmittedValue> {
        let stride = self.source_layouts.layout(element)?.stride;
        if stride == 0 {
            return Some(cursor.clone());
        }
        let next = self.register();
        self.line(format!(
            "  {next} = getelementptr i8, ptr {}, {} {stride}",
            cursor.representation,
            self.types.pointer_integer()?
        ));
        Some(EmittedValue {
            ty: cursor.ty.clone(),
            representation: next,
            owned: false,
        })
    }
}
