#ifndef MAL_PROGRAM_MAL_H
#define MAL_PROGRAM_MAL_H

#include <mal.h>

_Static_assert(MAL_C_ABI_VERSION == 0x000900u, "generated header requires mal C ABI 0x000900");
_Static_assert((sizeof((size_t)0) * CHAR_BIT) == 64, "size_t does not match the mal target pointer index width");

/* Host-visible types */

typedef struct MalRepr_Product_6a09db6e50421494 MalRepr_Product_6a09db6e50421494;

struct MalRepr_Product_6a09db6e50421494 {
    MalType_Float32 field_0;
    MalType_Float32 field_1;
    MalType_Float64 field_2;
    MalType_Int64 field_3;
};

typedef struct mal_detail_repr_product_6a09db6e50421494 mal_repr_product_6a09db6e50421494_t;

struct mal_detail_repr_product_6a09db6e50421494 {
    mal_Float32_t field_0;
    mal_Float32_t field_1;
    mal_Float64_t field_2;
    mal_Int64_t field_3;
};

/* Type helpers */

static inline MalRepr_Product_6a09db6e50421494 mal_repr_product_6a09db6e50421494_return(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, mal_repr_product_6a09db6e50421494_t value) {
    return (MalRepr_Product_6a09db6e50421494){ .field_0 = value.field_0, .field_1 = value.field_1, .field_2 = value.field_2, .field_3 = value.field_3 };
}

/* External operations */

MalType_Int32 mal_ext_inspect(MalContext *context, MalType_Float32 argument_0, MalType_Float32 argument_1, MalType_Float64 argument_2, MalType_Int64 argument_3);

/* External definition helpers */

#define MAL_HAS_EXTERN_inspect 1
#define MAL_DEFINE_inspect(call, value) \
static MalType_Int32 mal_detail_inspect(mal_call_t *call, mal_repr_product_6a09db6e50421494_t value); \
MalType_Int32 mal_ext_inspect(MalContext *context MAL_DETAIL_MAYBE_UNUSED, MalType_Float32 argument_0, MalType_Float32 argument_1, MalType_Float64 argument_2, MalType_Int64 argument_3) { \
    mal_call_t call = (mal_call_t){ .mal_detail_context = context }; \
    return mal_detail_inspect(&call, (mal_repr_product_6a09db6e50421494_t){ .field_0 = ((MalRepr_Product_6a09db6e50421494){ .field_0 = argument_0, .field_1 = argument_1, .field_2 = argument_2, .field_3 = argument_3 }).field_0, .field_1 = ((MalRepr_Product_6a09db6e50421494){ .field_0 = argument_0, .field_1 = argument_1, .field_2 = argument_2, .field_3 = argument_3 }).field_1, .field_2 = ((MalRepr_Product_6a09db6e50421494){ .field_0 = argument_0, .field_1 = argument_1, .field_2 = argument_2, .field_3 = argument_3 }).field_2, .field_3 = ((MalRepr_Product_6a09db6e50421494){ .field_0 = argument_0, .field_1 = argument_1, .field_2 = argument_2, .field_3 = argument_3 }).field_3 }); \
} \
static MalType_Int32 mal_detail_inspect( \
    mal_call_t *call, \
    mal_repr_product_6a09db6e50421494_t value \
)

#endif
