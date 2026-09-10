use crate::check::ast::{MemoryPrimitive, Type};
use crate::closure::ast::Atom;

use super::{EmittedValue, FunctionEmitter};

impl FunctionEmitter<'_> {
    pub(super) fn emit_memory(
        &mut self,
        primitive: MemoryPrimitive,
        argument: &Atom,
        result_type: &Type,
    ) -> Option<EmittedValue> {
        let (parameter_type, expected_result) = primitive.signature();
        if argument.ty != parameter_type || *result_type != expected_result {
            return None;
        }
        let argument = self.atom(argument)?;
        match primitive {
            MemoryPrimitive::OffsetForward | MemoryPrimitive::OffsetBackward => {
                let [pointer, offset] =
                    self.product_fields(&argument, [&Type::Ptr, &Type::UInt64])?;
                let offset = if primitive == MemoryPrimitive::OffsetBackward {
                    let negated = self.register();
                    self.line(format!(
                        "  {negated} = sub i64 0, {}",
                        offset.representation
                    ));
                    negated
                } else {
                    offset.representation
                };
                let result = self.register();
                self.line(format!(
                    "  {result} = getelementptr i8, ptr {}, i64 {offset}",
                    pointer.representation
                ));
                Some(EmittedValue {
                    ty: Type::Ptr,
                    representation: result,
                    owned: false,
                })
            }
            MemoryPrimitive::Load(scalar) => self.emit_load(&argument, scalar.ty()),
            MemoryPrimitive::LoadPtr => self.emit_load(&argument, Type::Ptr),
            MemoryPrimitive::Store(scalar) => self.emit_store(&argument, &scalar.ty()),
            MemoryPrimitive::StorePtr => self.emit_store(&argument, &Type::Ptr),
            MemoryPrimitive::LoadSymbol => {
                let [pointer, length] =
                    self.product_fields(&argument, [&Type::Ptr, &Type::UInt64])?;
                let result = self.register();
                self.line(format!(
                    "  {result} = call ptr @mal_runtime_symbol_read(ptr %mal_context, ptr {}, i64 {})",
                    pointer.representation, length.representation
                ));
                Some(EmittedValue {
                    ty: Type::Symbol,
                    representation: result,
                    owned: true,
                })
            }
            MemoryPrimitive::StoreSymbol => {
                let [pointer, symbol] =
                    self.product_fields(&argument, [&Type::Ptr, &Type::Symbol])?;
                self.line(format!(
                    "  call void @mal_runtime_symbol_write(ptr {}, ptr {})",
                    pointer.representation, symbol.representation
                ));
                Some(EmittedValue {
                    ty: Type::Unit,
                    representation: "0".into(),
                    owned: false,
                })
            }
        }
    }

    fn emit_load(&mut self, pointer: &EmittedValue, ty: Type) -> Option<EmittedValue> {
        if pointer.ty != Type::Ptr {
            return None;
        }
        let value_type = self.types.value(&ty)?;
        let result = self.register();
        self.line(format!(
            "  {result} = load {}, ptr {}, align 1",
            value_type.llvm, pointer.representation
        ));
        Some(EmittedValue {
            ty,
            representation: result,
            owned: false,
        })
    }

    fn emit_store(&mut self, argument: &EmittedValue, value_type: &Type) -> Option<EmittedValue> {
        let [pointer, value] = self.product_fields(argument, [&Type::Ptr, value_type])?;
        let representation = self.types.value(value_type)?;
        self.line(format!(
            "  store {} {}, ptr {}, align 1",
            representation.llvm, value.representation, pointer.representation
        ));
        Some(EmittedValue {
            ty: Type::Unit,
            representation: "0".into(),
            owned: false,
        })
    }

    pub(super) fn product_fields<const N: usize>(
        &mut self,
        product: &EmittedValue,
        expected: [&Type; N],
    ) -> Option<[EmittedValue; N]> {
        let Type::Product(elements) = &product.ty else {
            return None;
        };
        if elements.iter().collect::<Vec<_>>() != expected {
            return None;
        }
        let product_type = self.types.value(&product.ty)?;
        let mut fields = Vec::with_capacity(N);
        for (index, ty) in elements.iter().enumerate() {
            let field = self.register();
            self.line(format!(
                "  {field} = extractvalue {} {}, {index}",
                product_type.llvm, product.representation
            ));
            fields.push(EmittedValue {
                ty: ty.clone(),
                representation: field,
                owned: false,
            });
        }
        fields.try_into().ok()
    }
}
