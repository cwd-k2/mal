static uint8_t mal_string_at(MalContext *context, MalString value, uint64_t index) {
    if (index >= value.length) {
        mal_trap(context, "string index out of range");
    }
    return value.data[index];
}

