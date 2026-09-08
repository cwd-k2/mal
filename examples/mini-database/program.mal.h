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

typedef struct { uintptr_t bits; } MalType_Allocator;
typedef struct { uintptr_t bits; } MalType_File;

typedef struct MalRepr_Product_0 MalRepr_Product_0;
typedef struct MalRepr_Product_1 MalRepr_Product_1;
typedef struct MalRepr_Product_2 MalRepr_Product_2;
typedef struct MalRepr_Product_3 MalRepr_Product_3;

struct MalRepr_Product_0 {
    MalType_Allocator field_0;
    MalType_UInt64 field_1;
};

struct MalRepr_Product_1 {
    MalType_Ptr field_0;
    MalType_UInt64 field_1;
    MalType_UInt64 field_2;
};

struct MalRepr_Product_2 {
    MalType_Ptr field_0;
    MalType_UInt64 field_1;
};

struct MalRepr_Product_3 {
    MalType_File field_0;
    MalRepr_Product_2 field_1;
};

typedef MalRepr_Product_1 MalType_Buffer;
typedef MalRepr_Product_2 MalType_Bytes;
typedef MalRepr_Product_2 MalType_Region;

/* Type helpers */

static inline MalType_Allocator mal_Allocator_from_bits(uintptr_t bits) {
    return (MalType_Allocator){ .bits = bits };
}

static inline uintptr_t mal_Allocator_bits(MalType_Allocator value) {
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

static inline MalType_Bytes mal_Bytes_make(MalType_Ptr value_0, MalType_UInt64 value_1) {
    return (MalType_Bytes){ .field_0 = value_0, .field_1 = value_1 };
}

static inline MalType_Ptr mal_Bytes_get_0(MalType_Bytes value) {
    return value.field_0;
}

static inline MalType_UInt64 mal_Bytes_get_1(MalType_Bytes value) {
    return value.field_1;
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

/* External operations */

MalType_Allocator mal_ext_createAllocator(MalContext *context);
MalType_Buffer mal_ext_allocateBuffer(MalContext *context, MalType_Allocator argument_0, MalType_UInt64 argument_1);
void mal_ext_destroyAllocator(MalContext *context, MalType_Allocator value);
MalType_File mal_ext_openReadWriteCreate(MalContext *context, MalType_Symbol value);
MalType_File mal_ext_standardInput(MalContext *context);
MalType_UInt64 mal_ext_readFile(MalContext *context, MalType_File argument_0, MalType_Region argument_1);
MalType_UInt64 mal_ext_writeFile(MalContext *context, MalType_File argument_0, MalType_Bytes argument_1);
void mal_ext_rewindFile(MalContext *context, MalType_File value);
void mal_ext_flushFile(MalContext *context, MalType_File value);
void mal_ext_closeFile(MalContext *context, MalType_File value);
void mal_ext_writeSymbol(MalContext *context, MalType_Symbol value);
void mal_ext_writeBytes(MalContext *context, MalType_Ptr argument_0, MalType_UInt64 argument_1);
void mal_ext_fail(MalContext *context, MalType_Symbol value);

/* External definition helpers */

#define MAL_HAS_EXTERN_createAllocator 1
#define MAL_DEFINE_createAllocator(context) \
MalType_Allocator mal_ext_createAllocator( \
    MalContext *context MAL_DETAIL_MAYBE_UNUSED \
)

#define MAL_HAS_EXTERN_allocateBuffer 1
#define MAL_DEFINE_allocateBuffer(context, argument_0, argument_1) \
MalType_Buffer mal_ext_allocateBuffer( \
    MalContext *context MAL_DETAIL_MAYBE_UNUSED, \
    MalType_Allocator argument_0, \
    MalType_UInt64 argument_1 \
)

#define MAL_HAS_EXTERN_destroyAllocator 1
#define MAL_DEFINE_destroyAllocator(context, value) \
void mal_ext_destroyAllocator( \
    MalContext *context MAL_DETAIL_MAYBE_UNUSED, \
    MalType_Allocator value \
)

#define MAL_HAS_EXTERN_openReadWriteCreate 1
#define MAL_DEFINE_openReadWriteCreate(context, value) \
MalType_File mal_ext_openReadWriteCreate( \
    MalContext *context MAL_DETAIL_MAYBE_UNUSED, \
    MalType_Symbol value \
)

#define MAL_HAS_EXTERN_standardInput 1
#define MAL_DEFINE_standardInput(context) \
MalType_File mal_ext_standardInput( \
    MalContext *context MAL_DETAIL_MAYBE_UNUSED \
)

#define MAL_HAS_EXTERN_readFile 1
#define MAL_DEFINE_readFile(context, argument_0, argument_1) \
MalType_UInt64 mal_ext_readFile( \
    MalContext *context MAL_DETAIL_MAYBE_UNUSED, \
    MalType_File argument_0, \
    MalType_Region argument_1 \
)

#define MAL_HAS_EXTERN_writeFile 1
#define MAL_DEFINE_writeFile(context, argument_0, argument_1) \
MalType_UInt64 mal_ext_writeFile( \
    MalContext *context MAL_DETAIL_MAYBE_UNUSED, \
    MalType_File argument_0, \
    MalType_Bytes argument_1 \
)

#define MAL_HAS_EXTERN_rewindFile 1
#define MAL_DEFINE_rewindFile(context, value) \
void mal_ext_rewindFile( \
    MalContext *context MAL_DETAIL_MAYBE_UNUSED, \
    MalType_File value \
)

#define MAL_HAS_EXTERN_flushFile 1
#define MAL_DEFINE_flushFile(context, value) \
void mal_ext_flushFile( \
    MalContext *context MAL_DETAIL_MAYBE_UNUSED, \
    MalType_File value \
)

#define MAL_HAS_EXTERN_closeFile 1
#define MAL_DEFINE_closeFile(context, value) \
void mal_ext_closeFile( \
    MalContext *context MAL_DETAIL_MAYBE_UNUSED, \
    MalType_File value \
)

#define MAL_HAS_EXTERN_writeSymbol 1
#define MAL_DEFINE_writeSymbol(context, value) \
void mal_ext_writeSymbol( \
    MalContext *context MAL_DETAIL_MAYBE_UNUSED, \
    MalType_Symbol value \
)

#define MAL_HAS_EXTERN_writeBytes 1
#define MAL_DEFINE_writeBytes(context, argument_0, argument_1) \
void mal_ext_writeBytes( \
    MalContext *context MAL_DETAIL_MAYBE_UNUSED, \
    MalType_Ptr argument_0, \
    MalType_UInt64 argument_1 \
)

#define MAL_HAS_EXTERN_fail 1
#define MAL_DEFINE_fail(context, value) \
void mal_ext_fail( \
    MalContext *context MAL_DETAIL_MAYBE_UNUSED, \
    MalType_Symbol value \
)

#endif
