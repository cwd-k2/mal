use super::body::RuntimeNeeds;
use super::syntax::{BinaryOperator, TranslationUnit};

mod core;
pub(super) mod memory;
mod numeric;
mod symbol;

use self::numeric::{
    emit_float_to_integer, emit_integer_checked, emit_integer_shift, emit_integer_wrap,
};

pub(super) fn emit(needs: &RuntimeNeeds) -> TranslationUnit {
    let mut output = core::emit();
    if needs.wrap != 0 {
        output.extend(emit_integer_wrap(needs.wrap));
    }
    if needs.divide != 0 {
        output.extend(emit_integer_checked(
            needs.divide,
            "divide",
            BinaryOperator::Divide,
            "division by zero",
        ));
    }
    if needs.remainder != 0 {
        output.extend(emit_integer_checked(
            needs.remainder,
            "remainder",
            BinaryOperator::Remainder,
            "remainder by zero",
        ));
    }
    if needs.shift_left != 0 || needs.shift_right != 0 {
        output.extend(emit_integer_shift(needs.shift_left, needs.shift_right));
    }
    if needs.symbol_equality {
        output.push(symbol::emit_equality());
        output.blank_line();
    }
    if needs.symbol_at {
        output.push(symbol::emit_at());
        output.blank_line();
    }
    if needs.symbol_concatenate {
        output.push(symbol::emit_concatenate());
        output.blank_line();
    }
    if needs.memory_offset_forward
        || needs.memory_offset_backward
        || needs.memory_load != 0
        || needs.memory_store != 0
        || needs.memory_load_ptr
        || needs.memory_store_ptr
        || needs.memory_load_symbol
        || needs.memory_store_symbol
    {
        output.extend(memory::emit(
            (needs.memory_offset_forward, needs.memory_offset_backward),
            needs.memory_load,
            needs.memory_store,
            needs.memory_load_ptr,
            needs.memory_store_ptr,
            needs.memory_load_symbol,
            needs.memory_store_symbol,
        ));
    }
    if needs.float_to_integer != 0 {
        output.extend(emit_float_to_integer(needs.float_to_integer));
    }
    output
}
