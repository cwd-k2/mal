#ifndef MAL_BUILD_UMBRELLA
#include <mal.h>
#include <string.h>

_Static_assert(MAL_C_ABI_VERSION == 0x000900u, "generated header requires mal C ABI 0x000900");
_Static_assert((sizeof((size_t)0) * CHAR_BIT) == 64, "size_t does not match the mal target pointer index width");

#ifndef MAL_GENERATED_INTERFACE_4BA10070B8E844F5_H
#define MAL_GENERATED_INTERFACE_4BA10070B8E844F5_H
/* Host-visible types */

#ifndef MAL_DETAIL_HOST_REPR_1e555ce9f3525c54_DECLARED
#define MAL_DETAIL_HOST_REPR_1e555ce9f3525c54_DECLARED
typedef struct mal_detail_repr_product_1e555ce9f3525c54 mal_repr_product_1e555ce9f3525c54_t;
#endif
typedef mal_repr_product_1e555ce9f3525c54_t mal_PairRow_t;

#ifndef MAL_DETAIL_HOST_REPR_1e555ce9f3525c54_DEFINED
#define MAL_DETAIL_HOST_REPR_1e555ce9f3525c54_DEFINED
struct mal_detail_repr_product_1e555ce9f3525c54 {
    mal_USize_t field_0;
    mal_USize_t field_1;
};
#endif

/* Canonical memory access */

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

#ifndef MAL_DETAIL_MEMORY_REPR_1e555ce9f3525c54_HELPERS
#define MAL_DETAIL_MEMORY_REPR_1e555ce9f3525c54_HELPERS
static inline mal_repr_product_1e555ce9f3525c54_t mal_detail_memory_read_1e555ce9f3525c54(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, const uint8_t *source MAL_DETAIL_MAYBE_UNUSED) {
    mal_repr_product_1e555ce9f3525c54_t value;
    value.field_0 = mal_detail_memory_read_USize(call, source + 0);
    value.field_1 = mal_detail_memory_read_USize(call, source + 8);
    return value;
}

static inline void mal_detail_memory_write_1e555ce9f3525c54(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, uint8_t *destination MAL_DETAIL_MAYBE_UNUSED, mal_repr_product_1e555ce9f3525c54_t value) {
    mal_detail_memory_write_USize(call, destination + 0, value.field_0);
    mal_detail_memory_write_USize(call, destination + 8, value.field_1);
}

#endif

static inline mal_PairRow_t mal_PairRow_read(mal_call_t *call, mal_Address_t address, mal_USize_t index) {
    mal_Address_return(call, address);
    return mal_detail_memory_read_1e555ce9f3525c54(call, (const uint8_t *)address + (index * 16));
}

static inline void mal_PairRow_write(mal_call_t *call, mal_Address_t address, mal_USize_t index, mal_PairRow_t value) {
    mal_Address_return(call, address);
    mal_detail_memory_write_1e555ce9f3525c54(call, (uint8_t *)address + (index * 16), value);
}

#endif
#endif
