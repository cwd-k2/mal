#ifndef MAL_PROGRAM_MAL_H
#define MAL_PROGRAM_MAL_H

#include <stdint.h>

#define MAL_C_ABI_VERSION 0x000500u
#define MAL_TYPE(name) MalType_##name
#define MAL_OPERATION(type, operation) mal_##type##_##operation
#define MAL_TAG(type, variant) MAL_##type##_TAG_##variant
#define MAL_EXTERN(name) mal_ext_##name
#define MAL_CLONE(owner) mal_##owner##_clone
#define MAL_MOVE(owner) mal_##owner##_take
#define MAL_DROP(owner) mal_##owner##_drop

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
typedef struct { void *state; } MalSymbolAdmission;

#define MAL_FALSE (MalType_Bool)UINT8_C(0)
#define MAL_TRUE (MalType_Bool)UINT8_C(1)

_Noreturn void mal_trap(MalContext *context, const char *message);
MalSymbolAdmission mal_SymbolAdmission_begin(MalContext *context, uint64_t minimum_capacity);
uint64_t mal_SymbolAdmission_capacity(const MalSymbolAdmission *admission);
uint8_t *mal_SymbolAdmission_data(MalSymbolAdmission *admission);
void mal_SymbolAdmission_reserve(MalContext *context, MalSymbolAdmission *admission, uint64_t minimum_capacity);
MalType_Symbol mal_SymbolAdmission_finish(MalContext *context, MalSymbolAdmission *admission, uint64_t length);
void mal_SymbolAdmission_drop(MalContext *context, MalSymbolAdmission *admission);
MalType_Symbol mal_Symbol_clone(MalContext *context, MalType_Symbol value);
MalType_Symbol mal_Symbol_take(MalType_Symbol *value);
void mal_Symbol_drop(MalContext *context, MalType_Symbol *value);

static inline const uint8_t *mal_Symbol_data(MalType_Symbol value) {
    return value.data;
}

static inline uint64_t mal_Symbol_length(MalType_Symbol value) {
    return value.length;
}

static inline MalType_Ptr mal_Ptr_from_address(uint8_t *address) {
    return (MalType_Ptr){ .address = address };
}

static inline uint8_t *mal_Ptr_address(MalType_Ptr value) {
    return value.address;
}

/* Host-visible types */

typedef struct { uintptr_t bits; } MalType_Allocation;

typedef struct MalRepr_Product_0 MalRepr_Product_0;
typedef struct MalRepr_Product_1 MalRepr_Product_1;
typedef struct MalRepr_Sum_2 MalRepr_Sum_2;
typedef struct MalRepr_Product_3 MalRepr_Product_3;
typedef struct MalRepr_Product_4 MalRepr_Product_4;
typedef struct MalRepr_Product_5 MalRepr_Product_5;

struct MalRepr_Product_0 {
    MalType_UInt64 field_0;
    MalType_UInt64 field_1;
};

struct MalRepr_Product_1 {
    MalType_Allocation field_0;
    MalType_Ptr field_1;
    MalType_UInt64 field_2;
    MalType_UInt64 field_3;
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
    MalType_UInt64 field_1;
};

struct MalRepr_Product_4 {
    MalType_Allocation field_0;
    MalType_Ptr field_1;
    MalType_UInt64 field_2;
};

struct MalRepr_Product_5 {
    MalType_Allocation field_0;
    MalType_Ptr field_1;
};

typedef MalRepr_Product_1 MalType_Buffer;
typedef MalRepr_Product_4 MalType_Slice;
typedef MalRepr_Sum_2 MalType_BufferResult;

/* Type helpers */

static inline MalType_Allocation mal_Allocation_from_bits(uintptr_t bits) {
    return (MalType_Allocation){ .bits = bits };
}

static inline uintptr_t mal_Allocation_bits(MalType_Allocation value) {
    return value.bits;
}

static inline MalType_Buffer mal_Buffer_make(MalType_Allocation value_0, MalType_Ptr value_1, MalType_UInt64 value_2, MalType_UInt64 value_3) {
    return (MalType_Buffer){ .field_0 = value_0, .field_1 = value_1, .field_2 = value_2, .field_3 = value_3 };
}

static inline MalType_Allocation mal_Buffer_get_0(MalType_Buffer value) {
    return value.field_0;
}

static inline MalType_Ptr mal_Buffer_get_1(MalType_Buffer value) {
    return value.field_1;
}

static inline MalType_UInt64 mal_Buffer_get_2(MalType_Buffer value) {
    return value.field_2;
}

static inline MalType_UInt64 mal_Buffer_get_3(MalType_Buffer value) {
    return value.field_3;
}

static inline MalType_Slice mal_Slice_make(MalType_Allocation value_0, MalType_Ptr value_1, MalType_UInt64 value_2) {
    return (MalType_Slice){ .field_0 = value_0, .field_1 = value_1, .field_2 = value_2 };
}

static inline MalType_Allocation mal_Slice_get_0(MalType_Slice value) {
    return value.field_0;
}

static inline MalType_Ptr mal_Slice_get_1(MalType_Slice value) {
    return value.field_1;
}

static inline MalType_UInt64 mal_Slice_get_2(MalType_Slice value) {
    return value.field_2;
}

#define MAL_BufferResult_TAG_0 UINT32_C(0)
#define MAL_BufferResult_TAG_1 UINT32_C(1)
static inline uint32_t mal_BufferResult_tag(MalType_BufferResult value) {
    return value.tag;
}

static inline MalType_Bool mal_BufferResult_is_0(MalType_BufferResult value) {
    return value.tag == MAL_BufferResult_TAG_0;
}

static inline MalType_BufferResult mal_BufferResult_make_0(MalType_Allocation value_0, MalType_Ptr value_1, MalType_UInt64 value_2, MalType_UInt64 value_3) {
    return (MalType_BufferResult){ .tag = MAL_BufferResult_TAG_0, .payload.variant_0 = (MalRepr_Product_1){ .field_0 = value_0, .field_1 = value_1, .field_2 = value_2, .field_3 = value_3 } };
}

static inline MalType_Allocation mal_BufferResult_expect_0_0(MalContext *context, MalType_BufferResult value) {
    if (!mal_BufferResult_is_0(value)) {
        mal_trap(context, "expected BufferResult variant 0");
    }
    return value.payload.variant_0.field_0;
}

static inline MalType_Ptr mal_BufferResult_expect_0_1(MalContext *context, MalType_BufferResult value) {
    if (!mal_BufferResult_is_0(value)) {
        mal_trap(context, "expected BufferResult variant 0");
    }
    return value.payload.variant_0.field_1;
}

static inline MalType_UInt64 mal_BufferResult_expect_0_2(MalContext *context, MalType_BufferResult value) {
    if (!mal_BufferResult_is_0(value)) {
        mal_trap(context, "expected BufferResult variant 0");
    }
    return value.payload.variant_0.field_2;
}

static inline MalType_UInt64 mal_BufferResult_expect_0_3(MalContext *context, MalType_BufferResult value) {
    if (!mal_BufferResult_is_0(value)) {
        mal_trap(context, "expected BufferResult variant 0");
    }
    return value.payload.variant_0.field_3;
}

static inline MalType_Bool mal_BufferResult_is_1(MalType_BufferResult value) {
    return value.tag == MAL_BufferResult_TAG_1;
}

static inline MalType_BufferResult mal_BufferResult_make_1(MalType_UInt32 value) {
    return (MalType_BufferResult){ .tag = MAL_BufferResult_TAG_1, .payload.variant_1 = value };
}

static inline MalType_UInt32 mal_BufferResult_expect_1(MalContext *context, MalType_BufferResult value) {
    if (!mal_BufferResult_is_1(value)) {
        mal_trap(context, "expected BufferResult variant 1");
    }
    return value.payload.variant_1;
}

/* External operations */

MalType_BufferResult mal_ext_allocateBuffer(MalContext *context, MalType_UInt64 argument_0, MalType_UInt64 argument_1);
MalType_BufferResult mal_ext_resizeBuffer(MalContext *context, MalType_Buffer argument_0, MalType_UInt64 argument_1);
void mal_ext_releaseBuffer(MalContext *context, MalType_Allocation value);
MalType_Bool mal_ext_isCurrentBuffer(MalContext *context, MalType_Allocation argument_0, MalType_Ptr argument_1, MalType_UInt64 argument_2, MalType_UInt64 argument_3);
MalType_Bool mal_ext_isCurrentSlice(MalContext *context, MalType_Allocation argument_0, MalType_Ptr argument_1, MalType_UInt64 argument_2);
void mal_ext_writeSlice(MalContext *context, MalType_Allocation argument_0, MalType_Ptr argument_1, MalType_UInt64 argument_2);
void mal_ext_writeSliceDescriptor(MalContext *context, MalType_Allocation argument_0, MalType_Ptr argument_1);
void mal_ext_writeSymbol(MalContext *context, MalType_Symbol value);

/* External definition helpers */

#define MAL_HAS_EXTERN_allocateBuffer 1
#define MAL_DEFINE_allocateBuffer(context, argument_0, argument_1) \
MalType_BufferResult mal_ext_allocateBuffer( \
    MalContext *context MAL_DETAIL_MAYBE_UNUSED, \
    MalType_UInt64 argument_0, \
    MalType_UInt64 argument_1 \
)

#define MAL_HAS_EXTERN_resizeBuffer 1
#define MAL_DEFINE_resizeBuffer(context, argument_0, argument_1) \
MalType_BufferResult mal_ext_resizeBuffer( \
    MalContext *context MAL_DETAIL_MAYBE_UNUSED, \
    MalType_Buffer argument_0, \
    MalType_UInt64 argument_1 \
)

#define MAL_HAS_EXTERN_releaseBuffer 1
#define MAL_DEFINE_releaseBuffer(context, value) \
void mal_ext_releaseBuffer( \
    MalContext *context MAL_DETAIL_MAYBE_UNUSED, \
    MalType_Allocation value \
)

#define MAL_HAS_EXTERN_isCurrentBuffer 1
#define MAL_DEFINE_isCurrentBuffer(context, argument_0, argument_1, argument_2, argument_3) \
MalType_Bool mal_ext_isCurrentBuffer( \
    MalContext *context MAL_DETAIL_MAYBE_UNUSED, \
    MalType_Allocation argument_0, \
    MalType_Ptr argument_1, \
    MalType_UInt64 argument_2, \
    MalType_UInt64 argument_3 \
)

#define MAL_HAS_EXTERN_isCurrentSlice 1
#define MAL_DEFINE_isCurrentSlice(context, argument_0, argument_1, argument_2) \
MalType_Bool mal_ext_isCurrentSlice( \
    MalContext *context MAL_DETAIL_MAYBE_UNUSED, \
    MalType_Allocation argument_0, \
    MalType_Ptr argument_1, \
    MalType_UInt64 argument_2 \
)

#define MAL_HAS_EXTERN_writeSlice 1
#define MAL_DEFINE_writeSlice(context, argument_0, argument_1, argument_2) \
void mal_ext_writeSlice( \
    MalContext *context MAL_DETAIL_MAYBE_UNUSED, \
    MalType_Allocation argument_0, \
    MalType_Ptr argument_1, \
    MalType_UInt64 argument_2 \
)

#define MAL_HAS_EXTERN_writeSliceDescriptor 1
#define MAL_DEFINE_writeSliceDescriptor(context, argument_0, argument_1) \
void mal_ext_writeSliceDescriptor( \
    MalContext *context MAL_DETAIL_MAYBE_UNUSED, \
    MalType_Allocation argument_0, \
    MalType_Ptr argument_1 \
)

#define MAL_HAS_EXTERN_writeSymbol 1
#define MAL_DEFINE_writeSymbol(context, value) \
void mal_ext_writeSymbol( \
    MalContext *context MAL_DETAIL_MAYBE_UNUSED, \
    MalType_Symbol value \
)

#endif
