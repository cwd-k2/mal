use super::body::RuntimeNeeds;

pub(super) mod memory;
mod numeric;

use self::numeric::{
    emit_float_to_integer, emit_integer_checked, emit_integer_shift, emit_integer_wrap,
};

pub(super) fn emit(needs: &RuntimeNeeds) -> String {
    let mut output = String::from(RUNTIME_CORE);
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
    if needs.string_equality {
        output.push_str(RUNTIME_STRING_EQUALITY);
    }
    if needs.string_at {
        output.push_str(RUNTIME_STRING_AT);
    }
    if needs.memory_offset
        || needs.memory_load != 0
        || needs.memory_store != 0
        || needs.memory_load_ptr
        || needs.memory_store_ptr
    {
        output.push_str(&memory::emit(
            needs.memory_offset,
            needs.memory_load,
            needs.memory_store,
            needs.memory_load_ptr,
            needs.memory_store_ptr,
        ));
    }
    if needs.float_to_integer != 0 {
        output.push_str(&emit_float_to_integer(needs.float_to_integer));
    }
    output
}

const RUNTIME_CORE: &str = include_str!("runtime/core.c");
const RUNTIME_STRING_EQUALITY: &str = include_str!("runtime/string_equal.c");
const RUNTIME_STRING_AT: &str = include_str!("runtime/string_at.c");
