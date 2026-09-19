use crate::backend::llvm::optimization::{type_contains_buffer, type_has_single_buffer};
use crate::check::ast::Type;

use super::super::{EmittedValue, FunctionEmitter};

impl FunctionEmitter<'_> {
    pub(in crate::backend::llvm::body) fn adapt_buffer_argument(
        &mut self,
        value: EmittedValue,
        target_uses_direct_buffer: bool,
    ) -> Option<EmittedValue> {
        if !type_contains_buffer(&value.ty) {
            return Some(value);
        }
        let source_uses_direct_buffer =
            self.optimizations.uses_direct_buffer(self.current_function);
        match (source_uses_direct_buffer, target_uses_direct_buffer) {
            (false, true) => {
                let representation =
                    self.direct_buffer_representation(&value.ty, &value.representation)?;
                Some(EmittedValue {
                    representation,
                    ..value
                })
            }
            (true, false) => None,
            _ => Some(value),
        }
    }

    pub(in crate::backend::llvm::body) fn call_arguments(
        &mut self,
        environment: &str,
        argument: &EmittedValue,
        target_uses_direct_buffer: bool,
    ) -> Option<String> {
        let value_type = self.types.value(&argument.ty)?;
        let mut arguments = format!(
            "ptr %mal_context, ptr %mal_control_top, ptr {environment}, {} {}",
            value_type.llvm, argument.representation
        );
        if target_uses_direct_buffer && type_has_single_buffer(&argument.ty) {
            let data = self.buffer_leaf_representation(&argument.ty, &argument.representation)?;
            arguments.push_str(&format!(", ptr {data}"));
        }
        Some(arguments)
    }

    fn direct_buffer_representation(&mut self, ty: &Type, value: &str) -> Option<String> {
        match ty {
            Type::Buffer(_) => {
                let slot = self.register();
                self.line(format!(
                    "  {slot} = call ptr @mal_runtime_packed_builder_data_slot(ptr {value})"
                ));
                let data = self.register();
                self.line(format!(
                    "  {data} = load ptr, ptr {slot}, align {}",
                    self.types.pointer_alignment()
                ));
                Some(data)
            }
            Type::Product(elements) => {
                let product_type = self.types.value(ty)?;
                let mut result = value.to_string();
                for (index, element) in elements.iter().enumerate() {
                    if !type_contains_buffer(element) {
                        continue;
                    }
                    let element_type = self.types.value(element)?;
                    let extracted = self.register();
                    self.line(format!(
                        "  {extracted} = extractvalue {} {value}, {index}",
                        product_type.llvm
                    ));
                    let converted = self.direct_buffer_representation(element, &extracted)?;
                    let inserted = self.register();
                    self.line(format!(
                        "  {inserted} = insertvalue {} {result}, {} {converted}, {index}",
                        product_type.llvm, element_type.llvm
                    ));
                    result = inserted;
                }
                Some(result)
            }
            _ => (!type_contains_buffer(ty)).then(|| value.to_string()),
        }
    }

    pub(in crate::backend::llvm::body) fn replace_buffer_leaf(
        &mut self,
        ty: &Type,
        value: &str,
        replacement: &str,
    ) -> Option<String> {
        match ty {
            Type::Buffer(_) => Some(replacement.to_string()),
            Type::Product(elements) => {
                let product_type = self.types.value(ty)?;
                let mut result = value.to_string();
                for (index, element) in elements.iter().enumerate() {
                    if !type_contains_buffer(element) {
                        continue;
                    }
                    let element_type = self.types.value(element)?;
                    let extracted = self.register();
                    self.line(format!(
                        "  {extracted} = extractvalue {} {value}, {index}",
                        product_type.llvm
                    ));
                    let converted = self.replace_buffer_leaf(element, &extracted, replacement)?;
                    let inserted = self.register();
                    self.line(format!(
                        "  {inserted} = insertvalue {} {result}, {} {converted}, {index}",
                        product_type.llvm, element_type.llvm
                    ));
                    result = inserted;
                }
                Some(result)
            }
            _ => None,
        }
    }

    fn buffer_leaf_representation(&mut self, ty: &Type, value: &str) -> Option<String> {
        match ty {
            Type::Buffer(_) => Some(value.to_string()),
            Type::Product(elements) => {
                let (index, element) = elements
                    .iter()
                    .enumerate()
                    .find(|(_, element)| type_contains_buffer(element))?;
                let product_type = self.types.value(ty)?;
                let extracted = self.register();
                self.line(format!(
                    "  {extracted} = extractvalue {} {value}, {index}",
                    product_type.llvm
                ));
                self.buffer_leaf_representation(element, &extracted)
            }
            _ => None,
        }
    }
}
