use super::body::RuntimeNeeds;
use super::syntax::RawTranslationUnit;

pub(super) mod memory;
mod numeric;

use self::numeric::{
    emit_float_to_integer, emit_integer_checked, emit_integer_shift, emit_integer_wrap,
};

pub(super) fn emit(needs: &RuntimeNeeds) -> String {
    let mut output = String::from(RawTranslationUnit::new(RUNTIME_CORE).render());
    if needs.wrap != 0 {
        output.push_str(&emit_integer_wrap(needs.wrap));
    }
    if needs.divide != 0 {
        output.push_str(&emit_integer_checked(
            needs.divide,
            "divide",
            "/",
            "division by zero",
        ));
    }
    if needs.remainder != 0 {
        output.push_str(&emit_integer_checked(
            needs.remainder,
            "remainder",
            "%",
            "remainder by zero",
        ));
    }
    if needs.shift_left != 0 || needs.shift_right != 0 {
        output.push_str(&emit_integer_shift(needs.shift_left, needs.shift_right));
    }
    if needs.symbol_equality {
        output.push_str(RawTranslationUnit::new(RUNTIME_SYMBOL_EQUALITY).render());
    }
    if needs.symbol_at {
        output.push_str(RawTranslationUnit::new(RUNTIME_SYMBOL_AT).render());
    }
    if needs.symbol_concatenate {
        output.push_str(RawTranslationUnit::new(RUNTIME_SYMBOL_CONCATENATE).render());
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
        output.push_str(&memory::emit(
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
        output.push_str(&emit_float_to_integer(needs.float_to_integer));
    }
    output
}

const RUNTIME_CORE: &str = include_str!("runtime/core.c");
const RUNTIME_SYMBOL_EQUALITY: &str = include_str!("runtime/symbol_equal.c");
const RUNTIME_SYMBOL_AT: &str = include_str!("runtime/symbol_at.c");
const RUNTIME_SYMBOL_CONCATENATE: &str = include_str!("runtime/symbol_concatenate.c");
