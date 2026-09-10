#ifndef MAL_PROGRAM_MAL_H
#define MAL_PROGRAM_MAL_H

#include <stddef.h>
#include <stdint.h>

#define MAL_C_ABI_VERSION 0x000600u

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

typedef struct { uintptr_t bits; } MalType_Allocation;
typedef struct { uintptr_t bits; } MalType_File;

typedef struct MalRepr_Product_0 MalRepr_Product_0;
typedef struct MalRepr_Product_1 MalRepr_Product_1;
typedef struct MalRepr_Sum_2 MalRepr_Sum_2;
typedef struct MalRepr_Product_3 MalRepr_Product_3;
typedef struct MalRepr_Product_4 MalRepr_Product_4;
typedef struct MalRepr_Sum_5 MalRepr_Sum_5;
typedef struct MalRepr_Sum_6 MalRepr_Sum_6;

struct MalRepr_Product_0 {
    MalType_Ptr field_0;
    MalType_UInt64 field_1;
    MalType_UInt64 field_2;
};

struct MalRepr_Product_1 {
    MalType_Allocation field_0;
    MalRepr_Product_0 field_1;
};

struct MalRepr_Sum_2 {
    uint32_t tag;
    union {
        MalType_File variant_0;
        MalType_UInt32 variant_1;
    } payload;
};

struct MalRepr_Product_3 {
    MalType_Ptr field_0;
    MalType_UInt64 field_1;
};

struct MalRepr_Product_4 {
    MalType_File field_0;
    MalRepr_Product_3 field_1;
};

struct MalRepr_Sum_5 {
    uint32_t tag;
    union {
        MalType_UInt64 variant_0;
        MalType_UInt32 variant_1;
    } payload;
};

struct MalRepr_Sum_6 {
    uint32_t tag;
    union {
        MalType_Unit variant_0;
        MalType_UInt32 variant_1;
    } payload;
};

typedef MalRepr_Product_0 MalType_Buffer;
typedef MalRepr_Product_3 MalType_Region;
typedef MalRepr_Product_3 MalType_Bytes;
typedef MalType_UInt32 MalType_IoError;
typedef MalRepr_Product_1 MalType_OwnedBuffer;
typedef MalRepr_Sum_2 MalType_OpenResult;
typedef MalRepr_Sum_5 MalType_ReadResult;
typedef MalRepr_Sum_6 MalType_CloseResult;
typedef MalRepr_Sum_6 MalType_Status;

typedef struct { uintptr_t mal_detail_bits; } mal_Allocation_t;
typedef struct { uintptr_t mal_detail_bits; } mal_File_t;
typedef struct mal_detail_repr_product_0 mal_repr_product_0_t;
typedef struct mal_detail_repr_product_1 mal_repr_product_1_t;
typedef struct mal_detail_repr_sum_2 mal_repr_sum_2_t;
typedef struct mal_detail_repr_product_3 mal_repr_product_3_t;
typedef struct mal_detail_repr_product_4 mal_repr_product_4_t;
typedef struct mal_detail_repr_sum_5 mal_repr_sum_5_t;
typedef struct mal_detail_repr_sum_6 mal_repr_sum_6_t;
typedef mal_repr_product_0_t mal_Buffer_t;
typedef mal_repr_product_3_t mal_Region_t;
typedef mal_repr_product_3_t mal_Bytes_t;
typedef mal_UInt32_t mal_IoError_t;
typedef mal_repr_product_1_t mal_OwnedBuffer_t;
typedef mal_repr_sum_2_t mal_OpenResult_t;
typedef mal_repr_sum_5_t mal_ReadResult_t;
typedef mal_repr_sum_6_t mal_CloseResult_t;
typedef mal_repr_sum_6_t mal_Status_t;

struct mal_detail_repr_product_0 {
    mal_Ptr_t field_0;
    mal_UInt64_t field_1;
    mal_UInt64_t field_2;
};

struct mal_detail_repr_product_1 {
    mal_Allocation_t field_0;
    mal_repr_product_0_t field_1;
};

struct mal_detail_repr_sum_2 {
    uint32_t tag;
    union {
        mal_File_t variant_0;
        mal_UInt32_t variant_1;
    } payload;
};

struct mal_detail_repr_product_3 {
    mal_Ptr_t field_0;
    mal_UInt64_t field_1;
};

struct mal_detail_repr_product_4 {
    mal_File_t field_0;
    mal_repr_product_3_t field_1;
};

struct mal_detail_repr_sum_5 {
    uint32_t tag;
    union {
        mal_UInt64_t variant_0;
        mal_UInt32_t variant_1;
    } payload;
};

struct mal_detail_repr_sum_6 {
    uint32_t tag;
    union {
        mal_Unit_t variant_0;
        mal_UInt32_t variant_1;
    } payload;
};

/* Type helpers */

static inline MalRepr_Product_0 mal_repr_product_0_return(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, mal_repr_product_0_t value) {
    return (MalRepr_Product_0){ .field_0 = (MalType_Ptr){ .address = (uint8_t *)value.field_0 }, .field_1 = value.field_1, .field_2 = value.field_2 };
}

static inline MalRepr_Product_1 mal_repr_product_1_return(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, mal_repr_product_1_t value) {
    return (MalRepr_Product_1){ .field_0 = (MalType_Allocation){ .bits = value.field_0.mal_detail_bits }, .field_1 = (MalRepr_Product_0){ .field_0 = (MalType_Ptr){ .address = (uint8_t *)value.field_1.field_0 }, .field_1 = value.field_1.field_1, .field_2 = value.field_1.field_2 } };
}

static inline mal_repr_sum_2_t mal_detail_to_host_2(mal_call_t *call, MalRepr_Sum_2 value) {
    switch (value.tag) {
        case UINT32_C(0): {
            return (mal_repr_sum_2_t){ .tag = UINT32_C(0), .payload.variant_0 = (mal_File_t){ .mal_detail_bits = value.payload.variant_0.bits } };
        }
        case UINT32_C(1): {
            return (mal_repr_sum_2_t){ .tag = UINT32_C(1), .payload.variant_1 = value.payload.variant_1 };
        }
        default: {
            mal_call_trap(call, "invalid sum tag");
        }
    }
}

static inline MalRepr_Sum_2 mal_detail_to_raw_2(mal_call_t *call, mal_repr_sum_2_t value) {
    switch (value.tag) {
        case UINT32_C(0): {
            return (MalRepr_Sum_2){ .tag = UINT32_C(0), .payload.variant_0 = (MalType_File){ .bits = value.payload.variant_0.mal_detail_bits } };
        }
        case UINT32_C(1): {
            return (MalRepr_Sum_2){ .tag = UINT32_C(1), .payload.variant_1 = value.payload.variant_1 };
        }
        default: {
            mal_call_trap(call, "invalid sum tag");
        }
    }
}

#define mal_repr_sum_2_tag_0 UINT32_C(0)
static inline mal_repr_sum_2_t mal_repr_sum_2_make_0(mal_File_t value) {
    return (mal_repr_sum_2_t){ .tag = mal_repr_sum_2_tag_0, .payload.variant_0 = value };
}

static inline MalRepr_Sum_2 mal_repr_sum_2_return_0(mal_call_t *call, mal_File_t value) {
    return mal_detail_to_raw_2(call, (mal_repr_sum_2_t){ .tag = mal_repr_sum_2_tag_0, .payload.variant_0 = value });
}

#define mal_repr_sum_2_tag_1 UINT32_C(1)
static inline mal_repr_sum_2_t mal_repr_sum_2_make_1(mal_UInt32_t value) {
    return (mal_repr_sum_2_t){ .tag = mal_repr_sum_2_tag_1, .payload.variant_1 = value };
}

static inline MalRepr_Sum_2 mal_repr_sum_2_return_1(mal_call_t *call, mal_UInt32_t value) {
    return mal_detail_to_raw_2(call, (mal_repr_sum_2_t){ .tag = mal_repr_sum_2_tag_1, .payload.variant_1 = value });
}

static inline MalRepr_Product_3 mal_repr_product_3_return(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, mal_repr_product_3_t value) {
    return (MalRepr_Product_3){ .field_0 = (MalType_Ptr){ .address = (uint8_t *)value.field_0 }, .field_1 = value.field_1 };
}

static inline MalRepr_Product_4 mal_repr_product_4_return(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, mal_repr_product_4_t value) {
    return (MalRepr_Product_4){ .field_0 = (MalType_File){ .bits = value.field_0.mal_detail_bits }, .field_1 = (MalRepr_Product_3){ .field_0 = (MalType_Ptr){ .address = (uint8_t *)value.field_1.field_0 }, .field_1 = value.field_1.field_1 } };
}

static inline mal_repr_sum_5_t mal_detail_to_host_5(mal_call_t *call, MalRepr_Sum_5 value) {
    switch (value.tag) {
        case UINT32_C(0): {
            return (mal_repr_sum_5_t){ .tag = UINT32_C(0), .payload.variant_0 = value.payload.variant_0 };
        }
        case UINT32_C(1): {
            return (mal_repr_sum_5_t){ .tag = UINT32_C(1), .payload.variant_1 = value.payload.variant_1 };
        }
        default: {
            mal_call_trap(call, "invalid sum tag");
        }
    }
}

static inline MalRepr_Sum_5 mal_detail_to_raw_5(mal_call_t *call, mal_repr_sum_5_t value) {
    switch (value.tag) {
        case UINT32_C(0): {
            return (MalRepr_Sum_5){ .tag = UINT32_C(0), .payload.variant_0 = value.payload.variant_0 };
        }
        case UINT32_C(1): {
            return (MalRepr_Sum_5){ .tag = UINT32_C(1), .payload.variant_1 = value.payload.variant_1 };
        }
        default: {
            mal_call_trap(call, "invalid sum tag");
        }
    }
}

#define mal_repr_sum_5_tag_0 UINT32_C(0)
static inline mal_repr_sum_5_t mal_repr_sum_5_make_0(mal_UInt64_t value) {
    return (mal_repr_sum_5_t){ .tag = mal_repr_sum_5_tag_0, .payload.variant_0 = value };
}

static inline MalRepr_Sum_5 mal_repr_sum_5_return_0(mal_call_t *call, mal_UInt64_t value) {
    return mal_detail_to_raw_5(call, (mal_repr_sum_5_t){ .tag = mal_repr_sum_5_tag_0, .payload.variant_0 = value });
}

#define mal_repr_sum_5_tag_1 UINT32_C(1)
static inline mal_repr_sum_5_t mal_repr_sum_5_make_1(mal_UInt32_t value) {
    return (mal_repr_sum_5_t){ .tag = mal_repr_sum_5_tag_1, .payload.variant_1 = value };
}

static inline MalRepr_Sum_5 mal_repr_sum_5_return_1(mal_call_t *call, mal_UInt32_t value) {
    return mal_detail_to_raw_5(call, (mal_repr_sum_5_t){ .tag = mal_repr_sum_5_tag_1, .payload.variant_1 = value });
}

static inline mal_repr_sum_6_t mal_detail_to_host_6(mal_call_t *call, MalRepr_Sum_6 value) {
    switch (value.tag) {
        case UINT32_C(0): {
            return (mal_repr_sum_6_t){ .tag = UINT32_C(0), .payload.variant_0 = value.payload.variant_0 };
        }
        case UINT32_C(1): {
            return (mal_repr_sum_6_t){ .tag = UINT32_C(1), .payload.variant_1 = value.payload.variant_1 };
        }
        default: {
            mal_call_trap(call, "invalid sum tag");
        }
    }
}

static inline MalRepr_Sum_6 mal_detail_to_raw_6(mal_call_t *call, mal_repr_sum_6_t value) {
    switch (value.tag) {
        case UINT32_C(0): {
            return (MalRepr_Sum_6){ .tag = UINT32_C(0), .payload.variant_0 = (MalType_Unit){ 0 } };
        }
        case UINT32_C(1): {
            return (MalRepr_Sum_6){ .tag = UINT32_C(1), .payload.variant_1 = value.payload.variant_1 };
        }
        default: {
            mal_call_trap(call, "invalid sum tag");
        }
    }
}

#define mal_repr_sum_6_tag_0 UINT32_C(0)
static inline mal_repr_sum_6_t mal_repr_sum_6_make_0(void) {
    return (mal_repr_sum_6_t){ .tag = mal_repr_sum_6_tag_0, .payload.variant_0 = (mal_Unit_t){ 0 } };
}

static inline MalRepr_Sum_6 mal_repr_sum_6_return_0(mal_call_t *call) {
    return mal_detail_to_raw_6(call, (mal_repr_sum_6_t){ .tag = mal_repr_sum_6_tag_0, .payload.variant_0 = (mal_Unit_t){ 0 } });
}

#define mal_repr_sum_6_tag_1 UINT32_C(1)
static inline mal_repr_sum_6_t mal_repr_sum_6_make_1(mal_UInt32_t value) {
    return (mal_repr_sum_6_t){ .tag = mal_repr_sum_6_tag_1, .payload.variant_1 = value };
}

static inline MalRepr_Sum_6 mal_repr_sum_6_return_1(mal_call_t *call, mal_UInt32_t value) {
    return mal_detail_to_raw_6(call, (mal_repr_sum_6_t){ .tag = mal_repr_sum_6_tag_1, .payload.variant_1 = value });
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

static inline mal_File_t mal_File_from_bits(uintptr_t bits) {
    return (mal_File_t){ .mal_detail_bits = bits };
}

static inline uintptr_t mal_File_to_bits(mal_File_t value) {
    return value.mal_detail_bits;
}

static inline MalType_File mal_File_return(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, mal_File_t value) {
    return (MalType_File){ .bits = value.mal_detail_bits };
}

static inline MalType_Buffer mal_Buffer_return(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, mal_Buffer_t value) {
    return (MalRepr_Product_0){ .field_0 = (MalType_Ptr){ .address = (uint8_t *)value.field_0 }, .field_1 = value.field_1, .field_2 = value.field_2 };
}

static inline MalType_Region mal_Region_return(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, mal_Region_t value) {
    return (MalRepr_Product_3){ .field_0 = (MalType_Ptr){ .address = (uint8_t *)value.field_0 }, .field_1 = value.field_1 };
}

static inline MalType_Bytes mal_Bytes_return(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, mal_Bytes_t value) {
    return (MalRepr_Product_3){ .field_0 = (MalType_Ptr){ .address = (uint8_t *)value.field_0 }, .field_1 = value.field_1 };
}

static inline MalType_IoError mal_IoError_return(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, mal_IoError_t value) {
    return value;
}

static inline MalType_OwnedBuffer mal_OwnedBuffer_return(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, mal_OwnedBuffer_t value) {
    return (MalRepr_Product_1){ .field_0 = (MalType_Allocation){ .bits = value.field_0.mal_detail_bits }, .field_1 = (MalRepr_Product_0){ .field_0 = (MalType_Ptr){ .address = (uint8_t *)value.field_1.field_0 }, .field_1 = value.field_1.field_1, .field_2 = value.field_1.field_2 } };
}

#define mal_OpenResult_tag_0 UINT32_C(0)
static inline mal_OpenResult_t mal_OpenResult_make_0(mal_File_t value) {
    return (mal_OpenResult_t){ .tag = mal_OpenResult_tag_0, .payload.variant_0 = value };
}

static inline MalType_OpenResult mal_OpenResult_return_0(mal_call_t *call, mal_File_t value) {
    return mal_detail_to_raw_2(call, (mal_OpenResult_t){ .tag = mal_OpenResult_tag_0, .payload.variant_0 = value });
}

#define mal_OpenResult_tag_1 UINT32_C(1)
static inline mal_OpenResult_t mal_OpenResult_make_1(mal_IoError_t value) {
    return (mal_OpenResult_t){ .tag = mal_OpenResult_tag_1, .payload.variant_1 = value };
}

static inline MalType_OpenResult mal_OpenResult_return_1(mal_call_t *call, mal_IoError_t value) {
    return mal_detail_to_raw_2(call, (mal_OpenResult_t){ .tag = mal_OpenResult_tag_1, .payload.variant_1 = value });
}

#define mal_ReadResult_tag_0 UINT32_C(0)
static inline mal_ReadResult_t mal_ReadResult_make_0(mal_UInt64_t value) {
    return (mal_ReadResult_t){ .tag = mal_ReadResult_tag_0, .payload.variant_0 = value };
}

static inline MalType_ReadResult mal_ReadResult_return_0(mal_call_t *call, mal_UInt64_t value) {
    return mal_detail_to_raw_5(call, (mal_ReadResult_t){ .tag = mal_ReadResult_tag_0, .payload.variant_0 = value });
}

#define mal_ReadResult_tag_1 UINT32_C(1)
static inline mal_ReadResult_t mal_ReadResult_make_1(mal_IoError_t value) {
    return (mal_ReadResult_t){ .tag = mal_ReadResult_tag_1, .payload.variant_1 = value };
}

static inline MalType_ReadResult mal_ReadResult_return_1(mal_call_t *call, mal_IoError_t value) {
    return mal_detail_to_raw_5(call, (mal_ReadResult_t){ .tag = mal_ReadResult_tag_1, .payload.variant_1 = value });
}

#define mal_CloseResult_tag_0 UINT32_C(0)
static inline mal_CloseResult_t mal_CloseResult_make_0(void) {
    return (mal_CloseResult_t){ .tag = mal_CloseResult_tag_0, .payload.variant_0 = (mal_Unit_t){ 0 } };
}

static inline MalType_CloseResult mal_CloseResult_return_0(mal_call_t *call) {
    return mal_detail_to_raw_6(call, (mal_CloseResult_t){ .tag = mal_CloseResult_tag_0, .payload.variant_0 = (mal_Unit_t){ 0 } });
}

#define mal_CloseResult_tag_1 UINT32_C(1)
static inline mal_CloseResult_t mal_CloseResult_make_1(mal_IoError_t value) {
    return (mal_CloseResult_t){ .tag = mal_CloseResult_tag_1, .payload.variant_1 = value };
}

static inline MalType_CloseResult mal_CloseResult_return_1(mal_call_t *call, mal_IoError_t value) {
    return mal_detail_to_raw_6(call, (mal_CloseResult_t){ .tag = mal_CloseResult_tag_1, .payload.variant_1 = value });
}

#define mal_Status_tag_0 UINT32_C(0)
static inline mal_Status_t mal_Status_make_0(void) {
    return (mal_Status_t){ .tag = mal_Status_tag_0, .payload.variant_0 = (mal_Unit_t){ 0 } };
}

static inline MalType_Status mal_Status_return_0(mal_call_t *call) {
    return mal_detail_to_raw_6(call, (mal_Status_t){ .tag = mal_Status_tag_0, .payload.variant_0 = (mal_Unit_t){ 0 } });
}

#define mal_Status_tag_1 UINT32_C(1)
static inline mal_Status_t mal_Status_make_1(mal_IoError_t value) {
    return (mal_Status_t){ .tag = mal_Status_tag_1, .payload.variant_1 = value };
}

static inline MalType_Status mal_Status_return_1(mal_call_t *call, mal_IoError_t value) {
    return mal_detail_to_raw_6(call, (mal_Status_t){ .tag = mal_Status_tag_1, .payload.variant_1 = value });
}

/* External operations */

MalType_OwnedBuffer mal_ext_allocateBuffer(MalContext *context, MalType_UInt64 value);
void mal_ext_releaseBuffer(MalContext *context, MalType_Allocation value);
MalType_OpenResult mal_ext_openReadOnly(MalContext *context, MalType_Symbol value);
MalType_ReadResult mal_ext_readFile(MalContext *context, MalType_File argument_0, MalType_Region argument_1);
MalType_CloseResult mal_ext_closeFile(MalContext *context, MalType_File value);
void mal_ext_writeBytes(MalContext *context, MalType_Ptr argument_0, MalType_UInt64 argument_1);
void mal_ext_writeSymbol(MalContext *context, MalType_Symbol value);
void mal_ext_writeError(MalContext *context, MalType_IoError value);

/* External definition helpers */

#define MAL_HAS_EXTERN_allocateBuffer 1
#define MAL_DEFINE_allocateBuffer(call, value) \
static MalType_OwnedBuffer mal_detail_allocateBuffer(mal_call_t *call, mal_UInt64_t value); \
MalType_OwnedBuffer mal_ext_allocateBuffer(MalContext *context MAL_DETAIL_MAYBE_UNUSED, MalType_UInt64 value) { \
    mal_call_t call = (mal_call_t){ .mal_detail_context = context }; \
    return mal_detail_allocateBuffer(&call, value); \
} \
static MalType_OwnedBuffer mal_detail_allocateBuffer( \
    mal_call_t *call, \
    mal_UInt64_t value \
)

#define MAL_HAS_EXTERN_releaseBuffer 1
#define MAL_DEFINE_releaseBuffer(call, value) \
static MalType_Unit mal_detail_releaseBuffer(mal_call_t *call, mal_Allocation_t value); \
void mal_ext_releaseBuffer(MalContext *context MAL_DETAIL_MAYBE_UNUSED, MalType_Allocation value) { \
    mal_call_t call = (mal_call_t){ .mal_detail_context = context }; \
    mal_detail_releaseBuffer(&call, (mal_Allocation_t){ .mal_detail_bits = value.bits }); \
} \
static MalType_Unit mal_detail_releaseBuffer( \
    mal_call_t *call, \
    mal_Allocation_t value \
)

#define MAL_HAS_EXTERN_openReadOnly 1
#define MAL_DEFINE_openReadOnly(call, value) \
static MalType_OpenResult mal_detail_openReadOnly(mal_call_t *call, mal_Symbol_t value); \
MalType_OpenResult mal_ext_openReadOnly(MalContext *context MAL_DETAIL_MAYBE_UNUSED, MalType_Symbol value) { \
    mal_call_t call = (mal_call_t){ .mal_detail_context = context }; \
    return mal_detail_openReadOnly(&call, (mal_Symbol_t){ .mal_detail_raw = value, .mal_detail_bytes = (mal_span_t){ 0 }, .mal_detail_source = UINT8_C(0) }); \
} \
static MalType_OpenResult mal_detail_openReadOnly( \
    mal_call_t *call, \
    mal_Symbol_t value \
)

#define MAL_HAS_EXTERN_readFile 1
#define MAL_DEFINE_readFile(call, value) \
static MalType_ReadResult mal_detail_readFile(mal_call_t *call, mal_repr_product_4_t value); \
MalType_ReadResult mal_ext_readFile(MalContext *context MAL_DETAIL_MAYBE_UNUSED, MalType_File argument_0, MalType_Region argument_1) { \
    mal_call_t call = (mal_call_t){ .mal_detail_context = context }; \
    return mal_detail_readFile(&call, (mal_repr_product_4_t){ .field_0 = (mal_File_t){ .mal_detail_bits = ((MalRepr_Product_4){ .field_0 = argument_0, .field_1 = argument_1 }).field_0.bits }, .field_1 = (mal_repr_product_3_t){ .field_0 = (mal_Ptr_t)((MalRepr_Product_4){ .field_0 = argument_0, .field_1 = argument_1 }).field_1.field_0.address, .field_1 = ((MalRepr_Product_4){ .field_0 = argument_0, .field_1 = argument_1 }).field_1.field_1 } }); \
} \
static MalType_ReadResult mal_detail_readFile( \
    mal_call_t *call, \
    mal_repr_product_4_t value \
)

#define MAL_HAS_EXTERN_closeFile 1
#define MAL_DEFINE_closeFile(call, value) \
static MalType_CloseResult mal_detail_closeFile(mal_call_t *call, mal_File_t value); \
MalType_CloseResult mal_ext_closeFile(MalContext *context MAL_DETAIL_MAYBE_UNUSED, MalType_File value) { \
    mal_call_t call = (mal_call_t){ .mal_detail_context = context }; \
    return mal_detail_closeFile(&call, (mal_File_t){ .mal_detail_bits = value.bits }); \
} \
static MalType_CloseResult mal_detail_closeFile( \
    mal_call_t *call, \
    mal_File_t value \
)

#define MAL_HAS_EXTERN_writeBytes 1
#define MAL_DEFINE_writeBytes(call, value) \
static MalType_Unit mal_detail_writeBytes(mal_call_t *call, mal_Bytes_t value); \
void mal_ext_writeBytes(MalContext *context MAL_DETAIL_MAYBE_UNUSED, MalType_Ptr argument_0, MalType_UInt64 argument_1) { \
    mal_call_t call = (mal_call_t){ .mal_detail_context = context }; \
    mal_detail_writeBytes(&call, (mal_Bytes_t){ .field_0 = (mal_Ptr_t)((MalRepr_Product_3){ .field_0 = argument_0, .field_1 = argument_1 }).field_0.address, .field_1 = ((MalRepr_Product_3){ .field_0 = argument_0, .field_1 = argument_1 }).field_1 }); \
} \
static MalType_Unit mal_detail_writeBytes( \
    mal_call_t *call, \
    mal_Bytes_t value \
)

#define MAL_HAS_EXTERN_writeSymbol 1
#define MAL_DEFINE_writeSymbol(call, value) \
static MalType_Unit mal_detail_writeSymbol(mal_call_t *call, mal_Symbol_t value); \
void mal_ext_writeSymbol(MalContext *context MAL_DETAIL_MAYBE_UNUSED, MalType_Symbol value) { \
    mal_call_t call = (mal_call_t){ .mal_detail_context = context }; \
    mal_detail_writeSymbol(&call, (mal_Symbol_t){ .mal_detail_raw = value, .mal_detail_bytes = (mal_span_t){ 0 }, .mal_detail_source = UINT8_C(0) }); \
} \
static MalType_Unit mal_detail_writeSymbol( \
    mal_call_t *call, \
    mal_Symbol_t value \
)

#define MAL_HAS_EXTERN_writeError 1
#define MAL_DEFINE_writeError(call, value) \
static MalType_Unit mal_detail_writeError(mal_call_t *call, mal_IoError_t value); \
void mal_ext_writeError(MalContext *context MAL_DETAIL_MAYBE_UNUSED, MalType_IoError value) { \
    mal_call_t call = (mal_call_t){ .mal_detail_context = context }; \
    mal_detail_writeError(&call, value); \
} \
static MalType_Unit mal_detail_writeError( \
    mal_call_t *call, \
    mal_IoError_t value \
)

#endif
