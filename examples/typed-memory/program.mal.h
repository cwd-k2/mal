#ifndef MAL_BUILD_UMBRELLA
#include <mal.h>
#include <string.h>

_Static_assert(MAL_C_ABI_VERSION == 0x000900u, "generated header requires mal C ABI 0x000900");
_Static_assert((sizeof((size_t)0) * CHAR_BIT) == 64, "size_t does not match the mal target pointer index width");

#ifndef MAL_GENERATED_INTERFACE_52B76425F3B62564_H
#define MAL_GENERATED_INTERFACE_52B76425F3B62564_H
/* Host-visible types */

#ifndef MAL_DETAIL_HOST_REPR_1e36b8e9f3384819_DECLARED
#define MAL_DETAIL_HOST_REPR_1e36b8e9f3384819_DECLARED
typedef struct mal_detail_repr_product_1e36b8e9f3384819 mal_repr_product_1e36b8e9f3384819_t;
#endif
typedef mal_repr_product_1e36b8e9f3384819_t mal_SampleRecord_t;

#ifndef MAL_DETAIL_HOST_REPR_1e36b8e9f3384819_DEFINED
#define MAL_DETAIL_HOST_REPR_1e36b8e9f3384819_DEFINED
struct mal_detail_repr_product_1e36b8e9f3384819 {
    mal_Int64_t field_0;
    mal_UInt8_t field_1;
};
#endif

/* Canonical memory access */

#ifndef MAL_DETAIL_MEMORY_Int64_HELPERS
#define MAL_DETAIL_MEMORY_Int64_HELPERS
static inline mal_Int64_t mal_detail_memory_read_Int64(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, const uint8_t *source) {
    mal_Int64_t value;
    memcpy(&value, source, sizeof(value));
    return value;
}

static inline void mal_detail_memory_write_Int64(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, uint8_t *destination, mal_Int64_t value) {
    memcpy(destination, &value, sizeof(value));
}

#endif

#ifndef MAL_DETAIL_MEMORY_UInt8_HELPERS
#define MAL_DETAIL_MEMORY_UInt8_HELPERS
static inline mal_UInt8_t mal_detail_memory_read_UInt8(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, const uint8_t *source) {
    mal_UInt8_t value;
    memcpy(&value, source, sizeof(value));
    return value;
}

static inline void mal_detail_memory_write_UInt8(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, uint8_t *destination, mal_UInt8_t value) {
    memcpy(destination, &value, sizeof(value));
}

#endif

#ifndef MAL_DETAIL_MEMORY_REPR_1e36b8e9f3384819_HELPERS
#define MAL_DETAIL_MEMORY_REPR_1e36b8e9f3384819_HELPERS
static inline mal_repr_product_1e36b8e9f3384819_t mal_detail_memory_read_1e36b8e9f3384819(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, const uint8_t *source MAL_DETAIL_MAYBE_UNUSED) {
    mal_repr_product_1e36b8e9f3384819_t value;
    value.field_0 = mal_detail_memory_read_Int64(call, source + 0);
    value.field_1 = mal_detail_memory_read_UInt8(call, source + 8);
    return value;
}

static inline void mal_detail_memory_write_1e36b8e9f3384819(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, uint8_t *destination MAL_DETAIL_MAYBE_UNUSED, mal_repr_product_1e36b8e9f3384819_t value) {
    mal_detail_memory_write_Int64(call, destination + 0, value.field_0);
    mal_detail_memory_write_UInt8(call, destination + 8, value.field_1);
}

#endif

static inline mal_SampleRecord_t mal_SampleRecord_read(mal_call_t *call, mal_Address_t address, mal_USize_t index) {
    mal_Address_return(call, address);
    return mal_detail_memory_read_1e36b8e9f3384819(call, (const uint8_t *)address + (index * 16));
}

static inline void mal_SampleRecord_write(mal_call_t *call, mal_Address_t address, mal_USize_t index, mal_SampleRecord_t value) {
    mal_Address_return(call, address);
    mal_detail_memory_write_1e36b8e9f3384819(call, (uint8_t *)address + (index * 16), value);
}

/* External operations */

MalType_Address mal_ext_unalignedStorage(MalContext *context);
void mal_ext_incrementSample(MalContext *context, MalType_Address value);

/* External definition helpers */

#define MAL_HAS_EXTERN_unalignedStorage 1
#define MAL_DEFINE_unalignedStorage(call) \
static MalType_Address mal_detail_unalignedStorage(mal_call_t *call); \
MalType_Address mal_ext_unalignedStorage(MalContext *context MAL_DETAIL_MAYBE_UNUSED) { \
    mal_call_t call = (mal_call_t){ .mal_detail_context = context }; \
    return mal_detail_unalignedStorage(&call); \
} \
static MalType_Address mal_detail_unalignedStorage( \
    mal_call_t *call \
)

#define MAL_HAS_EXTERN_incrementSample 1
#define MAL_DEFINE_incrementSample(call, value) \
static MalType_Unit mal_detail_incrementSample(mal_call_t *call, mal_Address_t value); \
void mal_ext_incrementSample(MalContext *context MAL_DETAIL_MAYBE_UNUSED, MalType_Address value) { \
    mal_call_t call = (mal_call_t){ .mal_detail_context = context }; \
    mal_detail_incrementSample(&call, value); \
} \
static MalType_Unit mal_detail_incrementSample( \
    mal_call_t *call, \
    mal_Address_t value \
)

#endif
#endif
