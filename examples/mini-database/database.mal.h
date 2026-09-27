#ifndef MAL_BUILD_UMBRELLA
#include <mal.h>
#include "host.mal.h"
#include "bytes.mal.h"
#include <string.h>

_Static_assert(MAL_C_ABI_VERSION == 0x000900u, "generated header requires mal C ABI 0x000900");
_Static_assert((sizeof((size_t)0) * CHAR_BIT) == 64, "size_t does not match the mal target pointer index width");

#ifndef MAL_GENERATED_INTERFACE_95B34C6CE4E6D12A_H
#define MAL_GENERATED_INTERFACE_95B34C6CE4E6D12A_H
/* Host-visible types */

#ifndef MAL_DETAIL_HOST_REPR_67c07b59ec15f9f0_DECLARED
#define MAL_DETAIL_HOST_REPR_67c07b59ec15f9f0_DECLARED
typedef struct mal_detail_repr_sum_67c07b59ec15f9f0 mal_repr_sum_67c07b59ec15f9f0_t;
#endif
typedef mal_repr_sum_67c07b59ec15f9f0_t mal_PutResult_t;

#ifndef MAL_DETAIL_HOST_REPR_67c07b59ec15f9f0_DEFINED
#define MAL_DETAIL_HOST_REPR_67c07b59ec15f9f0_DEFINED
struct mal_detail_repr_sum_67c07b59ec15f9f0 {
    uint32_t tag;
    union {
        mal_Unit_t variant_0;
        mal_Unit_t variant_1;
        mal_Unit_t variant_2;
    } payload;
};
#endif

/* Canonical memory access */

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

#ifndef MAL_DETAIL_MEMORY_REPR_67c07b59ec15f9f0_HELPERS
#define MAL_DETAIL_MEMORY_REPR_67c07b59ec15f9f0_HELPERS
static inline mal_repr_sum_67c07b59ec15f9f0_t mal_detail_memory_read_67c07b59ec15f9f0(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, const uint8_t *source MAL_DETAIL_MAYBE_UNUSED) {
    switch (mal_detail_memory_read_UInt8(call, source)) {
        case 0: {
            return (mal_repr_sum_67c07b59ec15f9f0_t){ .tag = UINT32_C(0), .payload.variant_0 = (mal_Unit_t){ 0 } };
        }
        case 1: {
            return (mal_repr_sum_67c07b59ec15f9f0_t){ .tag = UINT32_C(1), .payload.variant_1 = (mal_Unit_t){ 0 } };
        }
        case 2: {
            return (mal_repr_sum_67c07b59ec15f9f0_t){ .tag = UINT32_C(2), .payload.variant_2 = (mal_Unit_t){ 0 } };
        }
        default: {
            mal_call_trap(call, "invalid canonical sum tag");
        }
    }
}

static inline void mal_detail_memory_write_67c07b59ec15f9f0(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, uint8_t *destination MAL_DETAIL_MAYBE_UNUSED, mal_repr_sum_67c07b59ec15f9f0_t value) {
    switch (value.tag) {
        case UINT32_C(0): {
            mal_detail_memory_write_UInt8(call, destination, (mal_UInt8_t)value.tag);
            (void)value.payload.variant_0;
            return;
        }
        case UINT32_C(1): {
            mal_detail_memory_write_UInt8(call, destination, (mal_UInt8_t)value.tag);
            (void)value.payload.variant_1;
            return;
        }
        case UINT32_C(2): {
            mal_detail_memory_write_UInt8(call, destination, (mal_UInt8_t)value.tag);
            (void)value.payload.variant_2;
            return;
        }
        default: {
            mal_call_trap(call, "invalid sum tag");
        }
    }
}

#endif

static inline mal_PutResult_t mal_PutResult_read(mal_call_t *call, mal_Address_t address, mal_USize_t index) {
    mal_Address_return(call, address);
    return mal_detail_memory_read_67c07b59ec15f9f0(call, (const uint8_t *)address + (index * 1));
}

static inline void mal_PutResult_write(mal_call_t *call, mal_Address_t address, mal_USize_t index, mal_PutResult_t value) {
    mal_Address_return(call, address);
    mal_detail_memory_write_67c07b59ec15f9f0(call, (uint8_t *)address + (index * 1), value);
}

#endif
#endif
