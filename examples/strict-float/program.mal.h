#ifndef MAL_BUILD_UMBRELLA
#include <mal.h>

_Static_assert(MAL_C_ABI_VERSION == 0x000900u, "generated header requires mal C ABI 0x000900");
_Static_assert((sizeof((size_t)0) * CHAR_BIT) == 64, "size_t does not match the mal target pointer index width");

#ifndef MAL_GENERATED_INTERFACE_96BFF055B87C9C2D_H
#define MAL_GENERATED_INTERFACE_96BFF055B87C9C2D_H
/* Host-visible types */

#ifndef MAL_DETAIL_RAW_REPR_6a09db6e50421494_DECLARED
#define MAL_DETAIL_RAW_REPR_6a09db6e50421494_DECLARED
typedef struct MalRepr_Product_6a09db6e50421494 MalRepr_Product_6a09db6e50421494;
#endif

#ifndef MAL_DETAIL_REPR_FIELDS_6a09db6e50421494_DEFINED
#define MAL_DETAIL_REPR_FIELDS_6a09db6e50421494_DEFINED
#define MAL_DETAIL_REPR_FIELDS_6a09db6e50421494(field) \
field(field_0, MalType_Float32, mal_Float32_t) \
field(field_1, MalType_Float32, mal_Float32_t) \
field(field_2, MalType_Float64, mal_Float64_t) \
field(field_3, MalType_Int64, mal_Int64_t)
#endif
#ifndef MAL_DETAIL_RAW_REPR_6a09db6e50421494_DEFINED
#define MAL_DETAIL_RAW_REPR_6a09db6e50421494_DEFINED
MAL_DETAIL_DEFINE_PRODUCT_REPR(MalRepr_Product_6a09db6e50421494, MAL_DETAIL_REPR_FIELDS_6a09db6e50421494, MAL_DETAIL_RAW_REPR_FIELD)

#endif

#ifndef MAL_DETAIL_HOST_REPR_6a09db6e50421494_DECLARED
#define MAL_DETAIL_HOST_REPR_6a09db6e50421494_DECLARED
typedef struct mal_detail_repr_product_6a09db6e50421494 mal_repr_product_6a09db6e50421494_t;
#endif

#ifndef MAL_DETAIL_HOST_REPR_6a09db6e50421494_DEFINED
#define MAL_DETAIL_HOST_REPR_6a09db6e50421494_DEFINED
MAL_DETAIL_DEFINE_PRODUCT_REPR(mal_detail_repr_product_6a09db6e50421494, MAL_DETAIL_REPR_FIELDS_6a09db6e50421494, MAL_DETAIL_HOST_REPR_FIELD)
#endif

/* Type helpers */

#ifndef MAL_DETAIL_HOST_REPR_6a09db6e50421494_HELPERS
#define MAL_DETAIL_HOST_REPR_6a09db6e50421494_HELPERS
#define MAL_DETAIL_TO_HOST_6a09db6e50421494 (mal_repr_product_6a09db6e50421494_t){ .field_0 = value.field_0, .field_1 = value.field_1, .field_2 = value.field_2, .field_3 = value.field_3 }
MAL_DETAIL_DEFINE_CONVERSION(mal_detail_to_host_6a09db6e50421494, mal_repr_product_6a09db6e50421494_t, MalRepr_Product_6a09db6e50421494, MAL_DETAIL_TO_HOST_6a09db6e50421494)
#define MAL_DETAIL_TO_RAW_6a09db6e50421494 (MalRepr_Product_6a09db6e50421494){ .field_0 = value.field_0, .field_1 = value.field_1, .field_2 = value.field_2, .field_3 = value.field_3 }
MAL_DETAIL_DEFINE_CONVERSION(mal_repr_product_6a09db6e50421494_return, MalRepr_Product_6a09db6e50421494, mal_repr_product_6a09db6e50421494_t, MAL_DETAIL_TO_RAW_6a09db6e50421494)
#endif

/* External operations */

MalType_Int32 mal_ext_inspect(MalContext *context, MalType_Float32 argument_0, MalType_Float32 argument_1, MalType_Float64 argument_2, MalType_Int64 argument_3);

/* External definition helpers */

#define MAL_HAS_EXTERN_inspect 1
#define MAL_DEFINE_inspect(call, value) \
static MalType_Int32 mal_detail_inspect(mal_call_t *call, mal_repr_product_6a09db6e50421494_t value); \
MalType_Int32 mal_ext_inspect(MalContext *context MAL_DETAIL_MAYBE_UNUSED, MalType_Float32 argument_0, MalType_Float32 argument_1, MalType_Float64 argument_2, MalType_Int64 argument_3) { \
    mal_call_t call = (mal_call_t){ .mal_detail_context = context }; \
    return mal_detail_inspect(&call, mal_detail_to_host_6a09db6e50421494(&call, (MalRepr_Product_6a09db6e50421494){ .field_0 = argument_0, .field_1 = argument_1, .field_2 = argument_2, .field_3 = argument_3 })); \
} \
static MalType_Int32 mal_detail_inspect( \
    mal_call_t *call, \
    mal_repr_product_6a09db6e50421494_t value \
)

#endif
#endif
