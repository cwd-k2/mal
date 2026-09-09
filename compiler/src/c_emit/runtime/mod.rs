use super::body::RuntimeNeeds;
use super::syntax::{BinaryOperator, TranslationUnit};

mod core;
pub(super) mod memory;
mod numeric;
mod symbol;

use self::numeric::{
    emit_float_to_integer, emit_integer_binary, emit_integer_shift, emit_integer_wrap,
};

pub(super) fn emit(needs: &RuntimeNeeds) -> TranslationUnit {
    let mut output = core::emit(needs.memory_load_symbol, needs.control_arenas);
    if needs.wrap != 0 {
        output.extend(emit_integer_wrap(needs.wrap));
    }
    if needs.divide != 0 {
        output.extend(emit_integer_binary(
            needs.divide,
            "divide",
            BinaryOperator::Divide,
        ));
    }
    if needs.remainder != 0 {
        output.extend(emit_integer_binary(
            needs.remainder,
            "remainder",
            BinaryOperator::Remainder,
        ));
    }
    if needs.shift_left != 0 || needs.shift_right != 0 {
        output.extend(emit_integer_shift(needs.shift_left, needs.shift_right));
    }
    if needs.symbol_at {
        output.extend(symbol::emit_traversal());
    }
    if needs.symbol_equality {
        output.extend(symbol::emit_leaf_cursor());
        output.extend(symbol::emit_equality());
    }
    if needs.symbol_at {
        output.extend(symbol::emit_at());
    }
    if needs.symbol_concatenate
        || needs.symbol_concatenate_consuming_left
        || needs.symbol_concatenate_consuming_right
    {
        output.extend(symbol::emit_rope_support());
    }
    if needs.symbol_concatenate {
        output.push(symbol::emit_concatenate(false, false));
        output.blank_line();
    }
    if needs.symbol_concatenate_consuming_left {
        output.push(symbol::emit_concatenate(true, false));
        output.blank_line();
    }
    if needs.symbol_concatenate_consuming_right {
        output.push(symbol::emit_concatenate(false, true));
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
