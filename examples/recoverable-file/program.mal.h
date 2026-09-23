#ifndef MAL_PROGRAM_MAL_H
#define MAL_PROGRAM_MAL_H

#include <stddef.h>
#include <stdint.h>
#include <limits.h>
#include <string.h>

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
typedef struct { uintptr_t bits; } MalType_File;

typedef struct MalRepr_Product_0 MalRepr_Product_0;
typedef struct MalRepr_Product_1 MalRepr_Product_1;
typedef struct MalRepr_Product_2 MalRepr_Product_2;
typedef struct MalRepr_Sum_3 MalRepr_Sum_3;
typedef struct MalRepr_Product_4 MalRepr_Product_4;
typedef struct MalRepr_Sum_5 MalRepr_Sum_5;
typedef struct MalRepr_Sum_6 MalRepr_Sum_6;

struct MalRepr_Product_0 {
    MalType_Address field_0;
    MalType_USize field_1;
    MalType_USize field_2;
};

struct MalRepr_Product_1 {
    MalType_Allocation field_0;
    MalRepr_Product_0 field_1;
};

struct MalRepr_Product_2 {
    MalType_Address field_0;
    MalType_USize field_1;
};

struct MalRepr_Sum_3 {
    uint32_t tag;
    union {
        MalType_File variant_0;
        MalType_UInt32 variant_1;
    } payload;
};

struct MalRepr_Product_4 {
    MalType_File field_0;
    MalRepr_Product_2 field_1;
};

struct MalRepr_Sum_5 {
    uint32_t tag;
    union {
        MalType_USize variant_0;
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

typedef MalRepr_Product_0 MalType_ByteBuffer;
typedef MalRepr_Product_2 MalType_WritableBytes;
typedef MalType_UInt32 MalType_IoError;
typedef MalRepr_Product_1 MalType_OwnedBuffer;
typedef MalRepr_Sum_3 MalType_OpenResult;
typedef MalRepr_Sum_5 MalType_ReadResult;
typedef MalRepr_Sum_6 MalType_CloseResult;

typedef struct { uintptr_t mal_detail_bits; } mal_Allocation_t;
typedef struct { uintptr_t mal_detail_bits; } mal_File_t;
typedef struct mal_detail_repr_product_0 mal_repr_product_0_t;
typedef struct mal_detail_repr_product_1 mal_repr_product_1_t;
typedef struct mal_detail_repr_product_2 mal_repr_product_2_t;
typedef struct mal_detail_repr_sum_3 mal_repr_sum_3_t;
typedef struct mal_detail_repr_product_4 mal_repr_product_4_t;
typedef struct mal_detail_repr_sum_5 mal_repr_sum_5_t;
typedef struct mal_detail_repr_sum_6 mal_repr_sum_6_t;
typedef struct mal_detail_repr_product_7 mal_repr_product_7_t;
typedef mal_repr_product_0_t mal_ByteBuffer_t;
typedef mal_repr_product_2_t mal_WritableBytes_t;
typedef mal_UInt32_t mal_IoError_t;
typedef mal_repr_product_1_t mal_OwnedBuffer_t;
typedef mal_repr_sum_3_t mal_OpenResult_t;
typedef mal_repr_sum_5_t mal_ReadResult_t;
typedef mal_repr_sum_6_t mal_CloseResult_t;
typedef mal_repr_sum_6_t mal_CopyResult_t;
typedef mal_repr_product_7_t mal_Arguments_t;

struct mal_detail_repr_product_0 {
    mal_Address_t field_0;
    mal_USize_t field_1;
    mal_USize_t field_2;
};

struct mal_detail_repr_product_1 {
    mal_Allocation_t field_0;
    mal_repr_product_0_t field_1;
};

struct mal_detail_repr_product_2 {
    mal_Address_t field_0;
    mal_USize_t field_1;
};

struct mal_detail_repr_sum_3 {
    uint32_t tag;
    union {
        mal_File_t variant_0;
        mal_UInt32_t variant_1;
    } payload;
};

struct mal_detail_repr_product_4 {
    mal_File_t field_0;
    mal_repr_product_2_t field_1;
};

struct mal_detail_repr_sum_5 {
    uint32_t tag;
    union {
        mal_USize_t variant_0;
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

struct mal_detail_repr_product_7 {
    mal_USize_t field_0;
    mal_Address_t field_1;
};

/* Type helpers */

static inline MalRepr_Product_0 mal_repr_product_0_return(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, mal_repr_product_0_t value) {
    return (MalRepr_Product_0){ .field_0 = mal_Address_return(call, value.field_0), .field_1 = value.field_1, .field_2 = value.field_2 };
}

static inline MalRepr_Product_1 mal_repr_product_1_return(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, mal_repr_product_1_t value) {
    return (MalRepr_Product_1){ .field_0 = (MalType_Allocation){ .bits = value.field_0.mal_detail_bits }, .field_1 = (MalRepr_Product_0){ .field_0 = mal_Address_return(call, value.field_1.field_0), .field_1 = value.field_1.field_1, .field_2 = value.field_1.field_2 } };
}

static inline MalRepr_Product_2 mal_repr_product_2_return(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, mal_repr_product_2_t value) {
    return (MalRepr_Product_2){ .field_0 = mal_Address_return(call, value.field_0), .field_1 = value.field_1 };
}

static inline mal_repr_sum_3_t mal_detail_to_host_3(mal_call_t *call, MalRepr_Sum_3 value) {
    switch (value.tag) {
        case UINT32_C(0): {
            return (mal_repr_sum_3_t){ .tag = UINT32_C(0), .payload.variant_0 = (mal_File_t){ .mal_detail_bits = value.payload.variant_0.bits } };
        }
        case UINT32_C(1): {
            return (mal_repr_sum_3_t){ .tag = UINT32_C(1), .payload.variant_1 = value.payload.variant_1 };
        }
        default: {
            mal_call_trap(call, "invalid sum tag");
        }
    }
}

static inline MalRepr_Sum_3 mal_detail_to_raw_3(mal_call_t *call, mal_repr_sum_3_t value) {
    switch (value.tag) {
        case UINT32_C(0): {
            return (MalRepr_Sum_3){ .tag = UINT32_C(0), .payload.variant_0 = (MalType_File){ .bits = value.payload.variant_0.mal_detail_bits } };
        }
        case UINT32_C(1): {
            return (MalRepr_Sum_3){ .tag = UINT32_C(1), .payload.variant_1 = value.payload.variant_1 };
        }
        default: {
            mal_call_trap(call, "invalid sum tag");
        }
    }
}

#define mal_repr_sum_3_tag_0 UINT32_C(0)
static inline mal_repr_sum_3_t mal_repr_sum_3_make_0(mal_File_t value) {
    return (mal_repr_sum_3_t){ .tag = mal_repr_sum_3_tag_0, .payload.variant_0 = value };
}

static inline MalRepr_Sum_3 mal_repr_sum_3_return_0(mal_call_t *call, mal_File_t value) {
    return mal_detail_to_raw_3(call, (mal_repr_sum_3_t){ .tag = mal_repr_sum_3_tag_0, .payload.variant_0 = value });
}

#define mal_repr_sum_3_tag_1 UINT32_C(1)
static inline mal_repr_sum_3_t mal_repr_sum_3_make_1(mal_UInt32_t value) {
    return (mal_repr_sum_3_t){ .tag = mal_repr_sum_3_tag_1, .payload.variant_1 = value };
}

static inline MalRepr_Sum_3 mal_repr_sum_3_return_1(mal_call_t *call, mal_UInt32_t value) {
    return mal_detail_to_raw_3(call, (mal_repr_sum_3_t){ .tag = mal_repr_sum_3_tag_1, .payload.variant_1 = value });
}

static inline MalRepr_Product_4 mal_repr_product_4_return(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, mal_repr_product_4_t value) {
    return (MalRepr_Product_4){ .field_0 = (MalType_File){ .bits = value.field_0.mal_detail_bits }, .field_1 = (MalRepr_Product_2){ .field_0 = mal_Address_return(call, value.field_1.field_0), .field_1 = value.field_1.field_1 } };
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
static inline mal_repr_sum_5_t mal_repr_sum_5_make_0(mal_USize_t value) {
    return (mal_repr_sum_5_t){ .tag = mal_repr_sum_5_tag_0, .payload.variant_0 = value };
}

static inline MalRepr_Sum_5 mal_repr_sum_5_return_0(mal_call_t *call, mal_USize_t value) {
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

static inline MalType_ByteBuffer mal_ByteBuffer_return(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, mal_ByteBuffer_t value) {
    return (MalRepr_Product_0){ .field_0 = mal_Address_return(call, value.field_0), .field_1 = value.field_1, .field_2 = value.field_2 };
}

static inline MalType_WritableBytes mal_WritableBytes_return(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, mal_WritableBytes_t value) {
    return (MalRepr_Product_2){ .field_0 = mal_Address_return(call, value.field_0), .field_1 = value.field_1 };
}

static inline MalType_IoError mal_IoError_return(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, mal_IoError_t value) {
    return value;
}

static inline MalType_OwnedBuffer mal_OwnedBuffer_return(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, mal_OwnedBuffer_t value) {
    return (MalRepr_Product_1){ .field_0 = (MalType_Allocation){ .bits = value.field_0.mal_detail_bits }, .field_1 = (MalRepr_Product_0){ .field_0 = mal_Address_return(call, value.field_1.field_0), .field_1 = value.field_1.field_1, .field_2 = value.field_1.field_2 } };
}

#define mal_OpenResult_tag_0 UINT32_C(0)
static inline mal_OpenResult_t mal_OpenResult_make_0(mal_File_t value) {
    return (mal_OpenResult_t){ .tag = mal_OpenResult_tag_0, .payload.variant_0 = value };
}

static inline MalType_OpenResult mal_OpenResult_return_0(mal_call_t *call, mal_File_t value) {
    return mal_detail_to_raw_3(call, (mal_OpenResult_t){ .tag = mal_OpenResult_tag_0, .payload.variant_0 = value });
}

#define mal_OpenResult_tag_1 UINT32_C(1)
static inline mal_OpenResult_t mal_OpenResult_make_1(mal_IoError_t value) {
    return (mal_OpenResult_t){ .tag = mal_OpenResult_tag_1, .payload.variant_1 = value };
}

static inline MalType_OpenResult mal_OpenResult_return_1(mal_call_t *call, mal_IoError_t value) {
    return mal_detail_to_raw_3(call, (mal_OpenResult_t){ .tag = mal_OpenResult_tag_1, .payload.variant_1 = value });
}

#define mal_ReadResult_tag_0 UINT32_C(0)
static inline mal_ReadResult_t mal_ReadResult_make_0(mal_USize_t value) {
    return (mal_ReadResult_t){ .tag = mal_ReadResult_tag_0, .payload.variant_0 = value };
}

static inline MalType_ReadResult mal_ReadResult_return_0(mal_call_t *call, mal_USize_t value) {
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

/* Canonical memory access */

static inline mal_UInt8_t mal_detail_memory_read_UInt8(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, const uint8_t *source) {
    mal_UInt8_t value;
    memcpy(&value, source, sizeof(value));
    return value;
}

static inline void mal_detail_memory_write_UInt8(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, uint8_t *destination, mal_UInt8_t value) {
    memcpy(destination, &value, sizeof(value));
}

static inline mal_UInt32_t mal_detail_memory_read_UInt32(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, const uint8_t *source) {
    mal_UInt32_t value;
    memcpy(&value, source, sizeof(value));
    return value;
}

static inline void mal_detail_memory_write_UInt32(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, uint8_t *destination, mal_UInt32_t value) {
    memcpy(destination, &value, sizeof(value));
}

static inline mal_Address_t mal_detail_memory_read_Address(mal_call_t *call, const uint8_t *source) {
    mal_Address_t value;
    memcpy(&value, source, sizeof(value));
    return mal_Address_return(call, value);
}

static inline void mal_detail_memory_write_Address(mal_call_t *call, uint8_t *destination, mal_Address_t value) {
    mal_Address_return(call, value);
    memcpy(destination, &value, sizeof(value));
}

static inline mal_USize_t mal_detail_memory_read_USize(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, const uint8_t *source) {
    mal_USize_t value;
    memcpy(&value, source, sizeof(value));
    return value;
}

static inline void mal_detail_memory_write_USize(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, uint8_t *destination, mal_USize_t value) {
    memcpy(destination, &value, sizeof(value));
}

static inline mal_repr_product_0_t mal_detail_memory_read_0(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, const uint8_t *source MAL_DETAIL_MAYBE_UNUSED) {
    mal_repr_product_0_t value;
    value.field_0 = mal_detail_memory_read_Address(call, source + 0);
    value.field_1 = mal_detail_memory_read_USize(call, source + 8);
    value.field_2 = mal_detail_memory_read_USize(call, source + 16);
    return value;
}

static inline void mal_detail_memory_write_0(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, uint8_t *destination MAL_DETAIL_MAYBE_UNUSED, mal_repr_product_0_t value) {
    mal_detail_memory_write_Address(call, destination + 0, value.field_0);
    mal_detail_memory_write_USize(call, destination + 8, value.field_1);
    mal_detail_memory_write_USize(call, destination + 16, value.field_2);
}

static inline mal_repr_product_2_t mal_detail_memory_read_2(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, const uint8_t *source MAL_DETAIL_MAYBE_UNUSED) {
    mal_repr_product_2_t value;
    value.field_0 = mal_detail_memory_read_Address(call, source + 0);
    value.field_1 = mal_detail_memory_read_USize(call, source + 8);
    return value;
}

static inline void mal_detail_memory_write_2(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, uint8_t *destination MAL_DETAIL_MAYBE_UNUSED, mal_repr_product_2_t value) {
    mal_detail_memory_write_Address(call, destination + 0, value.field_0);
    mal_detail_memory_write_USize(call, destination + 8, value.field_1);
}

static inline mal_repr_sum_5_t mal_detail_memory_read_5(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, const uint8_t *source MAL_DETAIL_MAYBE_UNUSED) {
    switch (mal_detail_memory_read_UInt8(call, source)) {
        case 0: {
            return (mal_repr_sum_5_t){ .tag = UINT32_C(0), .payload.variant_0 = mal_detail_memory_read_USize(call, source + 8) };
        }
        case 1: {
            return (mal_repr_sum_5_t){ .tag = UINT32_C(1), .payload.variant_1 = mal_detail_memory_read_UInt32(call, source + 8) };
        }
        default: {
            mal_call_trap(call, "invalid canonical sum tag");
        }
    }
}

static inline void mal_detail_memory_write_5(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, uint8_t *destination MAL_DETAIL_MAYBE_UNUSED, mal_repr_sum_5_t value) {
    switch (value.tag) {
        case UINT32_C(0): {
            mal_detail_memory_write_UInt8(call, destination, (mal_UInt8_t)value.tag);
            mal_detail_memory_write_USize(call, destination + 8, value.payload.variant_0);
            return;
        }
        case UINT32_C(1): {
            mal_detail_memory_write_UInt8(call, destination, (mal_UInt8_t)value.tag);
            mal_detail_memory_write_UInt32(call, destination + 8, value.payload.variant_1);
            return;
        }
        default: {
            mal_call_trap(call, "invalid sum tag");
        }
    }
}

static inline mal_repr_sum_6_t mal_detail_memory_read_6(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, const uint8_t *source MAL_DETAIL_MAYBE_UNUSED) {
    switch (mal_detail_memory_read_UInt8(call, source)) {
        case 0: {
            return (mal_repr_sum_6_t){ .tag = UINT32_C(0), .payload.variant_0 = (mal_Unit_t){ 0 } };
        }
        case 1: {
            return (mal_repr_sum_6_t){ .tag = UINT32_C(1), .payload.variant_1 = mal_detail_memory_read_UInt32(call, source + 4) };
        }
        default: {
            mal_call_trap(call, "invalid canonical sum tag");
        }
    }
}

static inline void mal_detail_memory_write_6(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, uint8_t *destination MAL_DETAIL_MAYBE_UNUSED, mal_repr_sum_6_t value) {
    switch (value.tag) {
        case UINT32_C(0): {
            mal_detail_memory_write_UInt8(call, destination, (mal_UInt8_t)value.tag);
            (void)value.payload.variant_0;
            return;
        }
        case UINT32_C(1): {
            mal_detail_memory_write_UInt8(call, destination, (mal_UInt8_t)value.tag);
            mal_detail_memory_write_UInt32(call, destination + 4, value.payload.variant_1);
            return;
        }
        default: {
            mal_call_trap(call, "invalid sum tag");
        }
    }
}

static inline mal_repr_product_7_t mal_detail_memory_read_7(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, const uint8_t *source MAL_DETAIL_MAYBE_UNUSED) {
    mal_repr_product_7_t value;
    value.field_0 = mal_detail_memory_read_USize(call, source + 0);
    value.field_1 = mal_detail_memory_read_Address(call, source + 8);
    return value;
}

static inline void mal_detail_memory_write_7(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, uint8_t *destination MAL_DETAIL_MAYBE_UNUSED, mal_repr_product_7_t value) {
    mal_detail_memory_write_USize(call, destination + 0, value.field_0);
    mal_detail_memory_write_Address(call, destination + 8, value.field_1);
}

static inline mal_ByteBuffer_t mal_ByteBuffer_read(mal_call_t *call, mal_Address_t address, mal_USize_t index) {
    mal_Address_return(call, address);
    return mal_detail_memory_read_0(call, (const uint8_t *)address + (index * 24));
}

static inline void mal_ByteBuffer_write(mal_call_t *call, mal_Address_t address, mal_USize_t index, mal_ByteBuffer_t value) {
    mal_Address_return(call, address);
    mal_detail_memory_write_0(call, (uint8_t *)address + (index * 24), value);
}

static inline mal_WritableBytes_t mal_WritableBytes_read(mal_call_t *call, mal_Address_t address, mal_USize_t index) {
    mal_Address_return(call, address);
    return mal_detail_memory_read_2(call, (const uint8_t *)address + (index * 16));
}

static inline void mal_WritableBytes_write(mal_call_t *call, mal_Address_t address, mal_USize_t index, mal_WritableBytes_t value) {
    mal_Address_return(call, address);
    mal_detail_memory_write_2(call, (uint8_t *)address + (index * 16), value);
}

static inline mal_IoError_t mal_IoError_read(mal_call_t *call, mal_Address_t address, mal_USize_t index) {
    mal_Address_return(call, address);
    return mal_detail_memory_read_UInt32(call, (const uint8_t *)address + (index * 4));
}

static inline void mal_IoError_write(mal_call_t *call, mal_Address_t address, mal_USize_t index, mal_IoError_t value) {
    mal_Address_return(call, address);
    mal_detail_memory_write_UInt32(call, (uint8_t *)address + (index * 4), value);
}

static inline mal_ReadResult_t mal_ReadResult_read(mal_call_t *call, mal_Address_t address, mal_USize_t index) {
    mal_Address_return(call, address);
    return mal_detail_memory_read_5(call, (const uint8_t *)address + (index * 16));
}

static inline void mal_ReadResult_write(mal_call_t *call, mal_Address_t address, mal_USize_t index, mal_ReadResult_t value) {
    mal_Address_return(call, address);
    mal_detail_memory_write_5(call, (uint8_t *)address + (index * 16), value);
}

static inline mal_CloseResult_t mal_CloseResult_read(mal_call_t *call, mal_Address_t address, mal_USize_t index) {
    mal_Address_return(call, address);
    return mal_detail_memory_read_6(call, (const uint8_t *)address + (index * 8));
}

static inline void mal_CloseResult_write(mal_call_t *call, mal_Address_t address, mal_USize_t index, mal_CloseResult_t value) {
    mal_Address_return(call, address);
    mal_detail_memory_write_6(call, (uint8_t *)address + (index * 8), value);
}

static inline mal_CopyResult_t mal_CopyResult_read(mal_call_t *call, mal_Address_t address, mal_USize_t index) {
    mal_Address_return(call, address);
    return mal_detail_memory_read_6(call, (const uint8_t *)address + (index * 8));
}

static inline void mal_CopyResult_write(mal_call_t *call, mal_Address_t address, mal_USize_t index, mal_CopyResult_t value) {
    mal_Address_return(call, address);
    mal_detail_memory_write_6(call, (uint8_t *)address + (index * 8), value);
}

static inline mal_Arguments_t mal_Arguments_read(mal_call_t *call, mal_Address_t address, mal_USize_t index) {
    mal_Address_return(call, address);
    return mal_detail_memory_read_7(call, (const uint8_t *)address + (index * 16));
}

static inline void mal_Arguments_write(mal_call_t *call, mal_Address_t address, mal_USize_t index, mal_Arguments_t value) {
    mal_Address_return(call, address);
    mal_detail_memory_write_7(call, (uint8_t *)address + (index * 16), value);
}

/* External operations */

MalType_OwnedBuffer mal_ext_allocateBuffer(MalContext *context, MalType_USize value);
void mal_ext_releaseBuffer(MalContext *context, MalType_Allocation value);
MalType_OpenResult mal_ext_openReadOnly(MalContext *context, MalType_Address argument_0, MalType_USize argument_1);
MalType_ReadResult mal_ext_readFile(MalContext *context, MalType_File argument_0, MalType_WritableBytes argument_1);
MalType_CloseResult mal_ext_closeFile(MalContext *context, MalType_File value);
void mal_ext_writeBytes(MalContext *context, MalType_Address argument_0, MalType_USize argument_1);
void mal_ext_writeError(MalContext *context, MalType_IoError value);
MalType_USize mal_ext_argumentLength(MalContext *context, MalType_Address value);

/* External definition helpers */

#define MAL_HAS_EXTERN_allocateBuffer 1
#define MAL_DEFINE_allocateBuffer(call, value) \
static MalType_OwnedBuffer mal_detail_allocateBuffer(mal_call_t *call, mal_USize_t value); \
MalType_OwnedBuffer mal_ext_allocateBuffer(MalContext *context MAL_DETAIL_MAYBE_UNUSED, MalType_USize value) { \
    mal_call_t call = (mal_call_t){ .mal_detail_context = context }; \
    return mal_detail_allocateBuffer(&call, value); \
} \
static MalType_OwnedBuffer mal_detail_allocateBuffer( \
    mal_call_t *call, \
    mal_USize_t value \
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
static MalType_OpenResult mal_detail_openReadOnly(mal_call_t *call, mal_repr_product_2_t value); \
MalType_OpenResult mal_ext_openReadOnly(MalContext *context MAL_DETAIL_MAYBE_UNUSED, MalType_Address argument_0, MalType_USize argument_1) { \
    mal_call_t call = (mal_call_t){ .mal_detail_context = context }; \
    return mal_detail_openReadOnly(&call, (mal_repr_product_2_t){ .field_0 = ((MalRepr_Product_2){ .field_0 = argument_0, .field_1 = argument_1 }).field_0, .field_1 = ((MalRepr_Product_2){ .field_0 = argument_0, .field_1 = argument_1 }).field_1 }); \
} \
static MalType_OpenResult mal_detail_openReadOnly( \
    mal_call_t *call, \
    mal_repr_product_2_t value \
)

#define MAL_HAS_EXTERN_readFile 1
#define MAL_DEFINE_readFile(call, value) \
static MalType_ReadResult mal_detail_readFile(mal_call_t *call, mal_repr_product_4_t value); \
MalType_ReadResult mal_ext_readFile(MalContext *context MAL_DETAIL_MAYBE_UNUSED, MalType_File argument_0, MalType_WritableBytes argument_1) { \
    mal_call_t call = (mal_call_t){ .mal_detail_context = context }; \
    return mal_detail_readFile(&call, (mal_repr_product_4_t){ .field_0 = (mal_File_t){ .mal_detail_bits = ((MalRepr_Product_4){ .field_0 = argument_0, .field_1 = argument_1 }).field_0.bits }, .field_1 = (mal_repr_product_2_t){ .field_0 = ((MalRepr_Product_4){ .field_0 = argument_0, .field_1 = argument_1 }).field_1.field_0, .field_1 = ((MalRepr_Product_4){ .field_0 = argument_0, .field_1 = argument_1 }).field_1.field_1 } }); \
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
static MalType_Unit mal_detail_writeBytes(mal_call_t *call, mal_repr_product_2_t value); \
void mal_ext_writeBytes(MalContext *context MAL_DETAIL_MAYBE_UNUSED, MalType_Address argument_0, MalType_USize argument_1) { \
    mal_call_t call = (mal_call_t){ .mal_detail_context = context }; \
    mal_detail_writeBytes(&call, (mal_repr_product_2_t){ .field_0 = ((MalRepr_Product_2){ .field_0 = argument_0, .field_1 = argument_1 }).field_0, .field_1 = ((MalRepr_Product_2){ .field_0 = argument_0, .field_1 = argument_1 }).field_1 }); \
} \
static MalType_Unit mal_detail_writeBytes( \
    mal_call_t *call, \
    mal_repr_product_2_t value \
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

#define MAL_HAS_EXTERN_argumentLength 1
#define MAL_DEFINE_argumentLength(call, value) \
static MalType_USize mal_detail_argumentLength(mal_call_t *call, mal_Address_t value); \
MalType_USize mal_ext_argumentLength(MalContext *context MAL_DETAIL_MAYBE_UNUSED, MalType_Address value) { \
    mal_call_t call = (mal_call_t){ .mal_detail_context = context }; \
    return mal_detail_argumentLength(&call, value); \
} \
static MalType_USize mal_detail_argumentLength( \
    mal_call_t *call, \
    mal_Address_t value \
)

#endif
