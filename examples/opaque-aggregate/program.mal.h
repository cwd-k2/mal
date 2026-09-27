#ifndef MAL_BUILD_UMBRELLA
#include <mal.h>

_Static_assert(MAL_C_ABI_VERSION == 0x000900u, "generated header requires mal C ABI 0x000900");
_Static_assert((sizeof((size_t)0) * CHAR_BIT) == 64, "size_t does not match the mal target pointer index width");

#ifndef MAL_GENERATED_INTERFACE_4DACB4AAAF918F4D_H
#define MAL_GENERATED_INTERFACE_4DACB4AAAF918F4D_H
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

#ifndef MAL_DETAIL_RAW_REPR_01ed5581230a2b12_DEFINED
#define MAL_DETAIL_RAW_REPR_01ed5581230a2b12_DEFINED
struct MalRepr_Product_01ed5581230a2b12 {
    MalType_Allocation field_0;
    MalType_USize field_1;
};

#endif

#ifndef MAL_DETAIL_RAW_REPR_e70fcef055657787_DEFINED
#define MAL_DETAIL_RAW_REPR_e70fcef055657787_DEFINED
struct MalRepr_Sum_e70fcef055657787 {
    uint32_t tag;
    union {
        MalType_Unit variant_0;
        MalRepr_Product_01ed5581230a2b12 variant_1;
    } payload;
};

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
struct mal_detail_repr_product_01ed5581230a2b12 {
    mal_Allocation_t field_0;
    mal_USize_t field_1;
};
#endif

#ifndef MAL_DETAIL_HOST_REPR_e70fcef055657787_DEFINED
#define MAL_DETAIL_HOST_REPR_e70fcef055657787_DEFINED
struct mal_detail_repr_sum_e70fcef055657787 {
    uint32_t tag;
    union {
        mal_Unit_t variant_0;
        mal_repr_product_01ed5581230a2b12_t variant_1;
    } payload;
};
#endif

/* Type helpers */

#ifndef MAL_DETAIL_HOST_REPR_01ed5581230a2b12_HELPERS
#define MAL_DETAIL_HOST_REPR_01ed5581230a2b12_HELPERS
static inline MalRepr_Product_01ed5581230a2b12 mal_repr_product_01ed5581230a2b12_return(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, mal_repr_product_01ed5581230a2b12_t value) {
    return (MalRepr_Product_01ed5581230a2b12){ .field_0 = (MalType_Allocation){ .bits = value.field_0.mal_detail_bits }, .field_1 = value.field_1 };
}

#endif

#ifndef MAL_DETAIL_HOST_REPR_e70fcef055657787_HELPERS
#define MAL_DETAIL_HOST_REPR_e70fcef055657787_HELPERS
static inline mal_repr_sum_e70fcef055657787_t mal_detail_to_host_e70fcef055657787(mal_call_t *call, MalRepr_Sum_e70fcef055657787 value) {
    switch (value.tag) {
        case UINT32_C(0): {
            return (mal_repr_sum_e70fcef055657787_t){ .tag = UINT32_C(0), .payload.variant_0 = value.payload.variant_0 };
        }
        case UINT32_C(1): {
            return (mal_repr_sum_e70fcef055657787_t){ .tag = UINT32_C(1), .payload.variant_1 = (mal_repr_product_01ed5581230a2b12_t){ .field_0 = (mal_Allocation_t){ .mal_detail_bits = value.payload.variant_1.field_0.bits }, .field_1 = value.payload.variant_1.field_1 } };
        }
        default: {
            mal_call_trap(call, "invalid sum tag");
        }
    }
}

static inline MalRepr_Sum_e70fcef055657787 mal_detail_to_raw_e70fcef055657787(mal_call_t *call, mal_repr_sum_e70fcef055657787_t value) {
    switch (value.tag) {
        case UINT32_C(0): {
            return (MalRepr_Sum_e70fcef055657787){ .tag = UINT32_C(0), .payload.variant_0 = (MalType_Unit){ 0 } };
        }
        case UINT32_C(1): {
            return (MalRepr_Sum_e70fcef055657787){ .tag = UINT32_C(1), .payload.variant_1 = (MalRepr_Product_01ed5581230a2b12){ .field_0 = (MalType_Allocation){ .bits = value.payload.variant_1.field_0.mal_detail_bits }, .field_1 = value.payload.variant_1.field_1 } };
        }
        default: {
            mal_call_trap(call, "invalid sum tag");
        }
    }
}

#define mal_repr_sum_e70fcef055657787_tag_0 UINT32_C(0)
static inline mal_repr_sum_e70fcef055657787_t mal_repr_sum_e70fcef055657787_make_0(void) {
    return (mal_repr_sum_e70fcef055657787_t){ .tag = mal_repr_sum_e70fcef055657787_tag_0, .payload.variant_0 = (mal_Unit_t){ 0 } };
}

static inline MalRepr_Sum_e70fcef055657787 mal_repr_sum_e70fcef055657787_return_0(mal_call_t *call) {
    return mal_detail_to_raw_e70fcef055657787(call, (mal_repr_sum_e70fcef055657787_t){ .tag = mal_repr_sum_e70fcef055657787_tag_0, .payload.variant_0 = (mal_Unit_t){ 0 } });
}

#define mal_repr_sum_e70fcef055657787_tag_1 UINT32_C(1)
static inline mal_repr_sum_e70fcef055657787_t mal_repr_sum_e70fcef055657787_make_1(mal_repr_product_01ed5581230a2b12_t value) {
    return (mal_repr_sum_e70fcef055657787_t){ .tag = mal_repr_sum_e70fcef055657787_tag_1, .payload.variant_1 = value };
}

static inline MalRepr_Sum_e70fcef055657787 mal_repr_sum_e70fcef055657787_return_1(mal_call_t *call, mal_repr_product_01ed5581230a2b12_t value) {
    return mal_detail_to_raw_e70fcef055657787(call, (mal_repr_sum_e70fcef055657787_t){ .tag = mal_repr_sum_e70fcef055657787_tag_1, .payload.variant_1 = value });
}

#endif

static inline mal_Allocation_t mal_Allocation_from_bits(uintptr_t bits) {
    return (mal_Allocation_t){ .mal_detail_bits = bits };
}

static inline uintptr_t mal_Allocation_to_bits(mal_Allocation_t value) {
    return value.mal_detail_bits;
}

static inline MalType_Allocation mal_Allocation_return(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, mal_Allocation_t value) {
    return (MalType_Allocation){ .bits = value.mal_detail_bits };
}

#define mal_ResizeResult_tag_0 UINT32_C(0)
static inline mal_ResizeResult_t mal_ResizeResult_make_0(void) {
    return (mal_ResizeResult_t){ .tag = mal_ResizeResult_tag_0, .payload.variant_0 = (mal_Unit_t){ 0 } };
}

static inline MalType_ResizeResult mal_ResizeResult_return_0(mal_call_t *call) {
    return mal_detail_to_raw_e70fcef055657787(call, (mal_ResizeResult_t){ .tag = mal_ResizeResult_tag_0, .payload.variant_0 = (mal_Unit_t){ 0 } });
}

#define mal_ResizeResult_tag_1 UINT32_C(1)
static inline mal_ResizeResult_t mal_ResizeResult_make_1(mal_repr_product_01ed5581230a2b12_t value) {
    return (mal_ResizeResult_t){ .tag = mal_ResizeResult_tag_1, .payload.variant_1 = value };
}

static inline MalType_ResizeResult mal_ResizeResult_return_1(mal_call_t *call, mal_repr_product_01ed5581230a2b12_t value) {
    return mal_detail_to_raw_e70fcef055657787(call, (mal_ResizeResult_t){ .tag = mal_ResizeResult_tag_1, .payload.variant_1 = value });
}

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
    return mal_detail_resize(&call, (mal_repr_product_01ed5581230a2b12_t){ .field_0 = (mal_Allocation_t){ .mal_detail_bits = ((MalRepr_Product_01ed5581230a2b12){ .field_0 = argument_0, .field_1 = argument_1 }).field_0.bits }, .field_1 = ((MalRepr_Product_01ed5581230a2b12){ .field_0 = argument_0, .field_1 = argument_1 }).field_1 }); \
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
