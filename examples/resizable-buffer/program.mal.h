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

typedef struct MalRepr_Product_0 MalRepr_Product_0;
typedef struct MalRepr_Product_1 MalRepr_Product_1;
typedef struct MalRepr_Sum_2 MalRepr_Sum_2;
typedef struct MalRepr_Product_3 MalRepr_Product_3;
typedef struct MalRepr_Product_4 MalRepr_Product_4;
typedef struct MalRepr_Product_5 MalRepr_Product_5;
typedef struct MalRepr_Product_6 MalRepr_Product_6;

struct MalRepr_Product_0 {
    MalType_USize field_0;
    MalType_USize field_1;
};

struct MalRepr_Product_1 {
    MalType_Allocation field_0;
    MalType_Address field_1;
    MalType_USize field_2;
    MalType_USize field_3;
};

struct MalRepr_Sum_2 {
    uint32_t tag;
    union {
        MalRepr_Product_1 variant_0;
        MalType_UInt32 variant_1;
    } payload;
};

struct MalRepr_Product_3 {
    MalRepr_Product_1 field_0;
    MalType_USize field_1;
};

struct MalRepr_Product_4 {
    MalType_Allocation field_0;
    MalType_Address field_1;
    MalType_USize field_2;
};

struct MalRepr_Product_5 {
    MalType_Allocation field_0;
    MalType_Address field_1;
};

struct MalRepr_Product_6 {
    MalType_Address field_0;
    MalType_USize field_1;
};

typedef MalRepr_Product_1 MalType_OwnedBuffer;
typedef MalRepr_Product_4 MalType_BorrowedBytes;
typedef MalRepr_Product_6 MalType_OutputBuffer;
typedef MalRepr_Sum_2 MalType_BufferResult;

typedef struct { uintptr_t mal_detail_bits; } mal_Allocation_t;
typedef struct mal_detail_repr_product_0 mal_repr_product_0_t;
typedef struct mal_detail_repr_product_1 mal_repr_product_1_t;
typedef struct mal_detail_repr_sum_2 mal_repr_sum_2_t;
typedef struct mal_detail_repr_product_3 mal_repr_product_3_t;
typedef struct mal_detail_repr_product_4 mal_repr_product_4_t;
typedef struct mal_detail_repr_product_5 mal_repr_product_5_t;
typedef struct mal_detail_repr_product_6 mal_repr_product_6_t;
typedef mal_repr_product_1_t mal_OwnedBuffer_t;
typedef mal_repr_product_4_t mal_BorrowedBytes_t;
typedef mal_repr_product_6_t mal_OutputBuffer_t;
typedef mal_repr_sum_2_t mal_BufferResult_t;

struct mal_detail_repr_product_0 {
    mal_USize_t field_0;
    mal_USize_t field_1;
};

struct mal_detail_repr_product_1 {
    mal_Allocation_t field_0;
    mal_Address_t field_1;
    mal_USize_t field_2;
    mal_USize_t field_3;
};

struct mal_detail_repr_sum_2 {
    uint32_t tag;
    union {
        mal_repr_product_1_t variant_0;
        mal_UInt32_t variant_1;
    } payload;
};

struct mal_detail_repr_product_3 {
    mal_repr_product_1_t field_0;
    mal_USize_t field_1;
};

struct mal_detail_repr_product_4 {
    mal_Allocation_t field_0;
    mal_Address_t field_1;
    mal_USize_t field_2;
};

struct mal_detail_repr_product_5 {
    mal_Allocation_t field_0;
    mal_Address_t field_1;
};

struct mal_detail_repr_product_6 {
    mal_Address_t field_0;
    mal_USize_t field_1;
};

/* Type helpers */

static inline MalRepr_Product_0 mal_repr_product_0_return(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, mal_repr_product_0_t value) {
    return (MalRepr_Product_0){ .field_0 = value.field_0, .field_1 = value.field_1 };
}

static inline MalRepr_Product_1 mal_repr_product_1_return(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, mal_repr_product_1_t value) {
    return (MalRepr_Product_1){ .field_0 = (MalType_Allocation){ .bits = value.field_0.mal_detail_bits }, .field_1 = mal_Address_return(call, value.field_1), .field_2 = value.field_2, .field_3 = value.field_3 };
}

static inline mal_repr_sum_2_t mal_detail_to_host_2(mal_call_t *call, MalRepr_Sum_2 value) {
    switch (value.tag) {
        case UINT32_C(0): {
            return (mal_repr_sum_2_t){ .tag = UINT32_C(0), .payload.variant_0 = (mal_repr_product_1_t){ .field_0 = (mal_Allocation_t){ .mal_detail_bits = value.payload.variant_0.field_0.bits }, .field_1 = value.payload.variant_0.field_1, .field_2 = value.payload.variant_0.field_2, .field_3 = value.payload.variant_0.field_3 } };
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
            return (MalRepr_Sum_2){ .tag = UINT32_C(0), .payload.variant_0 = (MalRepr_Product_1){ .field_0 = (MalType_Allocation){ .bits = value.payload.variant_0.field_0.mal_detail_bits }, .field_1 = mal_Address_return(call, value.payload.variant_0.field_1), .field_2 = value.payload.variant_0.field_2, .field_3 = value.payload.variant_0.field_3 } };
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
static inline mal_repr_sum_2_t mal_repr_sum_2_make_0(mal_repr_product_1_t value) {
    return (mal_repr_sum_2_t){ .tag = mal_repr_sum_2_tag_0, .payload.variant_0 = value };
}

static inline MalRepr_Sum_2 mal_repr_sum_2_return_0(mal_call_t *call, mal_repr_product_1_t value) {
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
    return (MalRepr_Product_3){ .field_0 = (MalRepr_Product_1){ .field_0 = (MalType_Allocation){ .bits = value.field_0.field_0.mal_detail_bits }, .field_1 = mal_Address_return(call, value.field_0.field_1), .field_2 = value.field_0.field_2, .field_3 = value.field_0.field_3 }, .field_1 = value.field_1 };
}

static inline MalRepr_Product_4 mal_repr_product_4_return(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, mal_repr_product_4_t value) {
    return (MalRepr_Product_4){ .field_0 = (MalType_Allocation){ .bits = value.field_0.mal_detail_bits }, .field_1 = mal_Address_return(call, value.field_1), .field_2 = value.field_2 };
}

static inline MalRepr_Product_5 mal_repr_product_5_return(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, mal_repr_product_5_t value) {
    return (MalRepr_Product_5){ .field_0 = (MalType_Allocation){ .bits = value.field_0.mal_detail_bits }, .field_1 = mal_Address_return(call, value.field_1) };
}

static inline MalRepr_Product_6 mal_repr_product_6_return(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, mal_repr_product_6_t value) {
    return (MalRepr_Product_6){ .field_0 = mal_Address_return(call, value.field_0), .field_1 = value.field_1 };
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

static inline MalType_OwnedBuffer mal_OwnedBuffer_return(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, mal_OwnedBuffer_t value) {
    return (MalRepr_Product_1){ .field_0 = (MalType_Allocation){ .bits = value.field_0.mal_detail_bits }, .field_1 = mal_Address_return(call, value.field_1), .field_2 = value.field_2, .field_3 = value.field_3 };
}

static inline MalType_BorrowedBytes mal_BorrowedBytes_return(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, mal_BorrowedBytes_t value) {
    return (MalRepr_Product_4){ .field_0 = (MalType_Allocation){ .bits = value.field_0.mal_detail_bits }, .field_1 = mal_Address_return(call, value.field_1), .field_2 = value.field_2 };
}

static inline MalType_OutputBuffer mal_OutputBuffer_return(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, mal_OutputBuffer_t value) {
    return (MalRepr_Product_6){ .field_0 = mal_Address_return(call, value.field_0), .field_1 = value.field_1 };
}

#define mal_BufferResult_tag_0 UINT32_C(0)
static inline mal_BufferResult_t mal_BufferResult_make_0(mal_OwnedBuffer_t value) {
    return (mal_BufferResult_t){ .tag = mal_BufferResult_tag_0, .payload.variant_0 = value };
}

static inline MalType_BufferResult mal_BufferResult_return_0(mal_call_t *call, mal_OwnedBuffer_t value) {
    return mal_detail_to_raw_2(call, (mal_BufferResult_t){ .tag = mal_BufferResult_tag_0, .payload.variant_0 = value });
}

#define mal_BufferResult_tag_1 UINT32_C(1)
static inline mal_BufferResult_t mal_BufferResult_make_1(mal_UInt32_t value) {
    return (mal_BufferResult_t){ .tag = mal_BufferResult_tag_1, .payload.variant_1 = value };
}

static inline MalType_BufferResult mal_BufferResult_return_1(mal_call_t *call, mal_UInt32_t value) {
    return mal_detail_to_raw_2(call, (mal_BufferResult_t){ .tag = mal_BufferResult_tag_1, .payload.variant_1 = value });
}

/* Canonical memory access */

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

static inline mal_repr_product_6_t mal_detail_memory_read_6(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, const uint8_t *source MAL_DETAIL_MAYBE_UNUSED) {
    mal_repr_product_6_t value;
    value.field_0 = mal_detail_memory_read_Address(call, source + 0);
    value.field_1 = mal_detail_memory_read_USize(call, source + 8);
    return value;
}

static inline void mal_detail_memory_write_6(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, uint8_t *destination MAL_DETAIL_MAYBE_UNUSED, mal_repr_product_6_t value) {
    mal_detail_memory_write_Address(call, destination + 0, value.field_0);
    mal_detail_memory_write_USize(call, destination + 8, value.field_1);
}

static inline mal_OutputBuffer_t mal_OutputBuffer_read(mal_call_t *call, mal_Address_t address, mal_USize_t index) {
    mal_Address_return(call, address);
    return mal_detail_memory_read_6(call, (const uint8_t *)address + (index * 16));
}

static inline void mal_OutputBuffer_write(mal_call_t *call, mal_Address_t address, mal_USize_t index, mal_OutputBuffer_t value) {
    mal_Address_return(call, address);
    mal_detail_memory_write_6(call, (uint8_t *)address + (index * 16), value);
}

/* External operations */

MalType_BufferResult mal_ext_allocateBuffer(MalContext *context, MalType_USize argument_0, MalType_USize argument_1);
MalType_BufferResult mal_ext_resizeBuffer(MalContext *context, MalType_OwnedBuffer argument_0, MalType_USize argument_1);
void mal_ext_releaseBuffer(MalContext *context, MalType_Allocation value);
MalType_Bool mal_ext_isCurrentBuffer(MalContext *context, MalType_Allocation argument_0, MalType_Address argument_1, MalType_USize argument_2, MalType_USize argument_3);
MalType_Bool mal_ext_isCurrentBorrow(MalContext *context, MalType_Allocation argument_0, MalType_Address argument_1, MalType_USize argument_2);
void mal_ext_writeBorrowedBytes(MalContext *context, MalType_Allocation argument_0, MalType_Address argument_1, MalType_USize argument_2);
void mal_ext_validateStoredDescriptor(MalContext *context, MalType_Allocation argument_0, MalType_Address argument_1);
MalType_OutputBuffer mal_ext_outputBuffer(MalContext *context);
void mal_ext_writeBytes(MalContext *context, MalType_Address argument_0, MalType_USize argument_1);

/* External definition helpers */

#define MAL_HAS_EXTERN_allocateBuffer 1
#define MAL_DEFINE_allocateBuffer(call, value) \
static MalType_BufferResult mal_detail_allocateBuffer(mal_call_t *call, mal_repr_product_0_t value); \
MalType_BufferResult mal_ext_allocateBuffer(MalContext *context MAL_DETAIL_MAYBE_UNUSED, MalType_USize argument_0, MalType_USize argument_1) { \
    mal_call_t call = (mal_call_t){ .mal_detail_context = context }; \
    return mal_detail_allocateBuffer(&call, (mal_repr_product_0_t){ .field_0 = ((MalRepr_Product_0){ .field_0 = argument_0, .field_1 = argument_1 }).field_0, .field_1 = ((MalRepr_Product_0){ .field_0 = argument_0, .field_1 = argument_1 }).field_1 }); \
} \
static MalType_BufferResult mal_detail_allocateBuffer( \
    mal_call_t *call, \
    mal_repr_product_0_t value \
)

#define MAL_HAS_EXTERN_resizeBuffer 1
#define MAL_DEFINE_resizeBuffer(call, value) \
static MalType_BufferResult mal_detail_resizeBuffer(mal_call_t *call, mal_repr_product_3_t value); \
MalType_BufferResult mal_ext_resizeBuffer(MalContext *context MAL_DETAIL_MAYBE_UNUSED, MalType_OwnedBuffer argument_0, MalType_USize argument_1) { \
    mal_call_t call = (mal_call_t){ .mal_detail_context = context }; \
    return mal_detail_resizeBuffer(&call, (mal_repr_product_3_t){ .field_0 = (mal_repr_product_1_t){ .field_0 = (mal_Allocation_t){ .mal_detail_bits = ((MalRepr_Product_3){ .field_0 = argument_0, .field_1 = argument_1 }).field_0.field_0.bits }, .field_1 = ((MalRepr_Product_3){ .field_0 = argument_0, .field_1 = argument_1 }).field_0.field_1, .field_2 = ((MalRepr_Product_3){ .field_0 = argument_0, .field_1 = argument_1 }).field_0.field_2, .field_3 = ((MalRepr_Product_3){ .field_0 = argument_0, .field_1 = argument_1 }).field_0.field_3 }, .field_1 = ((MalRepr_Product_3){ .field_0 = argument_0, .field_1 = argument_1 }).field_1 }); \
} \
static MalType_BufferResult mal_detail_resizeBuffer( \
    mal_call_t *call, \
    mal_repr_product_3_t value \
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

#define MAL_HAS_EXTERN_isCurrentBuffer 1
#define MAL_DEFINE_isCurrentBuffer(call, value) \
static MalType_Bool mal_detail_isCurrentBuffer(mal_call_t *call, mal_OwnedBuffer_t value); \
MalType_Bool mal_ext_isCurrentBuffer(MalContext *context MAL_DETAIL_MAYBE_UNUSED, MalType_Allocation argument_0, MalType_Address argument_1, MalType_USize argument_2, MalType_USize argument_3) { \
    mal_call_t call = (mal_call_t){ .mal_detail_context = context }; \
    return mal_detail_isCurrentBuffer(&call, (mal_OwnedBuffer_t){ .field_0 = (mal_Allocation_t){ .mal_detail_bits = ((MalRepr_Product_1){ .field_0 = argument_0, .field_1 = argument_1, .field_2 = argument_2, .field_3 = argument_3 }).field_0.bits }, .field_1 = ((MalRepr_Product_1){ .field_0 = argument_0, .field_1 = argument_1, .field_2 = argument_2, .field_3 = argument_3 }).field_1, .field_2 = ((MalRepr_Product_1){ .field_0 = argument_0, .field_1 = argument_1, .field_2 = argument_2, .field_3 = argument_3 }).field_2, .field_3 = ((MalRepr_Product_1){ .field_0 = argument_0, .field_1 = argument_1, .field_2 = argument_2, .field_3 = argument_3 }).field_3 }); \
} \
static MalType_Bool mal_detail_isCurrentBuffer( \
    mal_call_t *call, \
    mal_OwnedBuffer_t value \
)

#define MAL_HAS_EXTERN_isCurrentBorrow 1
#define MAL_DEFINE_isCurrentBorrow(call, value) \
static MalType_Bool mal_detail_isCurrentBorrow(mal_call_t *call, mal_BorrowedBytes_t value); \
MalType_Bool mal_ext_isCurrentBorrow(MalContext *context MAL_DETAIL_MAYBE_UNUSED, MalType_Allocation argument_0, MalType_Address argument_1, MalType_USize argument_2) { \
    mal_call_t call = (mal_call_t){ .mal_detail_context = context }; \
    return mal_detail_isCurrentBorrow(&call, (mal_BorrowedBytes_t){ .field_0 = (mal_Allocation_t){ .mal_detail_bits = ((MalRepr_Product_4){ .field_0 = argument_0, .field_1 = argument_1, .field_2 = argument_2 }).field_0.bits }, .field_1 = ((MalRepr_Product_4){ .field_0 = argument_0, .field_1 = argument_1, .field_2 = argument_2 }).field_1, .field_2 = ((MalRepr_Product_4){ .field_0 = argument_0, .field_1 = argument_1, .field_2 = argument_2 }).field_2 }); \
} \
static MalType_Bool mal_detail_isCurrentBorrow( \
    mal_call_t *call, \
    mal_BorrowedBytes_t value \
)

#define MAL_HAS_EXTERN_writeBorrowedBytes 1
#define MAL_DEFINE_writeBorrowedBytes(call, value) \
static MalType_Unit mal_detail_writeBorrowedBytes(mal_call_t *call, mal_BorrowedBytes_t value); \
void mal_ext_writeBorrowedBytes(MalContext *context MAL_DETAIL_MAYBE_UNUSED, MalType_Allocation argument_0, MalType_Address argument_1, MalType_USize argument_2) { \
    mal_call_t call = (mal_call_t){ .mal_detail_context = context }; \
    mal_detail_writeBorrowedBytes(&call, (mal_BorrowedBytes_t){ .field_0 = (mal_Allocation_t){ .mal_detail_bits = ((MalRepr_Product_4){ .field_0 = argument_0, .field_1 = argument_1, .field_2 = argument_2 }).field_0.bits }, .field_1 = ((MalRepr_Product_4){ .field_0 = argument_0, .field_1 = argument_1, .field_2 = argument_2 }).field_1, .field_2 = ((MalRepr_Product_4){ .field_0 = argument_0, .field_1 = argument_1, .field_2 = argument_2 }).field_2 }); \
} \
static MalType_Unit mal_detail_writeBorrowedBytes( \
    mal_call_t *call, \
    mal_BorrowedBytes_t value \
)

#define MAL_HAS_EXTERN_validateStoredDescriptor 1
#define MAL_DEFINE_validateStoredDescriptor(call, value) \
static MalType_Unit mal_detail_validateStoredDescriptor(mal_call_t *call, mal_repr_product_5_t value); \
void mal_ext_validateStoredDescriptor(MalContext *context MAL_DETAIL_MAYBE_UNUSED, MalType_Allocation argument_0, MalType_Address argument_1) { \
    mal_call_t call = (mal_call_t){ .mal_detail_context = context }; \
    mal_detail_validateStoredDescriptor(&call, (mal_repr_product_5_t){ .field_0 = (mal_Allocation_t){ .mal_detail_bits = ((MalRepr_Product_5){ .field_0 = argument_0, .field_1 = argument_1 }).field_0.bits }, .field_1 = ((MalRepr_Product_5){ .field_0 = argument_0, .field_1 = argument_1 }).field_1 }); \
} \
static MalType_Unit mal_detail_validateStoredDescriptor( \
    mal_call_t *call, \
    mal_repr_product_5_t value \
)

#define MAL_HAS_EXTERN_outputBuffer 1
#define MAL_DEFINE_outputBuffer(call) \
static MalType_OutputBuffer mal_detail_outputBuffer(mal_call_t *call); \
MalType_OutputBuffer mal_ext_outputBuffer(MalContext *context MAL_DETAIL_MAYBE_UNUSED) { \
    mal_call_t call = (mal_call_t){ .mal_detail_context = context }; \
    return mal_detail_outputBuffer(&call); \
} \
static MalType_OutputBuffer mal_detail_outputBuffer( \
    mal_call_t *call \
)

#define MAL_HAS_EXTERN_writeBytes 1
#define MAL_DEFINE_writeBytes(call, value) \
static MalType_Unit mal_detail_writeBytes(mal_call_t *call, mal_OutputBuffer_t value); \
void mal_ext_writeBytes(MalContext *context MAL_DETAIL_MAYBE_UNUSED, MalType_Address argument_0, MalType_USize argument_1) { \
    mal_call_t call = (mal_call_t){ .mal_detail_context = context }; \
    mal_detail_writeBytes(&call, (mal_OutputBuffer_t){ .field_0 = ((MalRepr_Product_6){ .field_0 = argument_0, .field_1 = argument_1 }).field_0, .field_1 = ((MalRepr_Product_6){ .field_0 = argument_0, .field_1 = argument_1 }).field_1 }); \
} \
static MalType_Unit mal_detail_writeBytes( \
    mal_call_t *call, \
    mal_OutputBuffer_t value \
)

#endif
