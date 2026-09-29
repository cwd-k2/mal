#ifndef MAL_BUILD_UMBRELLA
#include <mal.h>

_Static_assert(MAL_C_ABI_VERSION == 0x000900u, "generated header requires mal C ABI 0x000900");
_Static_assert((sizeof((size_t)0) * CHAR_BIT) == 64, "size_t does not match the mal target pointer index width");

#ifndef MAL_GENERATED_INTERFACE_8E0D4E1EF0FBA680_H
#define MAL_GENERATED_INTERFACE_8E0D4E1EF0FBA680_H

/* Host-visible types */

#ifndef MAL_DETAIL_RAW_REPR_11bb4291f5045bdf_DECLARED
#define MAL_DETAIL_RAW_REPR_11bb4291f5045bdf_DECLARED
typedef struct MalRepr_Product_11bb4291f5045bdf MalRepr_Product_11bb4291f5045bdf;
#endif

#ifndef MAL_DETAIL_REPR_FIELDS_11bb4291f5045bdf_DEFINED
#define MAL_DETAIL_REPR_FIELDS_11bb4291f5045bdf_DEFINED
#define MAL_DETAIL_REPR_FIELDS_11bb4291f5045bdf(field, context) \
field(context, 0, field_0, MalType_UInt64, mal_UInt64_t, MAL_DETAIL_REPR_IDENTITY, MAL_DETAIL_REPR_IDENTITY) \
field(context, 1, field_1, MalType_UInt64, mal_UInt64_t, MAL_DETAIL_REPR_IDENTITY, MAL_DETAIL_REPR_IDENTITY) \
field(context, 2, field_2, MalType_UInt64, mal_UInt64_t, MAL_DETAIL_REPR_IDENTITY, MAL_DETAIL_REPR_IDENTITY) \
field(context, 3, field_3, MalType_Float32, mal_Float32_t, MAL_DETAIL_REPR_IDENTITY, MAL_DETAIL_REPR_IDENTITY) \
field(context, 4, field_4, MalType_Float32, mal_Float32_t, MAL_DETAIL_REPR_IDENTITY, MAL_DETAIL_REPR_IDENTITY) \
field(context, 5, field_5, MalType_Float64, mal_Float64_t, MAL_DETAIL_REPR_IDENTITY, MAL_DETAIL_REPR_IDENTITY) \
field(context, 6, field_6, MalType_Int64, mal_Int64_t, MAL_DETAIL_REPR_IDENTITY, MAL_DETAIL_REPR_IDENTITY)
#endif
#ifndef MAL_DETAIL_RAW_REPR_11bb4291f5045bdf_DEFINED
#define MAL_DETAIL_RAW_REPR_11bb4291f5045bdf_DEFINED
MAL_DETAIL_DEFINE_PRODUCT_REPR(MalRepr_Product_11bb4291f5045bdf, MAL_DETAIL_REPR_FIELDS_11bb4291f5045bdf, MAL_DETAIL_RAW_REPR_FIELD)
#endif

#ifndef MAL_DETAIL_HOST_REPR_11bb4291f5045bdf_DECLARED
#define MAL_DETAIL_HOST_REPR_11bb4291f5045bdf_DECLARED
typedef struct mal_detail_repr_product_11bb4291f5045bdf mal_repr_product_11bb4291f5045bdf_t;
#endif

#ifndef MAL_DETAIL_HOST_REPR_11bb4291f5045bdf_DEFINED
#define MAL_DETAIL_HOST_REPR_11bb4291f5045bdf_DEFINED
MAL_DETAIL_DEFINE_PRODUCT_REPR(mal_detail_repr_product_11bb4291f5045bdf, MAL_DETAIL_REPR_FIELDS_11bb4291f5045bdf, MAL_DETAIL_HOST_REPR_FIELD)
#endif

/* Type helpers */

#ifndef MAL_DETAIL_HOST_REPR_11bb4291f5045bdf_HELPERS
#define MAL_DETAIL_HOST_REPR_11bb4291f5045bdf_HELPERS
MAL_DETAIL_DEFINE_PRODUCT_CONVERSIONS(mal_detail_to_host_11bb4291f5045bdf, mal_repr_product_11bb4291f5045bdf_return, MalRepr_Product_11bb4291f5045bdf, mal_repr_product_11bb4291f5045bdf_t, MAL_DETAIL_REPR_FIELDS_11bb4291f5045bdf)
#endif

/* External operations */

MalType_Int32 mal_ext_inspect(MalContext *context, MalType_UInt64 argument_0, MalType_UInt64 argument_1, MalType_UInt64 argument_2, MalType_Float32 argument_3, MalType_Float32 argument_4, MalType_Float64 argument_5, MalType_Int64 argument_6);

/* External definition helpers */

#define MAL_HAS_EXTERN_inspect 1
#define MAL_DEFINE_inspect(call, value) \
static MalType_Int32 mal_detail_inspect(mal_call_t *call, mal_repr_product_11bb4291f5045bdf_t value); \
MalType_Int32 mal_ext_inspect(MalContext *context MAL_DETAIL_MAYBE_UNUSED, MalType_UInt64 argument_0, MalType_UInt64 argument_1, MalType_UInt64 argument_2, MalType_Float32 argument_3, MalType_Float32 argument_4, MalType_Float64 argument_5, MalType_Int64 argument_6) { \
    mal_call_t call = (mal_call_t){ .mal_detail_context = context }; \
    return mal_detail_inspect(&call, mal_detail_to_host_11bb4291f5045bdf(&call, (MalRepr_Product_11bb4291f5045bdf){ .field_0 = argument_0, .field_1 = argument_1, .field_2 = argument_2, .field_3 = argument_3, .field_4 = argument_4, .field_5 = argument_5, .field_6 = argument_6 })); \
} \
static MalType_Int32 mal_detail_inspect( \
    mal_call_t *call, \
    mal_repr_product_11bb4291f5045bdf_t value \
)

#endif
#endif
