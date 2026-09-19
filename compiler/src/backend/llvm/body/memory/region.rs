use crate::check::ast::Type;

use super::super::{EmittedValue, FunctionEmitter};

impl FunctionEmitter<'_> {
    pub(in crate::backend::llvm::body) fn emit_form_region(
        &mut self,
        argument: &EmittedValue,
        result_type: &Type,
    ) -> Option<EmittedValue> {
        let Type::Region(element) = result_type else {
            return None;
        };
        let [address, start, end] =
            self.product_fields(argument, [&Type::Address, &Type::USize, &Type::USize])?;
        let stride = self.source_layouts.layout(element)?.stride;
        let start_bytes = self.multiply_by_stride(&start.representation, stride)?;
        let first = if stride == 0 {
            "null".to_owned()
        } else {
            self.pointer_offset(&address.representation, &start_bytes)?
        };
        let count = self.register();
        self.line(format!(
            "  {count} = sub {} {}, {}",
            self.types.pointer_integer()?,
            end.representation,
            start.representation
        ));
        self.make_region(result_type, &first, &count)
    }

    pub(in crate::backend::llvm::body) fn emit_region_get(
        &mut self,
        argument: &EmittedValue,
        result_type: &Type,
    ) -> Option<EmittedValue> {
        let region_type = Type::Region(result_type.clone().into());
        let [region, index] = self.product_fields(argument, [&region_type, &Type::USize])?;
        let (address, _) = self.region_fields(&region)?;
        let pointer = self.region_element_pointer(&address, &index, result_type)?;
        self.emit_source_load_at(&pointer, result_type)
    }

    pub(in crate::backend::llvm::body) fn emit_region_put(
        &mut self,
        argument: &EmittedValue,
        result_type: &Type,
    ) -> Option<EmittedValue> {
        if *result_type != Type::Unit {
            return None;
        }
        let Type::Product(fields) = &argument.ty else {
            return None;
        };
        let [region_type, index_type, element] = fields.as_ref() else {
            return None;
        };
        let Type::Region(region_element) = region_type else {
            return None;
        };
        if index_type != &Type::USize || region_element.as_ref() != element {
            return None;
        }
        let [region, index, value] =
            self.product_fields(argument, [region_type, &Type::USize, element])?;
        let (address, _) = self.region_fields(&region)?;
        let pointer = self.region_element_pointer(&address, &index, element)?;
        self.emit_source_store_at(&pointer, &value)?;
        Some(EmittedValue {
            ty: Type::Unit,
            representation: "0".into(),
            owned: false,
        })
    }

    fn region_element_pointer(
        &mut self,
        address: &str,
        index: &EmittedValue,
        element: &Type,
    ) -> Option<String> {
        let stride = self.source_layouts.layout(element)?.stride;
        if stride == 0 {
            return Some("null".into());
        }
        let offset = self.multiply_by_stride(&index.representation, stride)?;
        self.pointer_offset(address, &offset)
    }
}
