static inline MalPtr mal_ptr_offset(MalContext *context, MalPtr pointer, uint64_t offset) {
    if (offset > SIZE_MAX) {
        mal_trap(context, "pointer offset is not representable on this target");
    }
    return (MalPtr){ pointer.address + (size_t)offset };
}

static inline int64_t mal_load_int64(MalPtr pointer) {
    int64_t value;
    memcpy(&value, pointer.address, sizeof(value));
    return value;
}

static inline MalUnit mal_store_int64(MalPtr pointer, int64_t value) {
    memcpy(pointer.address, &value, sizeof(value));
    return (MalUnit){ UINT8_C(0) };
}

static inline uint8_t mal_load_uint8(MalPtr pointer) {
    uint8_t value;
    memcpy(&value, pointer.address, sizeof(value));
    return value;
}

static inline MalUnit mal_store_uint8(MalPtr pointer, uint8_t value) {
    memcpy(pointer.address, &value, sizeof(value));
    return (MalUnit){ UINT8_C(0) };
}
