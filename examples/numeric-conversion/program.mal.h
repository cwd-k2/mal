#ifndef MAL_PROGRAM_MAL_H
#define MAL_PROGRAM_MAL_H

#include <mal.h>

_Static_assert(MAL_C_ABI_VERSION == 0x000900u, "generated header requires mal C ABI 0x000900");
_Static_assert((sizeof((size_t)0) * CHAR_BIT) == 64, "size_t does not match the mal target pointer index width");

/* External operations */

void mal_ext_printUInt64(MalContext *context, MalType_UInt64 value);

/* External definition helpers */

#define MAL_HAS_EXTERN_printUInt64 1
#define MAL_DEFINE_printUInt64(call, value) \
static MalType_Unit mal_detail_printUInt64(mal_call_t *call, mal_UInt64_t value); \
void mal_ext_printUInt64(MalContext *context MAL_DETAIL_MAYBE_UNUSED, MalType_UInt64 value) { \
    mal_call_t call = (mal_call_t){ .mal_detail_context = context }; \
    mal_detail_printUInt64(&call, value); \
} \
static MalType_Unit mal_detail_printUInt64( \
    mal_call_t *call, \
    mal_UInt64_t value \
)

#endif
