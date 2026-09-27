#ifndef MAL_BUILD_UMBRELLA
#include <mal.h>
#include <string.h>

_Static_assert(MAL_C_ABI_VERSION == 0x000900u, "generated header requires mal C ABI 0x000900");
_Static_assert((sizeof((size_t)0) * CHAR_BIT) == 64, "size_t does not match the mal target pointer index width");

#ifndef MAL_GENERATED_INTERFACE_F380FACF02472957_H
#define MAL_GENERATED_INTERFACE_F380FACF02472957_H
/* Host-visible types */

#ifndef MAL_DETAIL_HOST_REPR_cc5bd8c1c1e15396_DECLARED
#define MAL_DETAIL_HOST_REPR_cc5bd8c1c1e15396_DECLARED
typedef struct mal_detail_repr_product_cc5bd8c1c1e15396 mal_repr_product_cc5bd8c1c1e15396_t;
#endif
typedef mal_repr_product_cc5bd8c1c1e15396_t mal_NodeRow_t;

#ifndef MAL_DETAIL_HOST_REPR_cc5bd8c1c1e15396_DEFINED
#define MAL_DETAIL_HOST_REPR_cc5bd8c1c1e15396_DEFINED
struct mal_detail_repr_product_cc5bd8c1c1e15396 {
    mal_Int32_t field_0;
    mal_UInt8_t field_1;
    mal_USize_t field_2;
    mal_USize_t field_3;
};
#endif

/* Canonical memory access */

#ifndef MAL_DETAIL_MEMORY_Int32_HELPERS
#define MAL_DETAIL_MEMORY_Int32_HELPERS
static inline mal_Int32_t mal_detail_memory_read_Int32(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, const uint8_t *source) {
    mal_Int32_t value;
    memcpy(&value, source, sizeof(value));
    return value;
}

static inline void mal_detail_memory_write_Int32(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, uint8_t *destination, mal_Int32_t value) {
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

#ifndef MAL_DETAIL_MEMORY_USize_HELPERS
#define MAL_DETAIL_MEMORY_USize_HELPERS
static inline mal_USize_t mal_detail_memory_read_USize(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, const uint8_t *source) {
    mal_USize_t value;
    memcpy(&value, source, sizeof(value));
    return value;
}

static inline void mal_detail_memory_write_USize(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, uint8_t *destination, mal_USize_t value) {
    memcpy(destination, &value, sizeof(value));
}

#endif

#ifndef MAL_DETAIL_MEMORY_REPR_cc5bd8c1c1e15396_HELPERS
#define MAL_DETAIL_MEMORY_REPR_cc5bd8c1c1e15396_HELPERS
static inline mal_repr_product_cc5bd8c1c1e15396_t mal_detail_memory_read_cc5bd8c1c1e15396(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, const uint8_t *source MAL_DETAIL_MAYBE_UNUSED) {
    mal_repr_product_cc5bd8c1c1e15396_t value;
    value.field_0 = mal_detail_memory_read_Int32(call, source + 0);
    value.field_1 = mal_detail_memory_read_UInt8(call, source + 4);
    value.field_2 = mal_detail_memory_read_USize(call, source + 8);
    value.field_3 = mal_detail_memory_read_USize(call, source + 16);
    return value;
}

static inline void mal_detail_memory_write_cc5bd8c1c1e15396(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, uint8_t *destination MAL_DETAIL_MAYBE_UNUSED, mal_repr_product_cc5bd8c1c1e15396_t value) {
    mal_detail_memory_write_Int32(call, destination + 0, value.field_0);
    mal_detail_memory_write_UInt8(call, destination + 4, value.field_1);
    mal_detail_memory_write_USize(call, destination + 8, value.field_2);
    mal_detail_memory_write_USize(call, destination + 16, value.field_3);
}

#endif

static inline mal_NodeRow_t mal_NodeRow_read(mal_call_t *call, mal_Address_t address, mal_USize_t index) {
    mal_Address_return(call, address);
    return mal_detail_memory_read_cc5bd8c1c1e15396(call, (const uint8_t *)address + (index * 24));
}

static inline void mal_NodeRow_write(mal_call_t *call, mal_Address_t address, mal_USize_t index, mal_NodeRow_t value) {
    mal_Address_return(call, address);
    mal_detail_memory_write_cc5bd8c1c1e15396(call, (uint8_t *)address + (index * 24), value);
}

#endif
#endif
