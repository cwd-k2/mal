#ifndef MAL_BUILD_UMBRELLA
#include <mal.h>

_Static_assert(MAL_C_ABI_VERSION == 0x000a00u, "generated header requires mal C ABI 0x000a00");
_Static_assert(sizeof((size_t)0) == 8, "size_t does not match the mal target index width");
_Static_assert(sizeof((void *)0) == 8, "C pointer size does not match the mal target");
_Static_assert(sizeof(*(MalType_Symbol *)0) == 24, "Symbol carrier size does not match the mal target");
_Static_assert(offsetof(MalType_Symbol, owner) == 0, "Symbol owner offset does not match the mal target");
_Static_assert(offsetof(MalType_Symbol, data) == 8, "Symbol data offset does not match the mal target");
_Static_assert(offsetof(MalType_Symbol, length) == 16, "Symbol length offset does not match the mal target");

#ifndef MAL_GENERATED_INTERFACE_82ED2F22944C26FE_H
#define MAL_GENERATED_INTERFACE_82ED2F22944C26FE_H

/* External operations */

MalType_Buffer mal_ext_receive(MalContext *context);
void mal_ext_send(MalContext *context, MalType_Symbol value);

/* External definition helpers */

#define MAL_HAS_EXTERN_receive 1
#define MAL_DEFINE_receive(call) \
static mal_Buffer_t mal_detail_receive(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED); \
MalType_Buffer mal_ext_receive(MalContext *context MAL_DETAIL_MAYBE_UNUSED) { \
    mal_call_t call = (mal_call_t){ .mal_detail_context = context }; \
    return mal_detail_receive(&call); \
} \
static mal_Buffer_t mal_detail_receive( \
    mal_call_t *call MAL_DETAIL_MAYBE_UNUSED \
)

#define MAL_HAS_EXTERN_send 1
#define MAL_DEFINE_send(call, value) \
static mal_Unit_t mal_detail_send(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, mal_Symbol_t value); \
void mal_ext_send(MalContext *context MAL_DETAIL_MAYBE_UNUSED, MalType_Symbol value) { \
    mal_call_t call = (mal_call_t){ .mal_detail_context = context }; \
    mal_detail_send(&call, value); \
} \
static mal_Unit_t mal_detail_send( \
    mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, \
    mal_Symbol_t value \
)

#endif
#endif
