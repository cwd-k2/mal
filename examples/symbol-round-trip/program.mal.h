#ifndef MAL_BUILD_UMBRELLA
#include <mal.h>

_Static_assert(MAL_C_ABI_VERSION == 0x000900u, "generated header requires mal C ABI 0x000900");
_Static_assert((sizeof((size_t)0) * CHAR_BIT) == 64, "size_t does not match the mal target pointer index width");

#ifndef MAL_GENERATED_INTERFACE_C880DF65D47096AA_H
#define MAL_GENERATED_INTERFACE_C880DF65D47096AA_H
/* Host-visible types */

#ifndef MAL_DETAIL_RAW_REPR_1e5c20e9f358150e_DECLARED
#define MAL_DETAIL_RAW_REPR_1e5c20e9f358150e_DECLARED
typedef struct MalRepr_Product_1e5c20e9f358150e MalRepr_Product_1e5c20e9f358150e;
#endif

#ifndef MAL_DETAIL_RAW_REPR_1e5c20e9f358150e_DEFINED
#define MAL_DETAIL_RAW_REPR_1e5c20e9f358150e_DEFINED
struct MalRepr_Product_1e5c20e9f358150e {
    MalType_Address field_0;
    MalType_USize field_1;
};

#endif

#ifndef MAL_DETAIL_HOST_REPR_1e5c20e9f358150e_DECLARED
#define MAL_DETAIL_HOST_REPR_1e5c20e9f358150e_DECLARED
typedef struct mal_detail_repr_product_1e5c20e9f358150e mal_repr_product_1e5c20e9f358150e_t;
#endif

#ifndef MAL_DETAIL_HOST_REPR_1e5c20e9f358150e_DEFINED
#define MAL_DETAIL_HOST_REPR_1e5c20e9f358150e_DEFINED
struct mal_detail_repr_product_1e5c20e9f358150e {
    mal_Address_t field_0;
    mal_USize_t field_1;
};
#endif

/* Type helpers */

#ifndef MAL_DETAIL_HOST_REPR_1e5c20e9f358150e_HELPERS
#define MAL_DETAIL_HOST_REPR_1e5c20e9f358150e_HELPERS
static inline MalRepr_Product_1e5c20e9f358150e mal_repr_product_1e5c20e9f358150e_return(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, mal_repr_product_1e5c20e9f358150e_t value) {
    return (MalRepr_Product_1e5c20e9f358150e){ .field_0 = mal_Address_return(call, value.field_0), .field_1 = value.field_1 };
}

#endif

/* External operations */

MalType_Address mal_ext_transferBuffer(MalContext *context);
MalType_USize mal_ext_receive(MalContext *context, MalType_Address argument_0, MalType_USize argument_1);
void mal_ext_send(MalContext *context, MalType_Address argument_0, MalType_USize argument_1);

/* External definition helpers */

#define MAL_HAS_EXTERN_transferBuffer 1
#define MAL_DEFINE_transferBuffer(call) \
static MalType_Address mal_detail_transferBuffer(mal_call_t *call); \
MalType_Address mal_ext_transferBuffer(MalContext *context MAL_DETAIL_MAYBE_UNUSED) { \
    mal_call_t call = (mal_call_t){ .mal_detail_context = context }; \
    return mal_detail_transferBuffer(&call); \
} \
static MalType_Address mal_detail_transferBuffer( \
    mal_call_t *call \
)

#define MAL_HAS_EXTERN_receive 1
#define MAL_DEFINE_receive(call, value) \
static MalType_USize mal_detail_receive(mal_call_t *call, mal_repr_product_1e5c20e9f358150e_t value); \
MalType_USize mal_ext_receive(MalContext *context MAL_DETAIL_MAYBE_UNUSED, MalType_Address argument_0, MalType_USize argument_1) { \
    mal_call_t call = (mal_call_t){ .mal_detail_context = context }; \
    return mal_detail_receive(&call, (mal_repr_product_1e5c20e9f358150e_t){ .field_0 = ((MalRepr_Product_1e5c20e9f358150e){ .field_0 = argument_0, .field_1 = argument_1 }).field_0, .field_1 = ((MalRepr_Product_1e5c20e9f358150e){ .field_0 = argument_0, .field_1 = argument_1 }).field_1 }); \
} \
static MalType_USize mal_detail_receive( \
    mal_call_t *call, \
    mal_repr_product_1e5c20e9f358150e_t value \
)

#define MAL_HAS_EXTERN_send 1
#define MAL_DEFINE_send(call, value) \
static MalType_Unit mal_detail_send(mal_call_t *call, mal_repr_product_1e5c20e9f358150e_t value); \
void mal_ext_send(MalContext *context MAL_DETAIL_MAYBE_UNUSED, MalType_Address argument_0, MalType_USize argument_1) { \
    mal_call_t call = (mal_call_t){ .mal_detail_context = context }; \
    mal_detail_send(&call, (mal_repr_product_1e5c20e9f358150e_t){ .field_0 = ((MalRepr_Product_1e5c20e9f358150e){ .field_0 = argument_0, .field_1 = argument_1 }).field_0, .field_1 = ((MalRepr_Product_1e5c20e9f358150e){ .field_0 = argument_0, .field_1 = argument_1 }).field_1 }); \
} \
static MalType_Unit mal_detail_send( \
    mal_call_t *call, \
    mal_repr_product_1e5c20e9f358150e_t value \
)

#endif
#endif
