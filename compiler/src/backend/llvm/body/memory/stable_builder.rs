use crate::check::ast::Type;
use crate::core::ast::PackedBuilderOperation;

use super::super::{EmittedValue, FunctionEmitter};

impl FunctionEmitter<'_> {
    pub(in crate::backend::llvm::body) fn emit_stable_packed_access(
        &mut self,
        access: &crate::backend::llvm::optimization::StablePackedAccess,
        builder: &str,
        argument: &EmittedValue,
    ) -> Option<EmittedValue> {
        let slot = self.register();
        self.line(format!(
            "  {slot} = call ptr @mal_runtime_packed_builder_data_slot(ptr {builder})"
        ));
        self.line(format!(
            "  call ptr @llvm.invariant.start.p0(i64 {}, ptr {slot})",
            self.types.pointer_size()
        ));
        let data = self.register();
        self.line(format!(
            "  {data} = load ptr, ptr {slot}, align {}",
            self.types.pointer_alignment()
        ));
        let stride = self.source_layouts.layout(&access.element)?.stride;
        match access.operation {
            PackedBuilderOperation::Get if argument.ty == Type::USize => {
                if access.element == Type::Unit {
                    return Some(unit());
                }
                let pointer = self.element_pointer(&data, argument, stride)?;
                self.emit_source_load_at(&pointer, &access.element)
            }
            PackedBuilderOperation::PutUnique => {
                let parameter = Type::Product(vec![Type::USize, access.element.clone()].into());
                if argument.ty != parameter {
                    return None;
                }
                let [index, value] =
                    self.product_fields(argument, [&Type::USize, &access.element])?;
                if stride != 0 {
                    let pointer = self.element_pointer(&data, &index, stride)?;
                    self.emit_source_store_at(&pointer, &value)?;
                }
                Some(unit())
            }
            _ => None,
        }
    }

    fn element_pointer(
        &mut self,
        data: &str,
        index: &EmittedValue,
        stride: usize,
    ) -> Option<String> {
        if index.ty != Type::USize {
            return None;
        }
        let offset = self.register();
        self.line(format!(
            "  {offset} = mul {} {}, {stride}",
            self.types.pointer_integer()?,
            index.representation
        ));
        let pointer = self.register();
        self.line(format!(
            "  {pointer} = getelementptr i8, ptr {data}, {} {offset}",
            self.types.pointer_integer()?
        ));
        Some(pointer)
    }
}

fn unit() -> EmittedValue {
    EmittedValue {
        ty: Type::Unit,
        representation: "0".into(),
        owned: false,
    }
}
