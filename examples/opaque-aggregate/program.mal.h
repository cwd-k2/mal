#ifndef MAL_BUILD_UMBRELLA
#include <mal.h>

_Static_assert(MAL_C_ABI_VERSION == 0x000900u, "generated header requires mal C ABI 0x000900");
_Static_assert((sizeof((size_t)0) * CHAR_BIT) == 64, "size_t does not match the mal target pointer index width");

#ifndef MAL_GENERATED_INTERFACE_4916484C108ABEB0_H
#define MAL_GENERATED_INTERFACE_4916484C108ABEB0_H
/* Host-visible types */

typedef struct { uintptr_t bits; } MalType_Allocation;

#ifndef MAL_DETAIL_RAW_REPR_01ed5581230a2b12_DECLARED
#define MAL_DETAIL_RAW_REPR_01ed5581230a2b12_DECLARED
typedef struct MalRepr_Product_01ed5581230a2b12 MalRepr_Product_01ed5581230a2b12;
#endif
#ifndef MAL_DETAIL_RAW_REPR_e70fcef055657787_DECLARED
#define MAL_DETAIL_RAW_REPR_e70fcef055657787_DECLARED
typedef struct MalRepr_Sum_e70fcef055657787 MalRepr_Sum_e70fcef055657787;
#endif

#ifndef MAL_DETAIL_REPR_FIELDS_01ed5581230a2b12_DEFINED
#define MAL_DETAIL_REPR_FIELDS_01ed5581230a2b12_DEFINED
#define MAL_DETAIL_REPR_FIELDS_01ed5581230a2b12(field) \
field(field_0, MalType_Allocation, mal_Allocation_t) \
field(field_1, MalType_USize, mal_USize_t)
#endif
#ifndef MAL_DETAIL_RAW_REPR_01ed5581230a2b12_DEFINED
#define MAL_DETAIL_RAW_REPR_01ed5581230a2b12_DEFINED
MAL_DETAIL_DEFINE_PRODUCT_REPR(MalRepr_Product_01ed5581230a2b12, MAL_DETAIL_REPR_FIELDS_01ed5581230a2b12, MAL_DETAIL_RAW_REPR_FIELD)

#endif

#ifndef MAL_DETAIL_REPR_FIELDS_e70fcef055657787_DEFINED
#define MAL_DETAIL_REPR_FIELDS_e70fcef055657787_DEFINED
#define MAL_DETAIL_REPR_FIELDS_e70fcef055657787(field) \
field(variant_0, MalType_Unit, mal_Unit_t) \
field(variant_1, MalRepr_Product_01ed5581230a2b12, mal_repr_product_01ed5581230a2b12_t)
#endif
#ifndef MAL_DETAIL_RAW_REPR_e70fcef055657787_DEFINED
#define MAL_DETAIL_RAW_REPR_e70fcef055657787_DEFINED
MAL_DETAIL_DEFINE_SUM_REPR(MalRepr_Sum_e70fcef055657787, MAL_DETAIL_REPR_FIELDS_e70fcef055657787, MAL_DETAIL_RAW_REPR_FIELD)

#endif

typedef MalRepr_Sum_e70fcef055657787 MalType_ResizeResult;

typedef struct { uintptr_t mal_detail_bits; } mal_Allocation_t;
#ifndef MAL_DETAIL_HOST_REPR_01ed5581230a2b12_DECLARED
#define MAL_DETAIL_HOST_REPR_01ed5581230a2b12_DECLARED
typedef struct mal_detail_repr_product_01ed5581230a2b12 mal_repr_product_01ed5581230a2b12_t;
#endif
#ifndef MAL_DETAIL_HOST_REPR_e70fcef055657787_DECLARED
#define MAL_DETAIL_HOST_REPR_e70fcef055657787_DECLARED
typedef struct mal_detail_repr_sum_e70fcef055657787 mal_repr_sum_e70fcef055657787_t;
#endif
typedef mal_repr_sum_e70fcef055657787_t mal_ResizeResult_t;

#ifndef MAL_DETAIL_HOST_REPR_01ed5581230a2b12_DEFINED
#define MAL_DETAIL_HOST_REPR_01ed5581230a2b12_DEFINED
MAL_DETAIL_DEFINE_PRODUCT_REPR(mal_detail_repr_product_01ed5581230a2b12, MAL_DETAIL_REPR_FIELDS_01ed5581230a2b12, MAL_DETAIL_HOST_REPR_FIELD)
#endif

#ifndef MAL_DETAIL_HOST_REPR_e70fcef055657787_DEFINED
#define MAL_DETAIL_HOST_REPR_e70fcef055657787_DEFINED
MAL_DETAIL_DEFINE_SUM_REPR(mal_detail_repr_sum_e70fcef055657787, MAL_DETAIL_REPR_FIELDS_e70fcef055657787, MAL_DETAIL_HOST_REPR_FIELD)
#endif

/* Type helpers */

static inline mal_Allocation_t mal_detail_to_host_Allocation(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, MalType_Allocation value);
static inline MalType_Allocation mal_Allocation_return(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, mal_Allocation_t value);

#ifndef MAL_DETAIL_HOST_REPR_01ed5581230a2b12_HELPERS
#define MAL_DETAIL_HOST_REPR_01ed5581230a2b12_HELPERS
#define MAL_DETAIL_TO_HOST_01ed5581230a2b12 (mal_repr_product_01ed5581230a2b12_t){ .field_0 = (mal_Allocation_t){ .mal_detail_bits = value.field_0.bits }, .field_1 = value.field_1 }
MAL_DETAIL_DEFINE_CONVERSION(mal_detail_to_host_01ed5581230a2b12, mal_repr_product_01ed5581230a2b12_t, MalRepr_Product_01ed5581230a2b12, MAL_DETAIL_TO_HOST_01ed5581230a2b12)
#define MAL_DETAIL_TO_RAW_01ed5581230a2b12 (MalRepr_Product_01ed5581230a2b12){ .field_0 = (MalType_Allocation){ .bits = value.field_0.mal_detail_bits }, .field_1 = value.field_1 }
MAL_DETAIL_DEFINE_CONVERSION(mal_repr_product_01ed5581230a2b12_return, MalRepr_Product_01ed5581230a2b12, mal_repr_product_01ed5581230a2b12_t, MAL_DETAIL_TO_RAW_01ed5581230a2b12)
#endif

#ifndef MAL_DETAIL_HOST_REPR_e70fcef055657787_HELPERS
#define MAL_DETAIL_HOST_REPR_e70fcef055657787_HELPERS
#define MAL_DETAIL_TO_HOST_MEMBERS_e70fcef055657787(case, result_type) \
case(result_type, 0, variant_0, mal_detail_convert_Unit) \
case(result_type, 1, variant_1, mal_detail_to_host_01ed5581230a2b12)
#define MAL_DETAIL_TO_RAW_MEMBERS_e70fcef055657787(case, result_type) \
case(result_type, 0, variant_0, mal_detail_convert_Unit) \
case(result_type, 1, variant_1, mal_repr_product_01ed5581230a2b12_return)
MAL_DETAIL_DEFINE_SUM_CONVERSIONS(mal_detail_to_host_e70fcef055657787, mal_detail_to_raw_e70fcef055657787, MalRepr_Sum_e70fcef055657787, mal_repr_sum_e70fcef055657787_t, MAL_DETAIL_TO_HOST_MEMBERS_e70fcef055657787, MAL_DETAIL_TO_RAW_MEMBERS_e70fcef055657787)

#define mal_repr_sum_e70fcef055657787_tag_0 UINT32_C(0)
#define mal_repr_sum_e70fcef055657787_tag_1 UINT32_C(1)
#define MAL_DETAIL_SUM_API_repr_sum_e70fcef055657787(unit, value) \
unit(mal_repr_sum_e70fcef055657787_make_0, mal_repr_sum_e70fcef055657787_return_0, mal_repr_sum_e70fcef055657787_t, MalRepr_Sum_e70fcef055657787, mal_repr_sum_e70fcef055657787_tag_0, variant_0, mal_detail_to_raw_e70fcef055657787) \
value(mal_repr_sum_e70fcef055657787_make_1, mal_repr_sum_e70fcef055657787_return_1, mal_repr_sum_e70fcef055657787_t, MalRepr_Sum_e70fcef055657787, mal_repr_product_01ed5581230a2b12_t, mal_repr_sum_e70fcef055657787_tag_1, variant_1, mal_detail_to_raw_e70fcef055657787)
MAL_DETAIL_SUM_API_repr_sum_e70fcef055657787(MAL_DETAIL_DEFINE_SUM_UNIT_API, MAL_DETAIL_DEFINE_SUM_VALUE_API)
#endif

static inline mal_Allocation_t mal_Allocation_from_bits(uintptr_t bits) {
    return (mal_Allocation_t){ .mal_detail_bits = bits };
}

static inline mal_Allocation_t mal_detail_to_host_Allocation(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, MalType_Allocation value) {
    return mal_Allocation_from_bits(value.bits);
}

static inline uintptr_t mal_Allocation_to_bits(mal_Allocation_t value) {
    return value.mal_detail_bits;
}

static inline MalType_Allocation mal_Allocation_return(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, mal_Allocation_t value) {
    return (MalType_Allocation){ .bits = value.mal_detail_bits };
}

#define mal_ResizeResult_tag_0 UINT32_C(0)
#define mal_ResizeResult_tag_1 UINT32_C(1)
#define MAL_DETAIL_SUM_API_ResizeResult(unit, value) \
unit(mal_ResizeResult_make_0, mal_ResizeResult_return_0, mal_ResizeResult_t, MalType_ResizeResult, mal_ResizeResult_tag_0, variant_0, mal_detail_to_raw_e70fcef055657787) \
value(mal_ResizeResult_make_1, mal_ResizeResult_return_1, mal_ResizeResult_t, MalType_ResizeResult, mal_repr_product_01ed5581230a2b12_t, mal_ResizeResult_tag_1, variant_1, mal_detail_to_raw_e70fcef055657787)
MAL_DETAIL_SUM_API_ResizeResult(MAL_DETAIL_DEFINE_SUM_UNIT_API, MAL_DETAIL_DEFINE_SUM_VALUE_API)

/* External operations */

MalType_Allocation mal_ext_allocate(MalContext *context, MalType_USize value);
MalType_ResizeResult mal_ext_resize(MalContext *context, MalType_Allocation argument_0, MalType_USize argument_1);
MalType_UInt64 mal_ext_handleBits(MalContext *context, MalType_Allocation value);

/* External definition helpers */

#define MAL_HAS_EXTERN_allocate 1
#define MAL_DEFINE_allocate(call, value) \
static MalType_Allocation mal_detail_allocate(mal_call_t *call, mal_USize_t value); \
MalType_Allocation mal_ext_allocate(MalContext *context MAL_DETAIL_MAYBE_UNUSED, MalType_USize value) { \
    mal_call_t call = (mal_call_t){ .mal_detail_context = context }; \
    return mal_detail_allocate(&call, value); \
} \
static MalType_Allocation mal_detail_allocate( \
    mal_call_t *call, \
    mal_USize_t value \
)

#define MAL_HAS_EXTERN_resize 1
#define MAL_DEFINE_resize(call, value) \
static MalType_ResizeResult mal_detail_resize(mal_call_t *call, mal_repr_product_01ed5581230a2b12_t value); \
MalType_ResizeResult mal_ext_resize(MalContext *context MAL_DETAIL_MAYBE_UNUSED, MalType_Allocation argument_0, MalType_USize argument_1) { \
    mal_call_t call = (mal_call_t){ .mal_detail_context = context }; \
    return mal_detail_resize(&call, mal_detail_to_host_01ed5581230a2b12(&call, (MalRepr_Product_01ed5581230a2b12){ .field_0 = argument_0, .field_1 = argument_1 })); \
} \
static MalType_ResizeResult mal_detail_resize( \
    mal_call_t *call, \
    mal_repr_product_01ed5581230a2b12_t value \
)

#define MAL_HAS_EXTERN_handleBits 1
#define MAL_DEFINE_handleBits(call, value) \
static MalType_UInt64 mal_detail_handleBits(mal_call_t *call, mal_Allocation_t value); \
MalType_UInt64 mal_ext_handleBits(MalContext *context MAL_DETAIL_MAYBE_UNUSED, MalType_Allocation value) { \
    mal_call_t call = (mal_call_t){ .mal_detail_context = context }; \
    return mal_detail_handleBits(&call, (mal_Allocation_t){ .mal_detail_bits = value.bits }); \
} \
static MalType_UInt64 mal_detail_handleBits( \
    mal_call_t *call, \
    mal_Allocation_t value \
)

#endif
#endif
