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

typedef struct { uintptr_t bits; } MalType_Allocator;
typedef struct { uintptr_t bits; } MalType_File;

typedef struct MalRepr_Product_0 MalRepr_Product_0;
typedef struct MalRepr_Product_1 MalRepr_Product_1;
typedef struct MalRepr_Product_2 MalRepr_Product_2;

struct MalRepr_Product_0 {
    MalType_Allocator field_0;
    MalType_USize field_1;
};

struct MalRepr_Product_1 {
    MalType_Address field_0;
    MalType_USize field_1;
};

struct MalRepr_Product_2 {
    MalType_File field_0;
    MalType_Address field_1;
    MalType_USize field_2;
    MalType_USize field_3;
};

typedef MalRepr_Product_1 MalType_AllocatedBytes;
typedef MalRepr_Product_1 MalType_ReadableBytes;
typedef MalRepr_Product_1 MalType_OutputBuffer;

typedef struct { uintptr_t mal_detail_bits; } mal_Allocator_t;
typedef struct { uintptr_t mal_detail_bits; } mal_File_t;
typedef struct mal_detail_repr_product_0 mal_repr_product_0_t;
typedef struct mal_detail_repr_product_1 mal_repr_product_1_t;
typedef struct mal_detail_repr_product_2 mal_repr_product_2_t;
typedef mal_repr_product_1_t mal_AllocatedBytes_t;
typedef mal_repr_product_1_t mal_ReadableBytes_t;
typedef mal_repr_product_1_t mal_OutputBuffer_t;

struct mal_detail_repr_product_0 {
    mal_Allocator_t field_0;
    mal_USize_t field_1;
};

struct mal_detail_repr_product_1 {
    mal_Address_t field_0;
    mal_USize_t field_1;
};

struct mal_detail_repr_product_2 {
    mal_File_t field_0;
    mal_Address_t field_1;
    mal_USize_t field_2;
    mal_USize_t field_3;
};

/* Type helpers */

static inline MalRepr_Product_0 mal_repr_product_0_return(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, mal_repr_product_0_t value) {
    return (MalRepr_Product_0){ .field_0 = (MalType_Allocator){ .bits = value.field_0.mal_detail_bits }, .field_1 = value.field_1 };
}

static inline MalRepr_Product_1 mal_repr_product_1_return(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, mal_repr_product_1_t value) {
    return (MalRepr_Product_1){ .field_0 = mal_Address_return(call, value.field_0), .field_1 = value.field_1 };
}

static inline MalRepr_Product_2 mal_repr_product_2_return(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, mal_repr_product_2_t value) {
    return (MalRepr_Product_2){ .field_0 = (MalType_File){ .bits = value.field_0.mal_detail_bits }, .field_1 = mal_Address_return(call, value.field_1), .field_2 = value.field_2, .field_3 = value.field_3 };
}

static inline mal_Allocator_t mal_Allocator_from_bits(uintptr_t bits) {
    return (mal_Allocator_t){ .mal_detail_bits = bits };
}

static inline uintptr_t mal_Allocator_to_bits(mal_Allocator_t value) {
    return value.mal_detail_bits;
}

static inline MalType_Allocator mal_Allocator_return(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, mal_Allocator_t value) {
    return (MalType_Allocator){ .bits = value.mal_detail_bits };
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

static inline MalType_AllocatedBytes mal_AllocatedBytes_return(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, mal_AllocatedBytes_t value) {
    return (MalRepr_Product_1){ .field_0 = mal_Address_return(call, value.field_0), .field_1 = value.field_1 };
}

static inline MalType_ReadableBytes mal_ReadableBytes_return(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, mal_ReadableBytes_t value) {
    return (MalRepr_Product_1){ .field_0 = mal_Address_return(call, value.field_0), .field_1 = value.field_1 };
}

static inline MalType_OutputBuffer mal_OutputBuffer_return(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, mal_OutputBuffer_t value) {
    return (MalRepr_Product_1){ .field_0 = mal_Address_return(call, value.field_0), .field_1 = value.field_1 };
}

/* External operations */

MalType_Allocator mal_ext_createAllocator(MalContext *context);
MalType_AllocatedBytes mal_ext_allocateBuffer(MalContext *context, MalType_Allocator argument_0, MalType_USize argument_1);
void mal_ext_destroyAllocator(MalContext *context, MalType_Allocator value);
MalType_File mal_ext_openReadWriteCreate(MalContext *context, MalType_Address argument_0, MalType_USize argument_1);
MalType_File mal_ext_standardInput(MalContext *context);
MalType_USize mal_ext_readFile(MalContext *context, MalType_File argument_0, MalType_Address argument_1, MalType_USize argument_2, MalType_USize argument_3);
MalType_USize mal_ext_writeFile(MalContext *context, MalType_File argument_0, MalType_Address argument_1, MalType_USize argument_2, MalType_USize argument_3);
void mal_ext_rewindFile(MalContext *context, MalType_File value);
void mal_ext_flushFile(MalContext *context, MalType_File value);
void mal_ext_closeFile(MalContext *context, MalType_File value);
MalType_OutputBuffer mal_ext_outputBuffer(MalContext *context);
void mal_ext_writeStdout(MalContext *context, MalType_Address argument_0, MalType_USize argument_1);
void mal_ext_writeStderr(MalContext *context, MalType_Address argument_0, MalType_USize argument_1);
void mal_ext_failNow(MalContext *context);
MalType_USize mal_ext_argumentLength(MalContext *context, MalType_Address value);

/* External definition helpers */

#define MAL_HAS_EXTERN_createAllocator 1
#define MAL_DEFINE_createAllocator(call) \
static MalType_Allocator mal_detail_createAllocator(mal_call_t *call); \
MalType_Allocator mal_ext_createAllocator(MalContext *context MAL_DETAIL_MAYBE_UNUSED) { \
    mal_call_t call = (mal_call_t){ .mal_detail_context = context }; \
    return mal_detail_createAllocator(&call); \
} \
static MalType_Allocator mal_detail_createAllocator( \
    mal_call_t *call \
)

#define MAL_HAS_EXTERN_allocateBuffer 1
#define MAL_DEFINE_allocateBuffer(call, value) \
static MalType_AllocatedBytes mal_detail_allocateBuffer(mal_call_t *call, mal_repr_product_0_t value); \
MalType_AllocatedBytes mal_ext_allocateBuffer(MalContext *context MAL_DETAIL_MAYBE_UNUSED, MalType_Allocator argument_0, MalType_USize argument_1) { \
    mal_call_t call = (mal_call_t){ .mal_detail_context = context }; \
    return mal_detail_allocateBuffer(&call, (mal_repr_product_0_t){ .field_0 = (mal_Allocator_t){ .mal_detail_bits = ((MalRepr_Product_0){ .field_0 = argument_0, .field_1 = argument_1 }).field_0.bits }, .field_1 = ((MalRepr_Product_0){ .field_0 = argument_0, .field_1 = argument_1 }).field_1 }); \
} \
static MalType_AllocatedBytes mal_detail_allocateBuffer( \
    mal_call_t *call, \
    mal_repr_product_0_t value \
)

#define MAL_HAS_EXTERN_destroyAllocator 1
#define MAL_DEFINE_destroyAllocator(call, value) \
static MalType_Unit mal_detail_destroyAllocator(mal_call_t *call, mal_Allocator_t value); \
void mal_ext_destroyAllocator(MalContext *context MAL_DETAIL_MAYBE_UNUSED, MalType_Allocator value) { \
    mal_call_t call = (mal_call_t){ .mal_detail_context = context }; \
    mal_detail_destroyAllocator(&call, (mal_Allocator_t){ .mal_detail_bits = value.bits }); \
} \
static MalType_Unit mal_detail_destroyAllocator( \
    mal_call_t *call, \
    mal_Allocator_t value \
)

#define MAL_HAS_EXTERN_openReadWriteCreate 1
#define MAL_DEFINE_openReadWriteCreate(call, value) \
static MalType_File mal_detail_openReadWriteCreate(mal_call_t *call, mal_ReadableBytes_t value); \
MalType_File mal_ext_openReadWriteCreate(MalContext *context MAL_DETAIL_MAYBE_UNUSED, MalType_Address argument_0, MalType_USize argument_1) { \
    mal_call_t call = (mal_call_t){ .mal_detail_context = context }; \
    return mal_detail_openReadWriteCreate(&call, (mal_ReadableBytes_t){ .field_0 = ((MalRepr_Product_1){ .field_0 = argument_0, .field_1 = argument_1 }).field_0, .field_1 = ((MalRepr_Product_1){ .field_0 = argument_0, .field_1 = argument_1 }).field_1 }); \
} \
static MalType_File mal_detail_openReadWriteCreate( \
    mal_call_t *call, \
    mal_ReadableBytes_t value \
)

#define MAL_HAS_EXTERN_standardInput 1
#define MAL_DEFINE_standardInput(call) \
static MalType_File mal_detail_standardInput(mal_call_t *call); \
MalType_File mal_ext_standardInput(MalContext *context MAL_DETAIL_MAYBE_UNUSED) { \
    mal_call_t call = (mal_call_t){ .mal_detail_context = context }; \
    return mal_detail_standardInput(&call); \
} \
static MalType_File mal_detail_standardInput( \
    mal_call_t *call \
)

#define MAL_HAS_EXTERN_readFile 1
#define MAL_DEFINE_readFile(call, value) \
static MalType_USize mal_detail_readFile(mal_call_t *call, mal_repr_product_2_t value); \
MalType_USize mal_ext_readFile(MalContext *context MAL_DETAIL_MAYBE_UNUSED, MalType_File argument_0, MalType_Address argument_1, MalType_USize argument_2, MalType_USize argument_3) { \
    mal_call_t call = (mal_call_t){ .mal_detail_context = context }; \
    return mal_detail_readFile(&call, (mal_repr_product_2_t){ .field_0 = (mal_File_t){ .mal_detail_bits = ((MalRepr_Product_2){ .field_0 = argument_0, .field_1 = argument_1, .field_2 = argument_2, .field_3 = argument_3 }).field_0.bits }, .field_1 = ((MalRepr_Product_2){ .field_0 = argument_0, .field_1 = argument_1, .field_2 = argument_2, .field_3 = argument_3 }).field_1, .field_2 = ((MalRepr_Product_2){ .field_0 = argument_0, .field_1 = argument_1, .field_2 = argument_2, .field_3 = argument_3 }).field_2, .field_3 = ((MalRepr_Product_2){ .field_0 = argument_0, .field_1 = argument_1, .field_2 = argument_2, .field_3 = argument_3 }).field_3 }); \
} \
static MalType_USize mal_detail_readFile( \
    mal_call_t *call, \
    mal_repr_product_2_t value \
)

#define MAL_HAS_EXTERN_writeFile 1
#define MAL_DEFINE_writeFile(call, value) \
static MalType_USize mal_detail_writeFile(mal_call_t *call, mal_repr_product_2_t value); \
MalType_USize mal_ext_writeFile(MalContext *context MAL_DETAIL_MAYBE_UNUSED, MalType_File argument_0, MalType_Address argument_1, MalType_USize argument_2, MalType_USize argument_3) { \
    mal_call_t call = (mal_call_t){ .mal_detail_context = context }; \
    return mal_detail_writeFile(&call, (mal_repr_product_2_t){ .field_0 = (mal_File_t){ .mal_detail_bits = ((MalRepr_Product_2){ .field_0 = argument_0, .field_1 = argument_1, .field_2 = argument_2, .field_3 = argument_3 }).field_0.bits }, .field_1 = ((MalRepr_Product_2){ .field_0 = argument_0, .field_1 = argument_1, .field_2 = argument_2, .field_3 = argument_3 }).field_1, .field_2 = ((MalRepr_Product_2){ .field_0 = argument_0, .field_1 = argument_1, .field_2 = argument_2, .field_3 = argument_3 }).field_2, .field_3 = ((MalRepr_Product_2){ .field_0 = argument_0, .field_1 = argument_1, .field_2 = argument_2, .field_3 = argument_3 }).field_3 }); \
} \
static MalType_USize mal_detail_writeFile( \
    mal_call_t *call, \
    mal_repr_product_2_t value \
)

#define MAL_HAS_EXTERN_rewindFile 1
#define MAL_DEFINE_rewindFile(call, value) \
static MalType_Unit mal_detail_rewindFile(mal_call_t *call, mal_File_t value); \
void mal_ext_rewindFile(MalContext *context MAL_DETAIL_MAYBE_UNUSED, MalType_File value) { \
    mal_call_t call = (mal_call_t){ .mal_detail_context = context }; \
    mal_detail_rewindFile(&call, (mal_File_t){ .mal_detail_bits = value.bits }); \
} \
static MalType_Unit mal_detail_rewindFile( \
    mal_call_t *call, \
    mal_File_t value \
)

#define MAL_HAS_EXTERN_flushFile 1
#define MAL_DEFINE_flushFile(call, value) \
static MalType_Unit mal_detail_flushFile(mal_call_t *call, mal_File_t value); \
void mal_ext_flushFile(MalContext *context MAL_DETAIL_MAYBE_UNUSED, MalType_File value) { \
    mal_call_t call = (mal_call_t){ .mal_detail_context = context }; \
    mal_detail_flushFile(&call, (mal_File_t){ .mal_detail_bits = value.bits }); \
} \
static MalType_Unit mal_detail_flushFile( \
    mal_call_t *call, \
    mal_File_t value \
)

#define MAL_HAS_EXTERN_closeFile 1
#define MAL_DEFINE_closeFile(call, value) \
static MalType_Unit mal_detail_closeFile(mal_call_t *call, mal_File_t value); \
void mal_ext_closeFile(MalContext *context MAL_DETAIL_MAYBE_UNUSED, MalType_File value) { \
    mal_call_t call = (mal_call_t){ .mal_detail_context = context }; \
    mal_detail_closeFile(&call, (mal_File_t){ .mal_detail_bits = value.bits }); \
} \
static MalType_Unit mal_detail_closeFile( \
    mal_call_t *call, \
    mal_File_t value \
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

#define MAL_HAS_EXTERN_writeStdout 1
#define MAL_DEFINE_writeStdout(call, value) \
static MalType_Unit mal_detail_writeStdout(mal_call_t *call, mal_ReadableBytes_t value); \
void mal_ext_writeStdout(MalContext *context MAL_DETAIL_MAYBE_UNUSED, MalType_Address argument_0, MalType_USize argument_1) { \
    mal_call_t call = (mal_call_t){ .mal_detail_context = context }; \
    mal_detail_writeStdout(&call, (mal_ReadableBytes_t){ .field_0 = ((MalRepr_Product_1){ .field_0 = argument_0, .field_1 = argument_1 }).field_0, .field_1 = ((MalRepr_Product_1){ .field_0 = argument_0, .field_1 = argument_1 }).field_1 }); \
} \
static MalType_Unit mal_detail_writeStdout( \
    mal_call_t *call, \
    mal_ReadableBytes_t value \
)

#define MAL_HAS_EXTERN_writeStderr 1
#define MAL_DEFINE_writeStderr(call, value) \
static MalType_Unit mal_detail_writeStderr(mal_call_t *call, mal_ReadableBytes_t value); \
void mal_ext_writeStderr(MalContext *context MAL_DETAIL_MAYBE_UNUSED, MalType_Address argument_0, MalType_USize argument_1) { \
    mal_call_t call = (mal_call_t){ .mal_detail_context = context }; \
    mal_detail_writeStderr(&call, (mal_ReadableBytes_t){ .field_0 = ((MalRepr_Product_1){ .field_0 = argument_0, .field_1 = argument_1 }).field_0, .field_1 = ((MalRepr_Product_1){ .field_0 = argument_0, .field_1 = argument_1 }).field_1 }); \
} \
static MalType_Unit mal_detail_writeStderr( \
    mal_call_t *call, \
    mal_ReadableBytes_t value \
)

#define MAL_HAS_EXTERN_failNow 1
#define MAL_DEFINE_failNow(call) \
static MalType_Unit mal_detail_failNow(mal_call_t *call); \
void mal_ext_failNow(MalContext *context MAL_DETAIL_MAYBE_UNUSED) { \
    mal_call_t call = (mal_call_t){ .mal_detail_context = context }; \
    mal_detail_failNow(&call); \
} \
static MalType_Unit mal_detail_failNow( \
    mal_call_t *call \
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
