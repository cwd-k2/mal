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
        "@{name} = private constant {{ i64, i64, i8, [7 x i8], [{} x i8] }} {{ i64 -1, i64 {}, i8 0, [7 x i8] zeroinitializer, [{} x i8] c\"{contents}\" }}, align 8\n",
        bytes.len(),
        bytes.len(),
        bytes.len()
    )
}

pub(super) fn program_uses_byte_runtime(execution: &crate::execution::Program) -> bool {
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
    ty.data_subtypes()
        .any(|ty| matches!(ty, Type::Symbol | Type::Packed(_)))
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
                crate::check::ast::MemoryPrimitive::AdmitRegion
                | crate::check::ast::MemoryPrimitive::Prefix
                | crate::check::ast::MemoryPrimitive::RemainderView
                | crate::check::ast::MemoryPrimitive::PackedToSymbol
                | crate::check::ast::MemoryPrimitive::SymbolToPacked,
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
        let (_, _, length) = self.byte_view_fields(&value)?;
        Some(EmittedValue {
            ty: Type::USize,
            representation: length,
            owned: false,
        })
    }

    pub(super) fn emit_symbol_at(&mut self, argument: &Atom) -> Option<EmittedValue> {
        let argument = self.atom(argument)?;
        let [symbol, index] = self.product_fields(&argument, [&Type::Symbol, &Type::USize])?;
        let (owner, offset, _) = self.byte_view_fields(&symbol)?;
        let result = self.register();
        self.line(format!(
            "  {result} = call i8 @mal_runtime_symbol_at(ptr {owner}, {} {offset}, {} {})",
            self.types.pointer_integer()?,
            self.types.pointer_integer()?,
            index.representation
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
        let (left_owner, left_offset, left_length) = self.byte_view_fields(&left)?;
        let (right_owner, right_offset, right_length) = self.byte_view_fields(&right)?;
        let result_type = self.types.value(&Type::Symbol)?;
        let result_storage = self.register();
        self.line(format!(
            "  {result_storage} = alloca {}, align {}",
            result_type.llvm, result_type.alignment
        ));
        let operation = if consume_left {
            "mal_runtime_symbol_concatenate_consuming_left"
        } else if consume_right {
            "mal_runtime_symbol_concatenate_consuming_right"
        } else {
            "mal_runtime_symbol_concatenate"
        };
        self.line(format!(
            "  call void @{operation}(ptr %mal_context, ptr {result_storage}, ptr {left_owner}, {0} {left_offset}, {0} {left_length}, ptr {right_owner}, {0} {right_offset}, {0} {right_length})",
            self.types.pointer_integer()?
        ));
        let result = self.register();
        self.line(format!(
            "  {result} = load {}, ptr {result_storage}, align {}",
            result_type.llvm, result_type.alignment
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
        let value_type = self.types.value(&Type::Symbol)?;
        self.line(format!(
            "  store {} zeroinitializer, ptr %mal_slot_{slot_index}, align {}",
            value_type.llvm, value_type.alignment
        ));
        Some(EmittedValue {
            owned: true,
            ..value
        })
    }
}
