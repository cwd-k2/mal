static uint8_t mal_engram_equal(MalType_Engram left, MalType_Engram right) {
    if (left.length != right.length) {
        return UINT8_C(0);
    }
    for (uint64_t index = UINT64_C(0); index < left.length; index += UINT64_C(1)) {
        if (left.data[index] != right.data[index]) {
            return UINT8_C(0);
        }
    }
    return UINT8_C(1);
}
