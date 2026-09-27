#ifndef MAL_BUILD_UMBRELLA
#include <mal.h>

_Static_assert(MAL_C_ABI_VERSION == 0x000900u, "generated header requires mal C ABI 0x000900");
_Static_assert((sizeof((size_t)0) * CHAR_BIT) == 64, "size_t does not match the mal target pointer index width");

#ifndef MAL_GENERATED_INTERFACE_291F2F730C6EF32B_H
#define MAL_GENERATED_INTERFACE_291F2F730C6EF32B_H

/* External operations */

void mal_ext__printInt32(MalContext *context, MalType_Int32 value);

/* External definition helpers */

#define MAL_HAS_EXTERN__printInt32 1
#define MAL_DEFINE__printInt32(call, value) \
static MalType_Unit mal_detail__printInt32(mal_call_t *call, mal_Int32_t value); \
void mal_ext__printInt32(MalContext *context MAL_DETAIL_MAYBE_UNUSED, MalType_Int32 value) { \
    mal_call_t call = (mal_call_t){ .mal_detail_context = context }; \
    mal_detail__printInt32(&call, value); \
} \
static MalType_Unit mal_detail__printInt32( \
    mal_call_t *call, \
    mal_Int32_t value \
)

#endif
#endif
