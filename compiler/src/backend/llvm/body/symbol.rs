use crate::check::ast::Type;
use crate::closure::ast::{Atom, AtomKind, Reference};
use crate::control::ast::{Operation, Terminator};

use super::{EmittedValue, FunctionEmitter};

pub(super) fn literal_definition(name: &str, bytes: &[u8]) -> String {
    let contents = bytes
        .iter()
        .map(|byte| match byte {
            0x20..=0x21 | 0x23..=0x5b | 0x5d..=0x7e => (*byte as char).to_string(),
            _ => format!("\\{byte:02X}"),
        })
        .collect::<String>();
    format!(
        "@{name} = private constant {{ i64, i64, i8, i8, [6 x i8], [{} x i8] }} {{ i64 -1, i64 {}, i8 0, i8 0, [6 x i8] zeroinitializer, [{} x i8] c\"{contents}\" }}, align 8\n",
        bytes.len(),
        bytes.len(),
        bytes.len()
    )
}

pub(super) fn program_uses_runtime(execution: &crate::execution::Program) -> bool {
    execution
        .lowered
        .interface
        .externals
        .iter()
        .any(|external| {
            type_contains_value(&external.parameter) || type_contains_value(&external.result)
        })
        || execution.control.functions.iter().any(|function| {
            function
                .environment
                .iter()
                .any(|field| type_contains_value(&field.ty))
                || type_contains_value(&function.parameter.ty)
        })
        || execution.control.states.iter().any(|state| {
            state.input.as_ref().is_some_and(pattern_contains_value)
                || state
                    .live
                    .iter()
                    .any(|value| type_contains_value(&value.ty))
                || state.bindings.iter().any(|binding| {
                    pattern_contains_value(&binding.pattern)
                        || operation_uses_runtime(&binding.operation)
                })
                || terminator_uses_runtime(&state.terminator)
        })
}

fn type_contains_value(ty: &Type) -> bool {
    match ty {
        Type::Symbol => true,
        Type::Product(elements) | Type::Sum(elements) => elements.iter().any(type_contains_value),
        Type::Unit
        | Type::Int8
        | Type::Int16
        | Type::Int32
        | Type::Int64
        | Type::UInt8
        | Type::UInt16
        | Type::UInt32
        | Type::UInt64
        | Type::Float32
        | Type::Float64
        | Type::Ptr
        | Type::External { .. }
        | Type::Function { .. } => false,
    }
}

fn atom_contains_value(atom: &Atom) -> bool {
    type_contains_value(&atom.ty)
}

fn pattern_contains_value(pattern: &crate::closure::ast::Pattern) -> bool {
    match pattern {
        crate::closure::ast::Pattern::Binding { ty, .. }
        | crate::closure::ast::Pattern::Wildcard { ty, .. }
        | crate::closure::ast::Pattern::Product { ty, .. } => type_contains_value(ty),
    }
}

fn operation_uses_runtime(operation: &Operation) -> bool {
    match operation {
        Operation::Atom(atom)
        | Operation::SymbolLength { value: atom }
        | Operation::NumericConversion { operand: atom }
        | Operation::SumInjection { value: atom, .. }
        | Operation::ExternalCall { argument: atom, .. } => atom_contains_value(atom),
        Operation::MakeClosure { captures, .. } | Operation::Product(captures) => {
            captures.iter().any(atom_contains_value)
        }
        Operation::SymbolAt { .. } => true,
        Operation::Memory {
            primitive:
                crate::check::ast::MemoryPrimitive::LoadSymbol
                | crate::check::ast::MemoryPrimitive::StoreSymbol,
            ..
        } => true,
        Operation::Memory { argument, .. } => atom_contains_value(argument),
        Operation::PrimitiveUnary { operand, .. } => atom_contains_value(operand),
        Operation::PrimitiveBinary { left, right, .. } => {
            atom_contains_value(left) || atom_contains_value(right)
        }
    }
}

fn terminator_uses_runtime(terminator: &Terminator) -> bool {
    match terminator {
        Terminator::Return(atom)
        | Terminator::Case {
            scrutinee: atom, ..
        } => atom_contains_value(atom),
        Terminator::Goto(_) => false,
        Terminator::Jump { value, .. } => atom_contains_value(value),
        Terminator::Call {
            callee, argument, ..
        }
        | Terminator::TailCall { callee, argument } => {
            atom_contains_value(callee) || atom_contains_value(argument)
        }
        Terminator::PrimitiveBranch { left, right, .. } => {
            atom_contains_value(left) || atom_contains_value(right)
        }
    }
}

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
        mode: super::super::optimization::SymbolConcatMode,
    ) -> Option<EmittedValue> {
        use super::super::optimization::SymbolConcatMode;

        let consume_left = mode == SymbolConcatMode::ConsumeLeft && self.atom_has_slot(left);
        let consume_right = mode == SymbolConcatMode::ConsumeRight && self.atom_has_slot(right);
        let left = if consume_left {
            self.take_symbol(left)?
        } else {
            self.atom(left)?
        };
        let right = if consume_right {
            self.take_symbol(right)?
        } else {
            self.atom(right)?
        };
        if left.ty != Type::Symbol || right.ty != Type::Symbol {
            return None;
        }
        let result = self.register();
        let operation = if consume_left {
            "mal_runtime_symbol_concatenate_consuming_left"
        } else if consume_right {
            "mal_runtime_symbol_concatenate_consuming_right"
        } else {
            "mal_runtime_symbol_concatenate"
        };
        self.line(format!(
            "  {result} = call ptr @{operation}(ptr %mal_context, ptr {}, ptr {})",
            left.representation, right.representation
        ));
        Some(EmittedValue {
            ty: Type::Symbol,
            representation: result,
            owned: true,
        })
    }

    fn atom_has_slot(&self, atom: &Atom) -> bool {
        matches!(atom.kind, AtomKind::Reference(Reference::Binding(id)) if self.slots.contains_key(&id))
    }

    fn take_symbol(&mut self, atom: &Atom) -> Option<EmittedValue> {
        let AtomKind::Reference(Reference::Binding(id)) = atom.kind else {
            return None;
        };
        let slot = self.slots.get(&id)?;
        if slot.ty != Type::Symbol {
            return None;
        }
        let slot_index = slot.index;
        let value = self.atom(atom)?;
        self.line(format!(
            "  store ptr null, ptr %mal_slot_{slot_index}, align {}",
            self.types.pointer_size()
        ));
        Some(EmittedValue {
            owned: true,
            ..value
        })
    }
}
