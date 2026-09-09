#ifndef MAL_PROGRAM_MAL_H
#define MAL_PROGRAM_MAL_H

#include <stddef.h>
#include <stdint.h>

#define MAL_C_ABI_VERSION 0x000600u

#if defined(__clang__) || defined(__GNUC__)
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
typedef struct { const uint8_t *data; uint64_t length; void *ownership; } MalType_Symbol;
typedef struct { uint8_t *address; } MalType_Ptr;
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
typedef void *mal_Ptr_t;
typedef struct { MalContext *mal_detail_context; } mal_call_t;
typedef struct { const uint8_t *data; uint64_t length; } mal_span_t;
typedef struct { MalType_Symbol mal_detail_raw; mal_span_t mal_detail_bytes; uint8_t mal_detail_source; } mal_Symbol_t;

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
static inline MalType_Bool mal_Bool_return(mal_call_t *call, mal_Bool_t value) {
    if ((value != mal_false) && (value != mal_true)) {
        mal_call_trap(call, "invalid Bool result");
    }
    return value;
}
static inline MalType_Ptr mal_Ptr_return(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, mal_Ptr_t value) {
    return (MalType_Ptr){ .address = (uint8_t *)value };
}
MalType_Symbol mal_symbol_materialize(MalContext *context, MalType_Symbol value);
MalType_Symbol mal_symbol_copy_from_bytes(MalContext *context, const uint8_t *data, uint64_t length);
MalType_Symbol mal_symbol_retain(MalContext *context, MalType_Symbol value);
static inline mal_Symbol_t mal_Symbol_from_bytes(mal_span_t bytes) {
    return (mal_Symbol_t){ .mal_detail_raw = (MalType_Symbol){ 0 }, .mal_detail_bytes = bytes, .mal_detail_source = UINT8_C(1) };
}
static inline mal_span_t mal_Symbol_to_bytes(mal_call_t *call, mal_Symbol_t value) {
    if (value.mal_detail_source != UINT8_C(0)) {
        return value.mal_detail_bytes;
    }
    MalType_Symbol raw = mal_symbol_materialize(call->mal_detail_context, value.mal_detail_raw);
    return (mal_span_t){ .data = raw.data, .length = raw.length };
}
static inline MalType_Symbol mal_detail_Symbol_return(mal_call_t *call, mal_Symbol_t value) {
    if (value.mal_detail_source != UINT8_C(0)) {
        if ((value.mal_detail_bytes.length != UINT64_C(0)) && (value.mal_detail_bytes.data == NULL)) {
            mal_call_trap(call, "null Symbol data");
        }
        return mal_symbol_copy_from_bytes(call->mal_detail_context, value.mal_detail_bytes.data, value.mal_detail_bytes.length);
    }
    return mal_symbol_retain(call->mal_detail_context, value.mal_detail_raw);
}
static inline MalType_Symbol mal_Symbol_return(mal_call_t *call, mal_Symbol_t value) {
    return mal_detail_Symbol_return(call, value);
}

/* Host-visible types */

typedef struct { uintptr_t bits; } MalType_Mem;

typedef struct MalRepr_Product_0 MalRepr_Product_0;
typedef struct MalRepr_Sum_1 MalRepr_Sum_1;

struct MalRepr_Product_0 {
    MalType_Mem field_0;
    MalType_UInt64 field_1;
};

struct MalRepr_Sum_1 {
    uint32_t tag;
    union {
        MalType_Unit variant_0;
        MalRepr_Product_0 variant_1;
    } payload;
};

typedef MalRepr_Sum_1 MalType_Response;

typedef struct { uintptr_t mal_detail_bits; } mal_Mem_t;
typedef struct mal_detail_repr_product_0 mal_repr_product_0_t;
typedef struct mal_detail_repr_sum_1 mal_repr_sum_1_t;
typedef mal_repr_sum_1_t mal_Response_t;

struct mal_detail_repr_product_0 {
    mal_Mem_t field_0;
    mal_UInt64_t field_1;
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
    return (MalRepr_Product_0){ .field_0 = (MalType_Mem){ .bits = value.field_0.mal_detail_bits }, .field_1 = value.field_1 };
}

static inline mal_repr_sum_1_t mal_detail_to_host_1(mal_call_t *call, MalRepr_Sum_1 value) {
    switch (value.tag) {
        case UINT32_C(0): {
            return (mal_repr_sum_1_t){ .tag = UINT32_C(0), .payload.variant_0 = value.payload.variant_0 };
        }
        case UINT32_C(1): {
            return (mal_repr_sum_1_t){ .tag = UINT32_C(1), .payload.variant_1 = (mal_repr_product_0_t){ .field_0 = (mal_Mem_t){ .mal_detail_bits = value.payload.variant_1.field_0.bits }, .field_1 = value.payload.variant_1.field_1 } };
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
            return (MalRepr_Sum_1){ .tag = UINT32_C(1), .payload.variant_1 = (MalRepr_Product_0){ .field_0 = (MalType_Mem){ .bits = value.payload.variant_1.field_0.mal_detail_bits }, .field_1 = value.payload.variant_1.field_1 } };
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

static inline mal_Mem_t mal_Mem_from_bits(uintptr_t bits) {
    return (mal_Mem_t){ .mal_detail_bits = bits };
}

static inline uintptr_t mal_Mem_to_bits(mal_Mem_t value) {
    return value.mal_detail_bits;
}

static inline MalType_Mem mal_Mem_return(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, mal_Mem_t value) {
    return (MalType_Mem){ .bits = value.mal_detail_bits };
}

#define mal_Response_tag_0 UINT32_C(0)
static inline mal_Response_t mal_Response_make_0(void) {
    return (mal_Response_t){ .tag = mal_Response_tag_0, .payload.variant_0 = (mal_Unit_t){ 0 } };
}

static inline MalType_Response mal_Response_return_0(mal_call_t *call) {
    return mal_detail_to_raw_1(call, (mal_Response_t){ .tag = mal_Response_tag_0, .payload.variant_0 = (mal_Unit_t){ 0 } });
}

#define mal_Response_tag_1 UINT32_C(1)
static inline mal_Response_t mal_Response_make_1(mal_repr_product_0_t value) {
    return (mal_Response_t){ .tag = mal_Response_tag_1, .payload.variant_1 = value };
}

static inline MalType_Response mal_Response_return_1(mal_call_t *call, mal_repr_product_0_t value) {
    return mal_detail_to_raw_1(call, (mal_Response_t){ .tag = mal_Response_tag_1, .payload.variant_1 = value });
}

/* External operations */

MalType_Mem mal_ext_allocate(MalContext *context, MalType_UInt64 value);
MalType_Response mal_ext_resize(MalContext *context, MalType_Mem argument_0, MalType_UInt64 argument_1);
MalType_UInt64 mal_ext_handleBits(MalContext *context, MalType_Mem value);

/* External definition helpers */

#define MAL_HAS_EXTERN_allocate 1
#define MAL_DEFINE_allocate(call, value) \
static MalType_Mem mal_detail_allocate(mal_call_t *call, mal_UInt64_t value); \
MalType_Mem mal_ext_allocate(MalContext *context MAL_DETAIL_MAYBE_UNUSED, MalType_UInt64 value) { \
    mal_call_t call = (mal_call_t){ .mal_detail_context = context }; \
    return mal_detail_allocate(&call, value); \
} \
static MalType_Mem mal_detail_allocate( \
    mal_call_t *call, \
    mal_UInt64_t value \
)

#define MAL_HAS_EXTERN_resize 1
#define MAL_DEFINE_resize(call, value) \
static MalType_Response mal_detail_resize(mal_call_t *call, mal_repr_product_0_t value); \
MalType_Response mal_ext_resize(MalContext *context MAL_DETAIL_MAYBE_UNUSED, MalType_Mem argument_0, MalType_UInt64 argument_1) { \
    mal_call_t call = (mal_call_t){ .mal_detail_context = context }; \
    return mal_detail_resize(&call, (mal_repr_product_0_t){ .field_0 = (mal_Mem_t){ .mal_detail_bits = ((MalRepr_Product_0){ .field_0 = argument_0, .field_1 = argument_1 }).field_0.bits }, .field_1 = ((MalRepr_Product_0){ .field_0 = argument_0, .field_1 = argument_1 }).field_1 }); \
} \
static MalType_Response mal_detail_resize( \
    mal_call_t *call, \
    mal_repr_product_0_t value \
)

#define MAL_HAS_EXTERN_handleBits 1
#define MAL_DEFINE_handleBits(call, value) \
static MalType_UInt64 mal_detail_handleBits(mal_call_t *call, mal_Mem_t value); \
MalType_UInt64 mal_ext_handleBits(MalContext *context MAL_DETAIL_MAYBE_UNUSED, MalType_Mem value) { \
    mal_call_t call = (mal_call_t){ .mal_detail_context = context }; \
    return mal_detail_handleBits(&call, (mal_Mem_t){ .mal_detail_bits = value.bits }); \
} \
static MalType_UInt64 mal_detail_handleBits( \
    mal_call_t *call, \
    mal_Mem_t value \
)

#endif
