#ifndef MAL_BUILD_UMBRELLA
#include <mal.h>

_Static_assert(MAL_C_ABI_VERSION == 0x000b00u, "generated header requires mal C ABI 0x000b00");
_Static_assert(sizeof((size_t)0) == 8, "size_t does not match the mal target index width");
_Static_assert(sizeof((void *)0) == 8, "C pointer size does not match the mal target");
_Static_assert(sizeof(*(mal_Symbol_t *)0) == 24, "Symbol carrier size does not match the mal target");
_Static_assert(offsetof(mal_Symbol_t, owner) == 0, "Symbol owner offset does not match the mal target");
_Static_assert(offsetof(mal_Symbol_t, data) == 8, "Symbol data offset does not match the mal target");
_Static_assert(offsetof(mal_Symbol_t, length) == 16, "Symbol length offset does not match the mal target");

#ifndef MAL_GENERATED_INTERFACE_42C4C77CF48EAEC2_H
#define MAL_GENERATED_INTERFACE_42C4C77CF48EAEC2_H

/* External operations */

mal_Buffer_t mal_ext_receive(mal_call_t *call);
mal_Unit_t mal_ext_send(mal_call_t *call, mal_Symbol_t value);

/* External definition helpers */

#define MAL_HAS_EXTERN_receive 1
#define MAL_DEFINE_receive(call) mal_Buffer_t mal_ext_receive(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED)

#define MAL_HAS_EXTERN_send 1
#define MAL_DEFINE_send(call, value) mal_Unit_t mal_ext_send(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, mal_Symbol_t value)

#endif
#endif
