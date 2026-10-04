#ifndef MAL_BUILD_UMBRELLA
#include <mal.h>
#include "box.mal.h"

_Static_assert(MAL_C_ABI_VERSION == 0x000a00u, "generated header requires mal C ABI 0x000a00");
_Static_assert(sizeof((size_t)0) == 8, "size_t does not match the mal target index width");
_Static_assert(sizeof((void *)0) == 8, "C pointer size does not match the mal target");
_Static_assert(sizeof(*(mal_Symbol_t *)0) == 24, "Symbol carrier size does not match the mal target");
_Static_assert(offsetof(mal_Symbol_t, owner) == 0, "Symbol owner offset does not match the mal target");
_Static_assert(offsetof(mal_Symbol_t, data) == 8, "Symbol data offset does not match the mal target");
_Static_assert(offsetof(mal_Symbol_t, length) == 16, "Symbol length offset does not match the mal target");

#ifndef MAL_GENERATED_INTERFACE_8C2142295E56EB16_H
#define MAL_GENERATED_INTERFACE_8C2142295E56EB16_H

/* Host-visible types */

#ifndef MAL_DETAIL_REPR_11bb4291f5045bdf_DECLARED
#define MAL_DETAIL_REPR_11bb4291f5045bdf_DECLARED
typedef struct mal_detail_repr_product_11bb4291f5045bdf mal_repr_product_11bb4291f5045bdf_t;
#endif

#ifndef MAL_DETAIL_REPR_FIELDS_11bb4291f5045bdf_DEFINED
#define MAL_DETAIL_REPR_FIELDS_11bb4291f5045bdf_DEFINED
#define MAL_DETAIL_REPR_FIELDS_11bb4291f5045bdf(field, context) \
field(context, 0, field_0, mal_UInt64_t) \
field(context, 1, field_1, mal_UInt64_t) \
field(context, 2, field_2, mal_UInt64_t) \
field(context, 3, field_3, mal_Float32_t) \
field(context, 4, field_4, mal_Float32_t) \
field(context, 5, field_5, mal_Float64_t) \
field(context, 6, field_6, mal_Int64_t)
#endif
#ifndef MAL_DETAIL_REPR_11bb4291f5045bdf_DEFINED
#define MAL_DETAIL_REPR_11bb4291f5045bdf_DEFINED
MAL_DETAIL_DEFINE_PRODUCT_REPR(mal_detail_repr_product_11bb4291f5045bdf, MAL_DETAIL_REPR_FIELDS_11bb4291f5045bdf, MAL_DETAIL_REPR_FIELD)
typedef void (*mal_detail_product_key_11bb4291f5045bdf_t)(mal_UInt64_t, mal_UInt64_t, mal_UInt64_t, mal_Float32_t, mal_Float32_t, mal_Float64_t, mal_Int64_t);
__attribute__((overloadable)) mal_repr_product_11bb4291f5045bdf_t *mal_detail_product_type(mal_detail_product_key_11bb4291f5045bdf_t);
#endif

/* External operations */

mal_Unit_t mal_ext_printInt32(mal_call_t *call, mal_Int32_t value);
mal_Int32_t mal_ext_inspectNumeric(mal_call_t *call, mal_repr_product_11bb4291f5045bdf_t value);

/* External definition helpers */

#define MAL_HAS_EXTERN_printInt32 1
#define MAL_DEFINE_printInt32(call, value) mal_Unit_t mal_ext_printInt32(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, mal_Int32_t value)

#define MAL_HAS_EXTERN_inspectNumeric 1
#define MAL_DEFINE_inspectNumeric(call, value) mal_Int32_t mal_ext_inspectNumeric(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, mal_repr_product_11bb4291f5045bdf_t value)

#endif
#endif
