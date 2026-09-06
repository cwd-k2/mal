static MalType_Engram mal_engram_concatenate(
    MalContext *context,
    MalType_Engram left,
    MalType_Engram right
) {
    if (left.length == UINT64_C(0)) {
        return right;
    }
    if (right.length == UINT64_C(0)) {
        return left;
    }
    if (left.length > UINT64_MAX - right.length) {
        mal_trap(context, "Engram length overflow");
    }
    uint64_t length = left.length + right.length;
    size_t size = (size_t)length;
    if ((uint64_t)size != length) {
        mal_trap(context, "allocation size overflow");
    }
    uint8_t *bytes = (uint8_t *)mal_allocate(context, size);
    memcpy(bytes, left.data, (size_t)left.length);
    memcpy(bytes + (size_t)left.length, right.data, (size_t)right.length);
    return (MalType_Engram){ bytes, length };
}
