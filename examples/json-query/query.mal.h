#ifndef MAL_BUILD_UMBRELLA
#include <mal.h>
#include "bytes.mal.h"
#include <string.h>

_Static_assert(MAL_C_ABI_VERSION == 0x000900u, "generated header requires mal C ABI 0x000900");
_Static_assert((sizeof((size_t)0) * CHAR_BIT) == 64, "size_t does not match the mal target pointer index width");

#ifndef MAL_GENERATED_INTERFACE_ED1A35C58816407C_H
#define MAL_GENERATED_INTERFACE_ED1A35C58816407C_H
/* Host-visible types */

typedef mal_Bool_t mal_Query_t;

/* Canonical memory access */

#ifndef MAL_DETAIL_MEMORY_Bool_HELPERS
#define MAL_DETAIL_MEMORY_Bool_HELPERS
static inline mal_Bool_t mal_detail_memory_read_Bool(mal_call_t *call, const uint8_t *source) {
    mal_Bool_t value;
    memcpy(&value, source, sizeof(value));
    return mal_Bool_return(call, value);
}

static inline void mal_detail_memory_write_Bool(mal_call_t *call, uint8_t *destination, mal_Bool_t value) {
    mal_Bool_return(call, value);
    memcpy(destination, &value, sizeof(value));
}

#endif

static inline mal_Query_t mal_Query_read(mal_call_t *call, mal_Address_t address, mal_USize_t index) {
    mal_Address_return(call, address);
    return mal_detail_memory_read_Bool(call, (const uint8_t *)address + (index * 1));
}

static inline void mal_Query_write(mal_call_t *call, mal_Address_t address, mal_USize_t index, mal_Query_t value) {
    mal_Address_return(call, address);
    mal_detail_memory_write_Bool(call, (uint8_t *)address + (index * 1), value);
}

#endif
#endif
