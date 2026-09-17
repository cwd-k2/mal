#ifndef MAL_PROGRAM_MAL_H
#define MAL_PROGRAM_MAL_H

#include <stddef.h>
#include <stdint.h>
#include <limits.h>

#define MAL_C_ABI_VERSION 0x000800u

#if defined(__clang__)
#define MAL_DETAIL_MAYBE_UNUSED __attribute__((unused))
#else
#define MAL_DETAIL_MAYBE_UNUSED
#endif

/* Runtime API */

typedef struct MalContext MalContext;
typedef struct { uint8_t unused; } MalType_Unit;
typedef uint8_t MalType_Bool;
typedef int8_t MalType_Int8;
typedef int16_t MalType_Int16;
typedef int32_t MalType_Int32;
typedef int64_t MalType_Int64;
typedef uint8_t MalType_UInt8;
typedef uint16_t MalType_UInt16;
typedef uint32_t MalType_UInt32;
typedef uint64_t MalType_UInt64;
typedef float MalType_Float32;
typedef double MalType_Float64;
typedef size_t MalType_ByteSize;
typedef size_t MalType_USize;
typedef void *MalType_Address;
_Static_assert((sizeof((size_t)0) * CHAR_BIT) == 64, "size_t does not match the mal target pointer index width");
typedef MalType_Unit mal_Unit_t;
typedef MalType_Bool mal_Bool_t;
typedef MalType_Int8 mal_Int8_t;
typedef MalType_Int16 mal_Int16_t;
typedef MalType_Int32 mal_Int32_t;
typedef MalType_Int64 mal_Int64_t;
typedef MalType_UInt8 mal_UInt8_t;
typedef MalType_UInt16 mal_UInt16_t;
typedef MalType_UInt32 mal_UInt32_t;
typedef MalType_UInt64 mal_UInt64_t;
typedef MalType_Float32 mal_Float32_t;
typedef MalType_Float64 mal_Float64_t;
typedef MalType_Address mal_Address_t;
typedef MalType_ByteSize mal_ByteSize_t;
typedef MalType_USize mal_USize_t;
typedef struct { MalContext *mal_detail_context; } mal_call_t;

#define mal_false (mal_Bool_t)UINT8_C(0)
#define mal_true (mal_Bool_t)UINT8_C(1)

_Noreturn void mal_trap(MalContext *context, const char *message);
static inline _Noreturn void mal_call_trap(mal_call_t *call, const char *message) {
    mal_trap(call->mal_detail_context, message);
}
static inline MalType_Unit mal_Unit_return(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED) {
    return (MalType_Unit){ .unused = UINT8_C(0) };
}
static inline MalType_Int8 mal_Int8_return(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, mal_Int8_t value) {
    return value;
}
static inline MalType_Int16 mal_Int16_return(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, mal_Int16_t value) {
    return value;
}
static inline MalType_Int32 mal_Int32_return(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, mal_Int32_t value) {
    return value;
}
static inline MalType_Int64 mal_Int64_return(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, mal_Int64_t value) {
    return value;
}
static inline MalType_UInt8 mal_UInt8_return(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, mal_UInt8_t value) {
    return value;
}
static inline MalType_UInt16 mal_UInt16_return(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, mal_UInt16_t value) {
    return value;
}
static inline MalType_UInt32 mal_UInt32_return(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, mal_UInt32_t value) {
    return value;
}
static inline MalType_UInt64 mal_UInt64_return(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, mal_UInt64_t value) {
    return value;
}
static inline MalType_Float32 mal_Float32_return(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, mal_Float32_t value) {
    return value;
}
static inline MalType_Float64 mal_Float64_return(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, mal_Float64_t value) {
    return value;
}
static inline MalType_ByteSize mal_ByteSize_return(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, mal_ByteSize_t value) {
    return value;
}
static inline MalType_USize mal_USize_return(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, mal_USize_t value) {
    return value;
}
static inline MalType_Address mal_Address_return(mal_call_t *call, mal_Address_t value) {
    if (value == 0) {
        mal_call_trap(call, "invalid Address result");
    }
    return value;
}
static inline MalType_Bool mal_Bool_return(mal_call_t *call, mal_Bool_t value) {
    if ((value != mal_false) && (value != mal_true)) {
        mal_call_trap(call, "invalid Bool result");
    }
    return value;
}

/* Host-visible types */

typedef struct { uintptr_t bits; } MalType_Allocation;

typedef struct MalRepr_Product_0 MalRepr_Product_0;
typedef struct MalRepr_Sum_1 MalRepr_Sum_1;

struct MalRepr_Product_0 {
    MalType_Allocation field_0;
    MalType_USize field_1;
};

struct MalRepr_Sum_1 {
    uint32_t tag;
    union {
        MalType_Unit variant_0;
        MalRepr_Product_0 variant_1;
    } payload;
};

typedef MalRepr_Sum_1 MalType_ResizeResult;

typedef struct { uintptr_t mal_detail_bits; } mal_Allocation_t;
typedef struct mal_detail_repr_product_0 mal_repr_product_0_t;
typedef struct mal_detail_repr_sum_1 mal_repr_sum_1_t;
typedef mal_repr_sum_1_t mal_ResizeResult_t;

struct mal_detail_repr_product_0 {
    mal_Allocation_t field_0;
    mal_USize_t field_1;
};

struct mal_detail_repr_sum_1 {
    uint32_t tag;
    union {
        mal_Unit_t variant_0;
        mal_repr_product_0_t variant_1;
    } payload;
};

/* Type helpers */

static inline MalRepr_Product_0 mal_repr_product_0_return(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, mal_repr_product_0_t value) {
    return (MalRepr_Product_0){ .field_0 = (MalType_Allocation){ .bits = value.field_0.mal_detail_bits }, .field_1 = value.field_1 };
}

static inline mal_repr_sum_1_t mal_detail_to_host_1(mal_call_t *call, MalRepr_Sum_1 value) {
    switch (value.tag) {
        case UINT32_C(0): {
            return (mal_repr_sum_1_t){ .tag = UINT32_C(0), .payload.variant_0 = value.payload.variant_0 };
        }
        case UINT32_C(1): {
            return (mal_repr_sum_1_t){ .tag = UINT32_C(1), .payload.variant_1 = (mal_repr_product_0_t){ .field_0 = (mal_Allocation_t){ .mal_detail_bits = value.payload.variant_1.field_0.bits }, .field_1 = value.payload.variant_1.field_1 } };
        }
        default: {
            mal_call_trap(call, "invalid sum tag");
        }
    }
}

static inline MalRepr_Sum_1 mal_detail_to_raw_1(mal_call_t *call, mal_repr_sum_1_t value) {
    switch (value.tag) {
        case UINT32_C(0): {
            return (MalRepr_Sum_1){ .tag = UINT32_C(0), .payload.variant_0 = (MalType_Unit){ 0 } };
        }
        case UINT32_C(1): {
            return (MalRepr_Sum_1){ .tag = UINT32_C(1), .payload.variant_1 = (MalRepr_Product_0){ .field_0 = (MalType_Allocation){ .bits = value.payload.variant_1.field_0.mal_detail_bits }, .field_1 = value.payload.variant_1.field_1 } };
        }
        default: {
            mal_call_trap(call, "invalid sum tag");
        }
    }
}

#define mal_repr_sum_1_tag_0 UINT32_C(0)
static inline mal_repr_sum_1_t mal_repr_sum_1_make_0(void) {
    return (mal_repr_sum_1_t){ .tag = mal_repr_sum_1_tag_0, .payload.variant_0 = (mal_Unit_t){ 0 } };
}

static inline MalRepr_Sum_1 mal_repr_sum_1_return_0(mal_call_t *call) {
    return mal_detail_to_raw_1(call, (mal_repr_sum_1_t){ .tag = mal_repr_sum_1_tag_0, .payload.variant_0 = (mal_Unit_t){ 0 } });
}

#define mal_repr_sum_1_tag_1 UINT32_C(1)
static inline mal_repr_sum_1_t mal_repr_sum_1_make_1(mal_repr_product_0_t value) {
    return (mal_repr_sum_1_t){ .tag = mal_repr_sum_1_tag_1, .payload.variant_1 = value };
}

static inline MalRepr_Sum_1 mal_repr_sum_1_return_1(mal_call_t *call, mal_repr_product_0_t value) {
    return mal_detail_to_raw_1(call, (mal_repr_sum_1_t){ .tag = mal_repr_sum_1_tag_1, .payload.variant_1 = value });
}

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
    return mal_detail_to_raw_1(call, (mal_ResizeResult_t){ .tag = mal_ResizeResult_tag_0, .payload.variant_0 = (mal_Unit_t){ 0 } });
}

#define mal_ResizeResult_tag_1 UINT32_C(1)
static inline mal_ResizeResult_t mal_ResizeResult_make_1(mal_repr_product_0_t value) {
    return (mal_ResizeResult_t){ .tag = mal_ResizeResult_tag_1, .payload.variant_1 = value };
}

static inline MalType_ResizeResult mal_ResizeResult_return_1(mal_call_t *call, mal_repr_product_0_t value) {
    return mal_detail_to_raw_1(call, (mal_ResizeResult_t){ .tag = mal_ResizeResult_tag_1, .payload.variant_1 = value });
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
static MalType_ResizeResult mal_detail_resize(mal_call_t *call, mal_repr_product_0_t value); \
MalType_ResizeResult mal_ext_resize(MalContext *context MAL_DETAIL_MAYBE_UNUSED, MalType_Allocation argument_0, MalType_USize argument_1) { \
    mal_call_t call = (mal_call_t){ .mal_detail_context = context }; \
    return mal_detail_resize(&call, (mal_repr_product_0_t){ .field_0 = (mal_Allocation_t){ .mal_detail_bits = ((MalRepr_Product_0){ .field_0 = argument_0, .field_1 = argument_1 }).field_0.bits }, .field_1 = ((MalRepr_Product_0){ .field_0 = argument_0, .field_1 = argument_1 }).field_1 }); \
} \
static MalType_ResizeResult mal_detail_resize( \
    mal_call_t *call, \
    mal_repr_product_0_t value \
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
