use mal_frontend::check::ast::Type;

use super::super::{EmittedValue, FunctionEmitter};

impl FunctionEmitter<'_> {
    pub(in crate::backend::llvm::body) fn product_fields<const N: usize>(
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
        let fields = std::array::from_fn(|index| {
            let ty = &elements[index];
            let field = self.register();
            self.line(format!(
                "  {field} = extractvalue {} {}, {index}",
                product_type.llvm, product.representation
            ));
            EmittedValue {
                ty: ty.clone(),
                representation: field,
                owned: false,
            }
        });
        Some(fields)
    }
}
