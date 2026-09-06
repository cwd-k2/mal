static uint8_t mal_symbol_at(MalContext *context, MalType_Symbol value, uint64_t index) {
    if (index >= value.length) {
        mal_trap(context, "Symbol index out of range");
    }
    return value.data[index];
}
