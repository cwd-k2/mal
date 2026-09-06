#ifndef MAL_PROGRAM_MAL_H
#define MAL_PROGRAM_MAL_H

#include <stdint.h>

#define MAL_C_ABI_VERSION 0x000500u
#define MAL_TYPE(name) MalType_##name
#define MAL_OPERATION(type, operation) mal_##type##_##operation
#define MAL_TAG(type, variant) MAL_##type##_TAG_##variant
#define MAL_EXTERN(name) mal_ext_##name

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
typedef struct { const uint8_t *data; uint64_t length; } MalType_Symbol;
typedef struct { uint8_t *address; } MalType_Ptr;

#define MAL_FALSE (MalType_Bool)UINT8_C(0)
#define MAL_TRUE (MalType_Bool)UINT8_C(1)

_Noreturn void mal_trap(MalContext *context, const char *message);
MalType_Symbol mal_Symbol_copy_from_bytes(MalContext *context, const uint8_t *data, uint64_t length);

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

typedef struct { uintptr_t bits; } MalType_File;

typedef struct MalRepr_Product_0 MalRepr_Product_0;
typedef struct MalRepr_Product_1 MalRepr_Product_1;

struct MalRepr_Product_0 {
    MalType_File field_0;
    MalType_Ptr field_1;
    MalType_UInt64 field_2;
};

struct MalRepr_Product_1 {
    MalType_Ptr field_0;
    MalType_UInt64 field_1;
};

/* Type helpers */

static inline MalType_File mal_File_from_bits(uintptr_t bits) {
    return (MalType_File){ .bits = bits };
}

static inline uintptr_t mal_File_bits(MalType_File value) {
    return value.bits;
}

/* External operations */

MalType_Ptr mal_ext_allocate(MalContext *context, MalType_UInt64 value);
void mal_ext_release(MalContext *context, MalType_Ptr value);
MalType_File mal_ext_openReadWriteCreate(MalContext *context, MalType_Symbol value);
MalType_File mal_ext_standardInput(MalContext *context);
MalType_UInt64 mal_ext_readFile(MalContext *context, MalType_File argument_0, MalType_Ptr argument_1, MalType_UInt64 argument_2);
MalType_UInt64 mal_ext_writeFile(MalContext *context, MalType_File argument_0, MalType_Ptr argument_1, MalType_UInt64 argument_2);
void mal_ext_rewindFile(MalContext *context, MalType_File value);
void mal_ext_flushFile(MalContext *context, MalType_File value);
void mal_ext_closeFile(MalContext *context, MalType_File value);
void mal_ext_writeSymbol(MalContext *context, MalType_Symbol value);
void mal_ext_writeMemory(MalContext *context, MalType_Ptr argument_0, MalType_UInt64 argument_1);
void mal_ext_fail(MalContext *context, MalType_Symbol value);

/* External definition helpers */

#define MAL_HAS_EXTERN_allocate 1
#define MAL_DEFINE_allocate(context, value) \
MalType_Ptr mal_ext_allocate( \
    MalContext *context MAL_DETAIL_MAYBE_UNUSED, \
    MalType_UInt64 value \
)

#define MAL_HAS_EXTERN_release 1
#define MAL_DEFINE_release(context, value) \
void mal_ext_release( \
    MalContext *context MAL_DETAIL_MAYBE_UNUSED, \
    MalType_Ptr value \
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
#define MAL_DEFINE_readFile(context, argument_0, argument_1, argument_2) \
MalType_UInt64 mal_ext_readFile( \
    MalContext *context MAL_DETAIL_MAYBE_UNUSED, \
    MalType_File argument_0, \
    MalType_Ptr argument_1, \
    MalType_UInt64 argument_2 \
)

#define MAL_HAS_EXTERN_writeFile 1
#define MAL_DEFINE_writeFile(context, argument_0, argument_1, argument_2) \
MalType_UInt64 mal_ext_writeFile( \
    MalContext *context MAL_DETAIL_MAYBE_UNUSED, \
    MalType_File argument_0, \
    MalType_Ptr argument_1, \
    MalType_UInt64 argument_2 \
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

#define MAL_HAS_EXTERN_writeMemory 1
#define MAL_DEFINE_writeMemory(context, argument_0, argument_1) \
void mal_ext_writeMemory( \
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
