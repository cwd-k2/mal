static inline MalPtr mal_ptr_offset(MalContext *context, MalPtr pointer, uint64_t offset) {
    if (offset > SIZE_MAX) {
        mal_trap(context, "pointer offset is not representable on this target");
    }
    return (MalPtr){ pointer.address + (size_t)offset };
}

#define MAL_DEFINE_MEMORY_ACCESS(name, type)                                 \
    static inline type mal_load_##name(MalPtr pointer) {                    \
        type value;                                                          \
        memcpy(&value, pointer.address, sizeof(value));                      \
        return value;                                                        \
    }                                                                        \
    static inline MalUnit mal_store_##name(MalPtr pointer, type value) {     \
        memcpy(pointer.address, &value, sizeof(value));                      \
        return (MalUnit){ UINT8_C(0) };                                      \
    }

MAL_DEFINE_MEMORY_ACCESS(int8, int8_t)
MAL_DEFINE_MEMORY_ACCESS(int16, int16_t)
MAL_DEFINE_MEMORY_ACCESS(int32, int32_t)
MAL_DEFINE_MEMORY_ACCESS(int64, int64_t)
MAL_DEFINE_MEMORY_ACCESS(uint8, uint8_t)
MAL_DEFINE_MEMORY_ACCESS(uint16, uint16_t)
MAL_DEFINE_MEMORY_ACCESS(uint32, uint32_t)
MAL_DEFINE_MEMORY_ACCESS(uint64, uint64_t)
MAL_DEFINE_MEMORY_ACCESS(float32, float)
MAL_DEFINE_MEMORY_ACCESS(float64, double)

#undef MAL_DEFINE_MEMORY_ACCESS
