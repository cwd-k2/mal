#ifndef MAL_BUILD_UMBRELLA
#include <mal.h>
#include "memory.mal.h"
#include "syscall.mal.h"
#include <string.h>

_Static_assert(MAL_C_ABI_VERSION == 0x000900u, "generated header requires mal C ABI 0x000900");
_Static_assert((sizeof((size_t)0) * CHAR_BIT) == 64, "size_t does not match the mal target pointer index width");

#ifndef MAL_GENERATED_INTERFACE_0CD9BB85B2EA5F5D_H
#define MAL_GENERATED_INTERFACE_0CD9BB85B2EA5F5D_H
/* Host-visible types */

#ifndef MAL_DETAIL_HOST_REPR_bebc720e190b0ce6_DECLARED
#define MAL_DETAIL_HOST_REPR_bebc720e190b0ce6_DECLARED
typedef struct mal_detail_repr_product_bebc720e190b0ce6 mal_repr_product_bebc720e190b0ce6_t;
#endif
#ifndef MAL_DETAIL_HOST_REPR_2147c8899a0db6f9_DECLARED
#define MAL_DETAIL_HOST_REPR_2147c8899a0db6f9_DECLARED
typedef struct mal_detail_repr_product_2147c8899a0db6f9 mal_repr_product_2147c8899a0db6f9_t;
#endif
#ifndef MAL_DETAIL_HOST_REPR_7ff446105564b76c_DECLARED
#define MAL_DETAIL_HOST_REPR_7ff446105564b76c_DECLARED
typedef struct mal_detail_repr_sum_7ff446105564b76c mal_repr_sum_7ff446105564b76c_t;
#endif
typedef mal_repr_product_bebc720e190b0ce6_t mal_MappingBuffer_t;
typedef mal_repr_product_2147c8899a0db6f9_t mal_MappingFailure_t;
typedef mal_repr_sum_7ff446105564b76c_t mal_MappingResult_t;

#ifndef MAL_DETAIL_HOST_REPR_bebc720e190b0ce6_DEFINED
#define MAL_DETAIL_HOST_REPR_bebc720e190b0ce6_DEFINED
struct mal_detail_repr_product_bebc720e190b0ce6 {
    mal_Address_t field_0;
    mal_ByteSize_t field_1;
    mal_ByteSize_t field_2;
};
#endif

#ifndef MAL_DETAIL_HOST_REPR_2147c8899a0db6f9_DEFINED
#define MAL_DETAIL_HOST_REPR_2147c8899a0db6f9_DEFINED
struct mal_detail_repr_product_2147c8899a0db6f9 {
    mal_repr_product_bebc720e190b0ce6_t field_0;
    mal_UInt32_t field_1;
};
#endif

#ifndef MAL_DETAIL_HOST_REPR_7ff446105564b76c_DEFINED
#define MAL_DETAIL_HOST_REPR_7ff446105564b76c_DEFINED
struct mal_detail_repr_sum_7ff446105564b76c {
    uint32_t tag;
    union {
        mal_repr_product_bebc720e190b0ce6_t variant_0;
        mal_repr_product_2147c8899a0db6f9_t variant_1;
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

#ifndef MAL_DETAIL_MEMORY_UInt32_HELPERS
#define MAL_DETAIL_MEMORY_UInt32_HELPERS
static inline mal_UInt32_t mal_detail_memory_read_UInt32(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, const uint8_t *source) {
    mal_UInt32_t value;
    memcpy(&value, source, sizeof(value));
    return value;
}

static inline void mal_detail_memory_write_UInt32(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, uint8_t *destination, mal_UInt32_t value) {
    memcpy(destination, &value, sizeof(value));
}

#endif

#ifndef MAL_DETAIL_MEMORY_Address_HELPERS
#define MAL_DETAIL_MEMORY_Address_HELPERS
static inline mal_Address_t mal_detail_memory_read_Address(mal_call_t *call, const uint8_t *source) {
    mal_Address_t value;
    memcpy(&value, source, sizeof(value));
    return mal_Address_return(call, value);
}

static inline void mal_detail_memory_write_Address(mal_call_t *call, uint8_t *destination, mal_Address_t value) {
    mal_Address_return(call, value);
    memcpy(destination, &value, sizeof(value));
}

#endif

#ifndef MAL_DETAIL_MEMORY_ByteSize_HELPERS
#define MAL_DETAIL_MEMORY_ByteSize_HELPERS
static inline mal_ByteSize_t mal_detail_memory_read_ByteSize(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, const uint8_t *source) {
    mal_ByteSize_t value;
    memcpy(&value, source, sizeof(value));
    return value;
}

static inline void mal_detail_memory_write_ByteSize(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, uint8_t *destination, mal_ByteSize_t value) {
    memcpy(destination, &value, sizeof(value));
}

#endif

#ifndef MAL_DETAIL_MEMORY_REPR_bebc720e190b0ce6_HELPERS
#define MAL_DETAIL_MEMORY_REPR_bebc720e190b0ce6_HELPERS
static inline mal_repr_product_bebc720e190b0ce6_t mal_detail_memory_read_bebc720e190b0ce6(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, const uint8_t *source MAL_DETAIL_MAYBE_UNUSED) {
    mal_repr_product_bebc720e190b0ce6_t value;
    value.field_0 = mal_detail_memory_read_Address(call, source + 0);
    value.field_1 = mal_detail_memory_read_ByteSize(call, source + 8);
    value.field_2 = mal_detail_memory_read_ByteSize(call, source + 16);
    return value;
}

static inline void mal_detail_memory_write_bebc720e190b0ce6(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, uint8_t *destination MAL_DETAIL_MAYBE_UNUSED, mal_repr_product_bebc720e190b0ce6_t value) {
    mal_detail_memory_write_Address(call, destination + 0, value.field_0);
    mal_detail_memory_write_ByteSize(call, destination + 8, value.field_1);
    mal_detail_memory_write_ByteSize(call, destination + 16, value.field_2);
}

#endif

#ifndef MAL_DETAIL_MEMORY_REPR_2147c8899a0db6f9_HELPERS
#define MAL_DETAIL_MEMORY_REPR_2147c8899a0db6f9_HELPERS
static inline mal_repr_product_2147c8899a0db6f9_t mal_detail_memory_read_2147c8899a0db6f9(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, const uint8_t *source MAL_DETAIL_MAYBE_UNUSED) {
    mal_repr_product_2147c8899a0db6f9_t value;
    value.field_0 = mal_detail_memory_read_bebc720e190b0ce6(call, source + 0);
    value.field_1 = mal_detail_memory_read_UInt32(call, source + 24);
    return value;
}

static inline void mal_detail_memory_write_2147c8899a0db6f9(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, uint8_t *destination MAL_DETAIL_MAYBE_UNUSED, mal_repr_product_2147c8899a0db6f9_t value) {
    mal_detail_memory_write_bebc720e190b0ce6(call, destination + 0, value.field_0);
    mal_detail_memory_write_UInt32(call, destination + 24, value.field_1);
}

#endif

#ifndef MAL_DETAIL_MEMORY_REPR_7ff446105564b76c_HELPERS
#define MAL_DETAIL_MEMORY_REPR_7ff446105564b76c_HELPERS
static inline mal_repr_sum_7ff446105564b76c_t mal_detail_memory_read_7ff446105564b76c(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, const uint8_t *source MAL_DETAIL_MAYBE_UNUSED) {
    switch (mal_detail_memory_read_UInt8(call, source)) {
        case 0: {
            return (mal_repr_sum_7ff446105564b76c_t){ .tag = UINT32_C(0), .payload.variant_0 = mal_detail_memory_read_bebc720e190b0ce6(call, source + 8) };
        }
        case 1: {
            return (mal_repr_sum_7ff446105564b76c_t){ .tag = UINT32_C(1), .payload.variant_1 = mal_detail_memory_read_2147c8899a0db6f9(call, source + 8) };
        }
        default: {
            mal_call_trap(call, "invalid canonical sum tag");
        }
    }
}

static inline void mal_detail_memory_write_7ff446105564b76c(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, uint8_t *destination MAL_DETAIL_MAYBE_UNUSED, mal_repr_sum_7ff446105564b76c_t value) {
    switch (value.tag) {
        case UINT32_C(0): {
            mal_detail_memory_write_UInt8(call, destination, (mal_UInt8_t)value.tag);
            mal_detail_memory_write_bebc720e190b0ce6(call, destination + 8, value.payload.variant_0);
            return;
        }
        case UINT32_C(1): {
            mal_detail_memory_write_UInt8(call, destination, (mal_UInt8_t)value.tag);
            mal_detail_memory_write_2147c8899a0db6f9(call, destination + 8, value.payload.variant_1);
            return;
        }
        default: {
            mal_call_trap(call, "invalid sum tag");
        }
    }
}

#endif

static inline mal_MappingBuffer_t mal_MappingBuffer_read(mal_call_t *call, mal_Address_t address, mal_USize_t index) {
    mal_Address_return(call, address);
    return mal_detail_memory_read_bebc720e190b0ce6(call, (const uint8_t *)address + (index * 24));
}

static inline void mal_MappingBuffer_write(mal_call_t *call, mal_Address_t address, mal_USize_t index, mal_MappingBuffer_t value) {
    mal_Address_return(call, address);
    mal_detail_memory_write_bebc720e190b0ce6(call, (uint8_t *)address + (index * 24), value);
}

static inline mal_MappingFailure_t mal_MappingFailure_read(mal_call_t *call, mal_Address_t address, mal_USize_t index) {
    mal_Address_return(call, address);
    return mal_detail_memory_read_2147c8899a0db6f9(call, (const uint8_t *)address + (index * 32));
}

static inline void mal_MappingFailure_write(mal_call_t *call, mal_Address_t address, mal_USize_t index, mal_MappingFailure_t value) {
    mal_Address_return(call, address);
    mal_detail_memory_write_2147c8899a0db6f9(call, (uint8_t *)address + (index * 32), value);
}

static inline mal_MappingResult_t mal_MappingResult_read(mal_call_t *call, mal_Address_t address, mal_USize_t index) {
    mal_Address_return(call, address);
    return mal_detail_memory_read_7ff446105564b76c(call, (const uint8_t *)address + (index * 40));
}

static inline void mal_MappingResult_write(mal_call_t *call, mal_Address_t address, mal_USize_t index, mal_MappingResult_t value) {
    mal_Address_return(call, address);
    mal_detail_memory_write_7ff446105564b76c(call, (uint8_t *)address + (index * 40), value);
}

#endif
#endif
