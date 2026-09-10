use crate::check::ast::Type;
use crate::closure::ast::Atom;

use super::{EmittedValue, FunctionEmitter};

impl FunctionEmitter<'_> {
    pub(super) fn emit_symbol_length(&mut self, value: &Atom) -> Option<EmittedValue> {
        let value = self.atom(value)?;
        if value.ty != Type::Symbol {
            return None;
        }
        let result = self.register();
        self.line(format!(
            "  {result} = call i64 @mal_runtime_symbol_length(ptr {})",
            value.representation
        ));
        Some(EmittedValue {
            ty: Type::UInt64,
            representation: result,
            owned: false,
        })
    }

    pub(super) fn emit_symbol_at(&mut self, argument: &Atom) -> Option<EmittedValue> {
        let argument = self.atom(argument)?;
        let [symbol, index] = self.product_fields(&argument, [&Type::Symbol, &Type::UInt64])?;
        let result = self.register();
        self.line(format!(
            "  {result} = call i8 @mal_runtime_symbol_at(ptr {}, i64 {})",
            symbol.representation, index.representation
        ));
        Some(EmittedValue {
            ty: Type::UInt8,
            representation: result,
            owned: false,
        })
    }

    pub(super) fn emit_symbol_concatenate(
        &mut self,
        left: &Atom,
        right: &Atom,
    ) -> Option<EmittedValue> {
        let left = self.atom(left)?;
        let right = self.atom(right)?;
        if left.ty != Type::Symbol || right.ty != Type::Symbol {
            return None;
        }
        let result = self.register();
        self.line(format!(
            "  {result} = call ptr @mal_runtime_symbol_concatenate(ptr %mal_context, ptr {}, ptr {})",
            left.representation, right.representation
        ));
        Some(EmittedValue {
            ty: Type::Symbol,
            representation: result,
            owned: true,
        })
    }
}
