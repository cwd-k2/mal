#ifndef MAL_BUILD_UMBRELLA
#include <mal.h>

_Static_assert(MAL_C_ABI_VERSION == 0x000900u, "generated header requires mal C ABI 0x000900");
_Static_assert((sizeof((size_t)0) * CHAR_BIT) == 64, "size_t does not match the mal target pointer index width");

#ifndef MAL_GENERATED_INTERFACE_97344E7215A6EE6D_H
#define MAL_GENERATED_INTERFACE_97344E7215A6EE6D_H
/* Host-visible types */

typedef struct { uintptr_t bits; } MalType_Allocator;

#ifndef MAL_DETAIL_RAW_REPR_4658c025c85e4628_DECLARED
#define MAL_DETAIL_RAW_REPR_4658c025c85e4628_DECLARED
typedef struct MalRepr_Sum_4658c025c85e4628 MalRepr_Sum_4658c025c85e4628;
#endif
#ifndef MAL_DETAIL_RAW_REPR_917e59f57ce60d07_DECLARED
#define MAL_DETAIL_RAW_REPR_917e59f57ce60d07_DECLARED
typedef struct MalRepr_Product_917e59f57ce60d07 MalRepr_Product_917e59f57ce60d07;
#endif

#ifndef MAL_DETAIL_REPR_FIELDS_4658c025c85e4628_DEFINED
#define MAL_DETAIL_REPR_FIELDS_4658c025c85e4628_DEFINED
#define MAL_DETAIL_REPR_FIELDS_4658c025c85e4628(field, context) \
field(context, 0, variant_0, MalType_Address, mal_Address_t, mal_Address_return, mal_Address_return) \
field(context, 1, variant_1, MalType_Unit, mal_Unit_t, mal_detail_convert_Unit, mal_detail_convert_Unit)
#endif
#ifndef MAL_DETAIL_RAW_REPR_4658c025c85e4628_DEFINED
#define MAL_DETAIL_RAW_REPR_4658c025c85e4628_DEFINED
MAL_DETAIL_DEFINE_SUM_REPR(MalRepr_Sum_4658c025c85e4628, MAL_DETAIL_REPR_FIELDS_4658c025c85e4628, MAL_DETAIL_RAW_REPR_FIELD)

#endif

#ifndef MAL_DETAIL_REPR_FIELDS_917e59f57ce60d07_DEFINED
#define MAL_DETAIL_REPR_FIELDS_917e59f57ce60d07_DEFINED
#define MAL_DETAIL_REPR_FIELDS_917e59f57ce60d07(field, context) \
field(context, 0, field_0, MalType_Allocator, mal_Allocator_t, mal_detail_to_host_Allocator, mal_Allocator_return) \
field(context, 1, field_1, MalType_Address, mal_Address_t, MAL_DETAIL_REPR_IDENTITY, mal_Address_return)
#endif
#ifndef MAL_DETAIL_RAW_REPR_917e59f57ce60d07_DEFINED
#define MAL_DETAIL_RAW_REPR_917e59f57ce60d07_DEFINED
MAL_DETAIL_DEFINE_PRODUCT_REPR(MalRepr_Product_917e59f57ce60d07, MAL_DETAIL_REPR_FIELDS_917e59f57ce60d07, MAL_DETAIL_RAW_REPR_FIELD)

#endif

typedef MalType_Address MalType_NodeAddress;
typedef MalRepr_Sum_4658c025c85e4628 MalType_NodeBuildResult;

typedef struct { uintptr_t mal_detail_bits; } mal_Allocator_t;
#ifndef MAL_DETAIL_HOST_REPR_4658c025c85e4628_DECLARED
#define MAL_DETAIL_HOST_REPR_4658c025c85e4628_DECLARED
typedef struct mal_detail_repr_sum_4658c025c85e4628 mal_repr_sum_4658c025c85e4628_t;
#endif
#ifndef MAL_DETAIL_HOST_REPR_917e59f57ce60d07_DECLARED
#define MAL_DETAIL_HOST_REPR_917e59f57ce60d07_DECLARED
typedef struct mal_detail_repr_product_917e59f57ce60d07 mal_repr_product_917e59f57ce60d07_t;
#endif
#ifndef MAL_DETAIL_HOST_REPR_cc5512c1c1db9776_DECLARED
#define MAL_DETAIL_HOST_REPR_cc5512c1c1db9776_DECLARED
typedef struct mal_detail_repr_product_cc5512c1c1db9776 mal_repr_product_cc5512c1c1db9776_t;
#endif
typedef mal_Address_t mal_NodeAddress_t;
typedef mal_repr_sum_4658c025c85e4628_t mal_NodeBuildResult_t;
typedef mal_repr_product_cc5512c1c1db9776_t mal_NodeRecord_t;

#ifndef MAL_DETAIL_HOST_REPR_4658c025c85e4628_DEFINED
#define MAL_DETAIL_HOST_REPR_4658c025c85e4628_DEFINED
MAL_DETAIL_DEFINE_SUM_REPR(mal_detail_repr_sum_4658c025c85e4628, MAL_DETAIL_REPR_FIELDS_4658c025c85e4628, MAL_DETAIL_HOST_REPR_FIELD)
#endif

#ifndef MAL_DETAIL_HOST_REPR_917e59f57ce60d07_DEFINED
#define MAL_DETAIL_HOST_REPR_917e59f57ce60d07_DEFINED
MAL_DETAIL_DEFINE_PRODUCT_REPR(mal_detail_repr_product_917e59f57ce60d07, MAL_DETAIL_REPR_FIELDS_917e59f57ce60d07, MAL_DETAIL_HOST_REPR_FIELD)
#endif

#ifndef MAL_DETAIL_REPR_FIELDS_cc5512c1c1db9776_DEFINED
#define MAL_DETAIL_REPR_FIELDS_cc5512c1c1db9776_DEFINED
#define MAL_DETAIL_REPR_FIELDS_cc5512c1c1db9776(field, context) \
field(context, 0, field_0, MalType_Int32, mal_Int32_t, MAL_DETAIL_REPR_IDENTITY, MAL_DETAIL_REPR_IDENTITY) \
field(context, 1, field_1, MalType_UInt8, mal_UInt8_t, MAL_DETAIL_REPR_IDENTITY, MAL_DETAIL_REPR_IDENTITY) \
field(context, 2, field_2, MalType_Address, mal_Address_t, MAL_DETAIL_REPR_IDENTITY, mal_Address_return) \
field(context, 3, field_3, MalType_Address, mal_Address_t, MAL_DETAIL_REPR_IDENTITY, mal_Address_return)
#endif
#ifndef MAL_DETAIL_HOST_REPR_cc5512c1c1db9776_DEFINED
#define MAL_DETAIL_HOST_REPR_cc5512c1c1db9776_DEFINED
MAL_DETAIL_DEFINE_PRODUCT_REPR(mal_detail_repr_product_cc5512c1c1db9776, MAL_DETAIL_REPR_FIELDS_cc5512c1c1db9776, MAL_DETAIL_HOST_REPR_FIELD)
#endif

/* Type helpers */

static inline mal_Allocator_t mal_detail_to_host_Allocator(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, MalType_Allocator value);
static inline MalType_Allocator mal_Allocator_return(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, mal_Allocator_t value);

#ifndef MAL_DETAIL_HOST_REPR_4658c025c85e4628_HELPERS
#define MAL_DETAIL_HOST_REPR_4658c025c85e4628_HELPERS
MAL_DETAIL_DEFINE_SUM_CONVERSIONS(mal_detail_to_host_4658c025c85e4628, mal_detail_to_raw_4658c025c85e4628, MalRepr_Sum_4658c025c85e4628, mal_repr_sum_4658c025c85e4628_t, MAL_DETAIL_REPR_FIELDS_4658c025c85e4628)

#define mal_repr_sum_4658c025c85e4628_tag_0 UINT32_C(0)
#define mal_repr_sum_4658c025c85e4628_tag_1 UINT32_C(1)
#define MAL_DETAIL_SUM_API_repr_sum_4658c025c85e4628(unit, value) \
value(mal_repr_sum_4658c025c85e4628_make_0, mal_repr_sum_4658c025c85e4628_return_0, mal_repr_sum_4658c025c85e4628_t, MalRepr_Sum_4658c025c85e4628, mal_Address_t, mal_repr_sum_4658c025c85e4628_tag_0, variant_0, mal_detail_to_raw_4658c025c85e4628) \
unit(mal_repr_sum_4658c025c85e4628_make_1, mal_repr_sum_4658c025c85e4628_return_1, mal_repr_sum_4658c025c85e4628_t, MalRepr_Sum_4658c025c85e4628, mal_repr_sum_4658c025c85e4628_tag_1, variant_1, mal_detail_to_raw_4658c025c85e4628)
MAL_DETAIL_SUM_API_repr_sum_4658c025c85e4628(MAL_DETAIL_DEFINE_SUM_UNIT_API, MAL_DETAIL_DEFINE_SUM_VALUE_API)
#endif

#ifndef MAL_DETAIL_HOST_REPR_917e59f57ce60d07_HELPERS
#define MAL_DETAIL_HOST_REPR_917e59f57ce60d07_HELPERS
MAL_DETAIL_DEFINE_PRODUCT_CONVERSIONS(mal_detail_to_host_917e59f57ce60d07, mal_repr_product_917e59f57ce60d07_return, MalRepr_Product_917e59f57ce60d07, mal_repr_product_917e59f57ce60d07_t, MAL_DETAIL_REPR_FIELDS_917e59f57ce60d07)
#endif

static inline mal_Allocator_t mal_Allocator_from_bits(uintptr_t bits) {
    return (mal_Allocator_t){ .mal_detail_bits = bits };
}

static inline mal_Allocator_t mal_detail_to_host_Allocator(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, MalType_Allocator value) {
    return mal_Allocator_from_bits(value.bits);
}

static inline uintptr_t mal_Allocator_to_bits(mal_Allocator_t value) {
    return value.mal_detail_bits;
}

static inline MalType_Allocator mal_Allocator_return(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, mal_Allocator_t value) {
    return (MalType_Allocator){ .bits = value.mal_detail_bits };
}

#define MAL_DETAIL_TO_RAW_ALIAS_NodeAddress mal_Address_return(call, value)
MAL_DETAIL_DEFINE_CONVERSION(mal_NodeAddress_return, MalType_NodeAddress, mal_NodeAddress_t, MAL_DETAIL_TO_RAW_ALIAS_NodeAddress)
#define mal_NodeBuildResult_tag_0 UINT32_C(0)
#define mal_NodeBuildResult_tag_1 UINT32_C(1)
#define MAL_DETAIL_SUM_API_NodeBuildResult(unit, value) \
value(mal_NodeBuildResult_make_0, mal_NodeBuildResult_return_0, mal_NodeBuildResult_t, MalType_NodeBuildResult, mal_NodeAddress_t, mal_NodeBuildResult_tag_0, variant_0, mal_detail_to_raw_4658c025c85e4628) \
unit(mal_NodeBuildResult_make_1, mal_NodeBuildResult_return_1, mal_NodeBuildResult_t, MalType_NodeBuildResult, mal_NodeBuildResult_tag_1, variant_1, mal_detail_to_raw_4658c025c85e4628)
MAL_DETAIL_SUM_API_NodeBuildResult(MAL_DETAIL_DEFINE_SUM_UNIT_API, MAL_DETAIL_DEFINE_SUM_VALUE_API)

/* Canonical memory access */

#ifndef MAL_DETAIL_MEMORY_REPR_4658c025c85e4628_HELPERS
#define MAL_DETAIL_MEMORY_REPR_4658c025c85e4628_HELPERS
#define MAL_DETAIL_MEMORY_MEMBERS_4658c025c85e4628(member) \
member(mal_repr_sum_4658c025c85e4628_t, mal_UInt8_t, mal_detail_memory_write_UInt8, 0, variant_0, mal_detail_memory_read_Address, mal_detail_memory_write_Address, 8) \
member(mal_repr_sum_4658c025c85e4628_t, mal_UInt8_t, mal_detail_memory_write_UInt8, 1, variant_1, mal_detail_memory_read_Unit, mal_detail_memory_write_Unit, 8)
MAL_DETAIL_DEFINE_MEMORY_SUM(mal_detail_memory_read_4658c025c85e4628, mal_detail_memory_write_4658c025c85e4628, mal_repr_sum_4658c025c85e4628_t, mal_detail_memory_read_UInt8, MAL_DETAIL_MEMORY_MEMBERS_4658c025c85e4628)
#endif

#ifndef MAL_DETAIL_MEMORY_REPR_cc5512c1c1db9776_HELPERS
#define MAL_DETAIL_MEMORY_REPR_cc5512c1c1db9776_HELPERS
#define MAL_DETAIL_MEMORY_FIELDS_cc5512c1c1db9776(unit, value) \
value(field_0, mal_detail_memory_read_Int32, mal_detail_memory_write_Int32, 0) \
value(field_1, mal_detail_memory_read_UInt8, mal_detail_memory_write_UInt8, 4) \
value(field_2, mal_detail_memory_read_Address, mal_detail_memory_write_Address, 8) \
value(field_3, mal_detail_memory_read_Address, mal_detail_memory_write_Address, 16)
MAL_DETAIL_DEFINE_MEMORY_PRODUCT(mal_detail_memory_read_cc5512c1c1db9776, mal_detail_memory_write_cc5512c1c1db9776, mal_repr_product_cc5512c1c1db9776_t, MAL_DETAIL_MEMORY_FIELDS_cc5512c1c1db9776)
#endif

MAL_DETAIL_DEFINE_MEMORY_ALIAS(mal_NodeAddress_read, mal_NodeAddress_write, mal_NodeAddress_t, 8, mal_detail_memory_read_Address, mal_detail_memory_write_Address)

MAL_DETAIL_DEFINE_MEMORY_ALIAS(mal_NodeBuildResult_read, mal_NodeBuildResult_write, mal_NodeBuildResult_t, 16, mal_detail_memory_read_4658c025c85e4628, mal_detail_memory_write_4658c025c85e4628)

MAL_DETAIL_DEFINE_MEMORY_ALIAS(mal_NodeRecord_read, mal_NodeRecord_write, mal_NodeRecord_t, 24, mal_detail_memory_read_cc5512c1c1db9776, mal_detail_memory_write_cc5512c1c1db9776)

/* External operations */

MalType_Allocator mal_ext_createAllocator(MalContext *context, MalType_USize value);
MalType_NodeBuildResult mal_ext_allocateNode(MalContext *context, MalType_Allocator value);
void mal_ext_releaseNode(MalContext *context, MalType_Allocator argument_0, MalType_NodeAddress argument_1);
void mal_ext_destroyAllocator(MalContext *context, MalType_Allocator value);

/* External definition helpers */

#define MAL_HAS_EXTERN_createAllocator 1
#define MAL_DEFINE_createAllocator(call, value) \
static MalType_Allocator mal_detail_createAllocator(mal_call_t *call, mal_USize_t value); \
MalType_Allocator mal_ext_createAllocator(MalContext *context MAL_DETAIL_MAYBE_UNUSED, MalType_USize value) { \
    mal_call_t call = (mal_call_t){ .mal_detail_context = context }; \
    return mal_detail_createAllocator(&call, value); \
} \
static MalType_Allocator mal_detail_createAllocator( \
    mal_call_t *call, \
    mal_USize_t value \
)

#define MAL_HAS_EXTERN_allocateNode 1
#define MAL_DEFINE_allocateNode(call, value) \
static MalType_NodeBuildResult mal_detail_allocateNode(mal_call_t *call, mal_Allocator_t value); \
MalType_NodeBuildResult mal_ext_allocateNode(MalContext *context MAL_DETAIL_MAYBE_UNUSED, MalType_Allocator value) { \
    mal_call_t call = (mal_call_t){ .mal_detail_context = context }; \
    return mal_detail_allocateNode(&call, (mal_Allocator_t){ .mal_detail_bits = value.bits }); \
} \
static MalType_NodeBuildResult mal_detail_allocateNode( \
    mal_call_t *call, \
    mal_Allocator_t value \
)

#define MAL_HAS_EXTERN_releaseNode 1
#define MAL_DEFINE_releaseNode(call, value) \
static MalType_Unit mal_detail_releaseNode(mal_call_t *call, mal_repr_product_917e59f57ce60d07_t value); \
void mal_ext_releaseNode(MalContext *context MAL_DETAIL_MAYBE_UNUSED, MalType_Allocator argument_0, MalType_NodeAddress argument_1) { \
    mal_call_t call = (mal_call_t){ .mal_detail_context = context }; \
    mal_detail_releaseNode(&call, mal_detail_to_host_917e59f57ce60d07(&call, (MalRepr_Product_917e59f57ce60d07){ .field_0 = argument_0, .field_1 = argument_1 })); \
} \
static MalType_Unit mal_detail_releaseNode( \
    mal_call_t *call, \
    mal_repr_product_917e59f57ce60d07_t value \
)

#define MAL_HAS_EXTERN_destroyAllocator 1
#define MAL_DEFINE_destroyAllocator(call, value) \
static MalType_Unit mal_detail_destroyAllocator(mal_call_t *call, mal_Allocator_t value); \
void mal_ext_destroyAllocator(MalContext *context MAL_DETAIL_MAYBE_UNUSED, MalType_Allocator value) { \
    mal_call_t call = (mal_call_t){ .mal_detail_context = context }; \
    mal_detail_destroyAllocator(&call, (mal_Allocator_t){ .mal_detail_bits = value.bits }); \
} \
static MalType_Unit mal_detail_destroyAllocator( \
    mal_call_t *call, \
    mal_Allocator_t value \
)

#endif
#endif
