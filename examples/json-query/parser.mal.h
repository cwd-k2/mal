#ifndef MAL_BUILD_UMBRELLA
#include <mal.h>
#include "bytes.mal.h"
#include "scanner.mal.h"
#include <string.h>

_Static_assert(MAL_C_ABI_VERSION == 0x000900u, "generated header requires mal C ABI 0x000900");
_Static_assert((sizeof((size_t)0) * CHAR_BIT) == 64, "size_t does not match the mal target pointer index width");

#ifndef MAL_GENERATED_INTERFACE_230EB8491D457170_H
#define MAL_GENERATED_INTERFACE_230EB8491D457170_H
/* Host-visible types */

#ifndef MAL_DETAIL_HOST_REPR_1e5f7be9f35ae586_DECLARED
#define MAL_DETAIL_HOST_REPR_1e5f7be9f35ae586_DECLARED
typedef struct mal_detail_repr_product_1e5f7be9f35ae586 mal_repr_product_1e5f7be9f35ae586_t;
#endif
typedef mal_repr_product_1e5f7be9f35ae586_t mal_JsonStatistics_t;
typedef mal_UInt8_t mal_ParserFrame_t;

#ifndef MAL_DETAIL_HOST_REPR_1e5f7be9f35ae586_DEFINED
#define MAL_DETAIL_HOST_REPR_1e5f7be9f35ae586_DEFINED
struct mal_detail_repr_product_1e5f7be9f35ae586 {
    mal_UInt64_t field_0;
    mal_UInt64_t field_1;
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

#ifndef MAL_DETAIL_MEMORY_UInt64_HELPERS
#define MAL_DETAIL_MEMORY_UInt64_HELPERS
static inline mal_UInt64_t mal_detail_memory_read_UInt64(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, const uint8_t *source) {
    mal_UInt64_t value;
    memcpy(&value, source, sizeof(value));
    return value;
}

static inline void mal_detail_memory_write_UInt64(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, uint8_t *destination, mal_UInt64_t value) {
    memcpy(destination, &value, sizeof(value));
}

#endif

#ifndef MAL_DETAIL_MEMORY_REPR_1e5f7be9f35ae586_HELPERS
#define MAL_DETAIL_MEMORY_REPR_1e5f7be9f35ae586_HELPERS
static inline mal_repr_product_1e5f7be9f35ae586_t mal_detail_memory_read_1e5f7be9f35ae586(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, const uint8_t *source MAL_DETAIL_MAYBE_UNUSED) {
    mal_repr_product_1e5f7be9f35ae586_t value;
    value.field_0 = mal_detail_memory_read_UInt64(call, source + 0);
    value.field_1 = mal_detail_memory_read_UInt64(call, source + 8);
    return value;
}

static inline void mal_detail_memory_write_1e5f7be9f35ae586(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, uint8_t *destination MAL_DETAIL_MAYBE_UNUSED, mal_repr_product_1e5f7be9f35ae586_t value) {
    mal_detail_memory_write_UInt64(call, destination + 0, value.field_0);
    mal_detail_memory_write_UInt64(call, destination + 8, value.field_1);
}

#endif

static inline mal_JsonStatistics_t mal_JsonStatistics_read(mal_call_t *call, mal_Address_t address, mal_USize_t index) {
    mal_Address_return(call, address);
    return mal_detail_memory_read_1e5f7be9f35ae586(call, (const uint8_t *)address + (index * 16));
}

static inline void mal_JsonStatistics_write(mal_call_t *call, mal_Address_t address, mal_USize_t index, mal_JsonStatistics_t value) {
    mal_Address_return(call, address);
    mal_detail_memory_write_1e5f7be9f35ae586(call, (uint8_t *)address + (index * 16), value);
}

static inline mal_ParserFrame_t mal_ParserFrame_read(mal_call_t *call, mal_Address_t address, mal_USize_t index) {
    mal_Address_return(call, address);
    return mal_detail_memory_read_UInt8(call, (const uint8_t *)address + (index * 1));
}

static inline void mal_ParserFrame_write(mal_call_t *call, mal_Address_t address, mal_USize_t index, mal_ParserFrame_t value) {
    mal_Address_return(call, address);
    mal_detail_memory_write_UInt8(call, (uint8_t *)address + (index * 1), value);
}

#endif
#endif
