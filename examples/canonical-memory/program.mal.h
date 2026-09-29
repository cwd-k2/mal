#ifndef MAL_BUILD_UMBRELLA
#include <mal.h>

_Static_assert(MAL_C_ABI_VERSION == 0x000900u, "generated header requires mal C ABI 0x000900");
_Static_assert((sizeof((size_t)0) * CHAR_BIT) == 64, "size_t does not match the mal target pointer index width");

#ifndef MAL_GENERATED_INTERFACE_D6F6E9A828DAFF5D_H
#define MAL_GENERATED_INTERFACE_D6F6E9A828DAFF5D_H

/* Host-visible types */

#ifndef MAL_DETAIL_RAW_REPR_1e5c20e9f358150e_DECLARED
#define MAL_DETAIL_RAW_REPR_1e5c20e9f358150e_DECLARED
typedef struct MalRepr_Product_1e5c20e9f358150e MalRepr_Product_1e5c20e9f358150e;
#endif

#ifndef MAL_DETAIL_REPR_FIELDS_1e5c20e9f358150e_DEFINED
#define MAL_DETAIL_REPR_FIELDS_1e5c20e9f358150e_DEFINED
#define MAL_DETAIL_REPR_FIELDS_1e5c20e9f358150e(field, context) \
field(context, 0, field_0, MalType_Address, mal_Address_t, MAL_DETAIL_REPR_IDENTITY, mal_Address_return) \
field(context, 1, field_1, MalType_USize, mal_USize_t, MAL_DETAIL_REPR_IDENTITY, MAL_DETAIL_REPR_IDENTITY)
#endif
#ifndef MAL_DETAIL_RAW_REPR_1e5c20e9f358150e_DEFINED
#define MAL_DETAIL_RAW_REPR_1e5c20e9f358150e_DEFINED
MAL_DETAIL_DEFINE_PRODUCT_REPR(MalRepr_Product_1e5c20e9f358150e, MAL_DETAIL_REPR_FIELDS_1e5c20e9f358150e, MAL_DETAIL_RAW_REPR_FIELD)
#endif

typedef MalRepr_Product_1e5c20e9f358150e MalType_Storage;

#ifndef MAL_DETAIL_HOST_REPR_1e5c20e9f358150e_DECLARED
#define MAL_DETAIL_HOST_REPR_1e5c20e9f358150e_DECLARED
typedef struct mal_detail_repr_product_1e5c20e9f358150e mal_repr_product_1e5c20e9f358150e_t;
#endif
#ifndef MAL_DETAIL_HOST_REPR_1e36b8e9f3384819_DECLARED
#define MAL_DETAIL_HOST_REPR_1e36b8e9f3384819_DECLARED
typedef struct mal_detail_repr_product_1e36b8e9f3384819 mal_repr_product_1e36b8e9f3384819_t;
#endif
typedef mal_repr_product_1e36b8e9f3384819_t mal_Sample_t;
typedef mal_repr_product_1e5c20e9f358150e_t mal_Storage_t;

#ifndef MAL_DETAIL_HOST_REPR_1e5c20e9f358150e_DEFINED
#define MAL_DETAIL_HOST_REPR_1e5c20e9f358150e_DEFINED
MAL_DETAIL_DEFINE_PRODUCT_REPR(mal_detail_repr_product_1e5c20e9f358150e, MAL_DETAIL_REPR_FIELDS_1e5c20e9f358150e, MAL_DETAIL_HOST_REPR_FIELD)
#endif

#ifndef MAL_DETAIL_REPR_FIELDS_1e36b8e9f3384819_DEFINED
#define MAL_DETAIL_REPR_FIELDS_1e36b8e9f3384819_DEFINED
#define MAL_DETAIL_REPR_FIELDS_1e36b8e9f3384819(field, context) \
field(context, 0, field_0, MalType_Int64, mal_Int64_t, MAL_DETAIL_REPR_IDENTITY, MAL_DETAIL_REPR_IDENTITY) \
field(context, 1, field_1, MalType_UInt8, mal_UInt8_t, MAL_DETAIL_REPR_IDENTITY, MAL_DETAIL_REPR_IDENTITY)
#endif
#ifndef MAL_DETAIL_HOST_REPR_1e36b8e9f3384819_DEFINED
#define MAL_DETAIL_HOST_REPR_1e36b8e9f3384819_DEFINED
MAL_DETAIL_DEFINE_PRODUCT_REPR(mal_detail_repr_product_1e36b8e9f3384819, MAL_DETAIL_REPR_FIELDS_1e36b8e9f3384819, MAL_DETAIL_HOST_REPR_FIELD)
#endif

/* Type helpers */

#ifndef MAL_DETAIL_HOST_REPR_1e5c20e9f358150e_HELPERS
#define MAL_DETAIL_HOST_REPR_1e5c20e9f358150e_HELPERS
MAL_DETAIL_DEFINE_PRODUCT_CONVERSIONS(mal_detail_to_host_1e5c20e9f358150e, mal_repr_product_1e5c20e9f358150e_return, MalRepr_Product_1e5c20e9f358150e, mal_repr_product_1e5c20e9f358150e_t, MAL_DETAIL_REPR_FIELDS_1e5c20e9f358150e)
#endif

MAL_DETAIL_DEFINE_CONVERTING_RETURN(mal_Storage_return, MalType_Storage, mal_Storage_t, mal_repr_product_1e5c20e9f358150e_return)

/* Canonical memory access */

#ifndef MAL_DETAIL_MEMORY_REPR_1e5c20e9f358150e_HELPERS
#define MAL_DETAIL_MEMORY_REPR_1e5c20e9f358150e_HELPERS
#define MAL_DETAIL_MEMORY_FIELDS_1e5c20e9f358150e(unit, value) \
value(field_0, mal_detail_memory_read_Address, mal_detail_memory_write_Address, 0) \
value(field_1, mal_detail_memory_read_USize, mal_detail_memory_write_USize, 8)
MAL_DETAIL_DEFINE_MEMORY_PRODUCT(mal_detail_memory_read_1e5c20e9f358150e, mal_detail_memory_write_1e5c20e9f358150e, mal_repr_product_1e5c20e9f358150e_t, MAL_DETAIL_MEMORY_FIELDS_1e5c20e9f358150e)
#endif

#ifndef MAL_DETAIL_MEMORY_REPR_1e36b8e9f3384819_HELPERS
#define MAL_DETAIL_MEMORY_REPR_1e36b8e9f3384819_HELPERS
#define MAL_DETAIL_MEMORY_FIELDS_1e36b8e9f3384819(unit, value) \
value(field_0, mal_detail_memory_read_Int64, mal_detail_memory_write_Int64, 0) \
value(field_1, mal_detail_memory_read_UInt8, mal_detail_memory_write_UInt8, 8)
MAL_DETAIL_DEFINE_MEMORY_PRODUCT(mal_detail_memory_read_1e36b8e9f3384819, mal_detail_memory_write_1e36b8e9f3384819, mal_repr_product_1e36b8e9f3384819_t, MAL_DETAIL_MEMORY_FIELDS_1e36b8e9f3384819)
#endif

MAL_DETAIL_DEFINE_MEMORY_ALIAS(mal_Sample_read, mal_Sample_write, mal_Sample_t, 16, mal_detail_memory_read_1e36b8e9f3384819, mal_detail_memory_write_1e36b8e9f3384819)
MAL_DETAIL_DEFINE_MEMORY_ALIAS(mal_Storage_read, mal_Storage_write, mal_Storage_t, 16, mal_detail_memory_read_1e5c20e9f358150e, mal_detail_memory_write_1e5c20e9f358150e)

/* External operations */

MalType_Storage mal_ext_sampleStorage(MalContext *context);
void mal_ext_incrementSample(MalContext *context, MalType_Address argument_0, MalType_USize argument_1);

/* External definition helpers */

#define MAL_HAS_EXTERN_sampleStorage 1
#define MAL_DEFINE_sampleStorage(call) \
static MalType_Storage mal_detail_sampleStorage(mal_call_t *call); \
MalType_Storage mal_ext_sampleStorage(MalContext *context MAL_DETAIL_MAYBE_UNUSED) { \
    mal_call_t call = (mal_call_t){ .mal_detail_context = context }; \
    return mal_detail_sampleStorage(&call); \
} \
static MalType_Storage mal_detail_sampleStorage( \
    mal_call_t *call \
)

#define MAL_HAS_EXTERN_incrementSample 1
#define MAL_DEFINE_incrementSample(call, value) \
static MalType_Unit mal_detail_incrementSample(mal_call_t *call, mal_repr_product_1e5c20e9f358150e_t value); \
void mal_ext_incrementSample(MalContext *context MAL_DETAIL_MAYBE_UNUSED, MalType_Address argument_0, MalType_USize argument_1) { \
    mal_call_t call = (mal_call_t){ .mal_detail_context = context }; \
    mal_detail_incrementSample(&call, mal_detail_to_host_1e5c20e9f358150e(&call, (MalRepr_Product_1e5c20e9f358150e){ .field_0 = argument_0, .field_1 = argument_1 })); \
} \
static MalType_Unit mal_detail_incrementSample( \
    mal_call_t *call, \
    mal_repr_product_1e5c20e9f358150e_t value \
)

#endif
#endif
