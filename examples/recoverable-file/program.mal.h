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

/* Type helpers */

static inline MalType_Allocation mal_Allocation_from_bits(uintptr_t bits) {
    return (MalType_Allocation){ .bits = bits };
}

static inline uintptr_t mal_Allocation_bits(MalType_Allocation value) {
    return value.bits;
}

static inline MalType_File mal_File_from_bits(uintptr_t bits) {
    return (MalType_File){ .bits = bits };
}

static inline uintptr_t mal_File_bits(MalType_File value) {
    return value.bits;
}

static inline MalType_Buffer mal_Buffer_make(MalType_Ptr value_0, MalType_UInt64 value_1, MalType_UInt64 value_2) {
    return (MalType_Buffer){ .field_0 = value_0, .field_1 = value_1, .field_2 = value_2 };
}

static inline MalType_Ptr mal_Buffer_get_0(MalType_Buffer value) {
    return value.field_0;
}

static inline MalType_UInt64 mal_Buffer_get_1(MalType_Buffer value) {
    return value.field_1;
}

static inline MalType_UInt64 mal_Buffer_get_2(MalType_Buffer value) {
    return value.field_2;
}

static inline MalType_Region mal_Region_make(MalType_Ptr value_0, MalType_UInt64 value_1) {
    return (MalType_Region){ .field_0 = value_0, .field_1 = value_1 };
}

static inline MalType_Ptr mal_Region_get_0(MalType_Region value) {
    return value.field_0;
}

static inline MalType_UInt64 mal_Region_get_1(MalType_Region value) {
    return value.field_1;
}

static inline MalType_Bytes mal_Bytes_make(MalType_Ptr value_0, MalType_UInt64 value_1) {
    return (MalType_Bytes){ .field_0 = value_0, .field_1 = value_1 };
}

static inline MalType_Ptr mal_Bytes_get_0(MalType_Bytes value) {
    return value.field_0;
}

static inline MalType_UInt64 mal_Bytes_get_1(MalType_Bytes value) {
    return value.field_1;
}

static inline MalType_OwnedBuffer mal_OwnedBuffer_make(MalType_Allocation value_0, MalRepr_Product_0 value_1) {
    return (MalType_OwnedBuffer){ .field_0 = value_0, .field_1 = value_1 };
}

static inline MalType_Allocation mal_OwnedBuffer_get_0(MalType_OwnedBuffer value) {
    return value.field_0;
}

static inline MalRepr_Product_0 mal_OwnedBuffer_get_1(MalType_OwnedBuffer value) {
    return value.field_1;
}

#define MAL_OpenResult_TAG_0 UINT32_C(0)
#define MAL_OpenResult_TAG_1 UINT32_C(1)
static inline uint32_t mal_OpenResult_tag(MalType_OpenResult value) {
    return value.tag;
}

static inline MalType_Bool mal_OpenResult_is_0(MalType_OpenResult value) {
    return value.tag == MAL_OpenResult_TAG_0;
}

static inline MalType_OpenResult mal_OpenResult_make_0(MalType_File value) {
    return (MalType_OpenResult){ .tag = MAL_OpenResult_TAG_0, .payload.variant_0 = value };
}

static inline MalType_File mal_OpenResult_expect_0(MalContext *context, MalType_OpenResult value) {
    if (!mal_OpenResult_is_0(value)) {
        mal_trap(context, "expected OpenResult variant 0");
    }
    return value.payload.variant_0;
}

static inline MalType_Bool mal_OpenResult_is_1(MalType_OpenResult value) {
    return value.tag == MAL_OpenResult_TAG_1;
}

static inline MalType_OpenResult mal_OpenResult_make_1(MalType_UInt32 value) {
    return (MalType_OpenResult){ .tag = MAL_OpenResult_TAG_1, .payload.variant_1 = value };
}

static inline MalType_UInt32 mal_OpenResult_expect_1(MalContext *context, MalType_OpenResult value) {
    if (!mal_OpenResult_is_1(value)) {
        mal_trap(context, "expected OpenResult variant 1");
    }
    return value.payload.variant_1;
}

#define MAL_ReadResult_TAG_0 UINT32_C(0)
#define MAL_ReadResult_TAG_1 UINT32_C(1)
static inline uint32_t mal_ReadResult_tag(MalType_ReadResult value) {
    return value.tag;
}

static inline MalType_Bool mal_ReadResult_is_0(MalType_ReadResult value) {
    return value.tag == MAL_ReadResult_TAG_0;
}

static inline MalType_ReadResult mal_ReadResult_make_0(MalType_UInt64 value) {
    return (MalType_ReadResult){ .tag = MAL_ReadResult_TAG_0, .payload.variant_0 = value };
}

static inline MalType_UInt64 mal_ReadResult_expect_0(MalContext *context, MalType_ReadResult value) {
    if (!mal_ReadResult_is_0(value)) {
        mal_trap(context, "expected ReadResult variant 0");
    }
    return value.payload.variant_0;
}

static inline MalType_Bool mal_ReadResult_is_1(MalType_ReadResult value) {
    return value.tag == MAL_ReadResult_TAG_1;
}

static inline MalType_ReadResult mal_ReadResult_make_1(MalType_UInt32 value) {
    return (MalType_ReadResult){ .tag = MAL_ReadResult_TAG_1, .payload.variant_1 = value };
}

static inline MalType_UInt32 mal_ReadResult_expect_1(MalContext *context, MalType_ReadResult value) {
    if (!mal_ReadResult_is_1(value)) {
        mal_trap(context, "expected ReadResult variant 1");
    }
    return value.payload.variant_1;
}

#define MAL_CloseResult_TAG_0 UINT32_C(0)
#define MAL_CloseResult_TAG_1 UINT32_C(1)
static inline uint32_t mal_CloseResult_tag(MalType_CloseResult value) {
    return value.tag;
}

static inline MalType_Bool mal_CloseResult_is_0(MalType_CloseResult value) {
    return value.tag == MAL_CloseResult_TAG_0;
}

static inline MalType_CloseResult mal_CloseResult_make_0(void) {
    return (MalType_CloseResult){ .tag = MAL_CloseResult_TAG_0, .payload.variant_0 = (MalType_Unit){ .unused = UINT8_C(0) } };
}

static inline MalType_Bool mal_CloseResult_is_1(MalType_CloseResult value) {
    return value.tag == MAL_CloseResult_TAG_1;
}

static inline MalType_CloseResult mal_CloseResult_make_1(MalType_UInt32 value) {
    return (MalType_CloseResult){ .tag = MAL_CloseResult_TAG_1, .payload.variant_1 = value };
}

static inline MalType_UInt32 mal_CloseResult_expect_1(MalContext *context, MalType_CloseResult value) {
    if (!mal_CloseResult_is_1(value)) {
        mal_trap(context, "expected CloseResult variant 1");
    }
    return value.payload.variant_1;
}

#define MAL_Status_TAG_0 UINT32_C(0)
#define MAL_Status_TAG_1 UINT32_C(1)
static inline uint32_t mal_Status_tag(MalType_Status value) {
    return value.tag;
}

static inline MalType_Bool mal_Status_is_0(MalType_Status value) {
    return value.tag == MAL_Status_TAG_0;
}

static inline MalType_Status mal_Status_make_0(void) {
    return (MalType_Status){ .tag = MAL_Status_TAG_0, .payload.variant_0 = (MalType_Unit){ .unused = UINT8_C(0) } };
}

static inline MalType_Bool mal_Status_is_1(MalType_Status value) {
    return value.tag == MAL_Status_TAG_1;
}

static inline MalType_Status mal_Status_make_1(MalType_UInt32 value) {
    return (MalType_Status){ .tag = MAL_Status_TAG_1, .payload.variant_1 = value };
}

static inline MalType_UInt32 mal_Status_expect_1(MalContext *context, MalType_Status value) {
    if (!mal_Status_is_1(value)) {
        mal_trap(context, "expected Status variant 1");
    }
    return value.payload.variant_1;
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
#define MAL_DEFINE_allocateBuffer(context, value) \
MalType_OwnedBuffer mal_ext_allocateBuffer( \
    MalContext *context MAL_DETAIL_MAYBE_UNUSED, \
    MalType_UInt64 value \
)

#define MAL_HAS_EXTERN_releaseBuffer 1
#define MAL_DEFINE_releaseBuffer(context, value) \
void mal_ext_releaseBuffer( \
    MalContext *context MAL_DETAIL_MAYBE_UNUSED, \
    MalType_Allocation value \
)

#define MAL_HAS_EXTERN_openReadOnly 1
#define MAL_DEFINE_openReadOnly(context, value) \
MalType_OpenResult mal_ext_openReadOnly( \
    MalContext *context MAL_DETAIL_MAYBE_UNUSED, \
    MalType_Symbol value \
)

#define MAL_HAS_EXTERN_readFile 1
#define MAL_DEFINE_readFile(context, argument_0, argument_1) \
MalType_ReadResult mal_ext_readFile( \
    MalContext *context MAL_DETAIL_MAYBE_UNUSED, \
    MalType_File argument_0, \
    MalType_Region argument_1 \
)

#define MAL_HAS_EXTERN_closeFile 1
#define MAL_DEFINE_closeFile(context, value) \
MalType_CloseResult mal_ext_closeFile( \
    MalContext *context MAL_DETAIL_MAYBE_UNUSED, \
    MalType_File value \
)

#define MAL_HAS_EXTERN_writeBytes 1
#define MAL_DEFINE_writeBytes(context, argument_0, argument_1) \
void mal_ext_writeBytes( \
    MalContext *context MAL_DETAIL_MAYBE_UNUSED, \
    MalType_Ptr argument_0, \
    MalType_UInt64 argument_1 \
)

#define MAL_HAS_EXTERN_writeSymbol 1
#define MAL_DEFINE_writeSymbol(context, value) \
void mal_ext_writeSymbol( \
    MalContext *context MAL_DETAIL_MAYBE_UNUSED, \
    MalType_Symbol value \
)

#define MAL_HAS_EXTERN_writeError 1
#define MAL_DEFINE_writeError(context, value) \
void mal_ext_writeError( \
    MalContext *context MAL_DETAIL_MAYBE_UNUSED, \
    MalType_IoError value \
)

#endif
