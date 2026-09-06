static uint8_t mal_engram_at(MalContext *context, MalEngram value, uint64_t index) {
    if (index >= value.length) {
        mal_trap(context, "Engram index out of range");
    }
    return value.data[index];
}
