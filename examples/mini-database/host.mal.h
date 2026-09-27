#ifndef MAL_BUILD_UMBRELLA
#include <mal.h>

_Static_assert(MAL_C_ABI_VERSION == 0x000900u, "generated header requires mal C ABI 0x000900");
_Static_assert((sizeof((size_t)0) * CHAR_BIT) == 64, "size_t does not match the mal target pointer index width");

#ifndef MAL_GENERATED_INTERFACE_1D16C337D6BA1A7C_H
#define MAL_GENERATED_INTERFACE_1D16C337D6BA1A7C_H
/* Host-visible types */

typedef struct { uintptr_t bits; } MalType_Allocator;
typedef struct { uintptr_t bits; } MalType_File;

#ifndef MAL_DETAIL_RAW_REPR_917e57f57ce609a1_DECLARED
#define MAL_DETAIL_RAW_REPR_917e57f57ce609a1_DECLARED
typedef struct MalRepr_Product_917e57f57ce609a1 MalRepr_Product_917e57f57ce609a1;
#endif
#ifndef MAL_DETAIL_RAW_REPR_1e5c20e9f358150e_DECLARED
#define MAL_DETAIL_RAW_REPR_1e5c20e9f358150e_DECLARED
typedef struct MalRepr_Product_1e5c20e9f358150e MalRepr_Product_1e5c20e9f358150e;
#endif
#ifndef MAL_DETAIL_RAW_REPR_65117c10bbbba260_DECLARED
#define MAL_DETAIL_RAW_REPR_65117c10bbbba260_DECLARED
typedef struct MalRepr_Product_65117c10bbbba260 MalRepr_Product_65117c10bbbba260;
#endif

#ifndef MAL_DETAIL_RAW_REPR_917e57f57ce609a1_DEFINED
#define MAL_DETAIL_RAW_REPR_917e57f57ce609a1_DEFINED
struct MalRepr_Product_917e57f57ce609a1 {
    MalType_Allocator field_0;
    MalType_USize field_1;
};

#endif

#ifndef MAL_DETAIL_RAW_REPR_1e5c20e9f358150e_DEFINED
#define MAL_DETAIL_RAW_REPR_1e5c20e9f358150e_DEFINED
struct MalRepr_Product_1e5c20e9f358150e {
    MalType_Address field_0;
    MalType_USize field_1;
};

#endif

#ifndef MAL_DETAIL_RAW_REPR_65117c10bbbba260_DEFINED
#define MAL_DETAIL_RAW_REPR_65117c10bbbba260_DEFINED
struct MalRepr_Product_65117c10bbbba260 {
    MalType_File field_0;
    MalType_Address field_1;
    MalType_USize field_2;
    MalType_USize field_3;
};

#endif

typedef MalRepr_Product_1e5c20e9f358150e MalType_AllocatedBytes;
typedef MalRepr_Product_1e5c20e9f358150e MalType_ReadableBytes;
typedef MalRepr_Product_1e5c20e9f358150e MalType_OutputBuffer;

typedef struct { uintptr_t mal_detail_bits; } mal_Allocator_t;
typedef struct { uintptr_t mal_detail_bits; } mal_File_t;
#ifndef MAL_DETAIL_HOST_REPR_917e57f57ce609a1_DECLARED
#define MAL_DETAIL_HOST_REPR_917e57f57ce609a1_DECLARED
typedef struct mal_detail_repr_product_917e57f57ce609a1 mal_repr_product_917e57f57ce609a1_t;
#endif
#ifndef MAL_DETAIL_HOST_REPR_1e5c20e9f358150e_DECLARED
#define MAL_DETAIL_HOST_REPR_1e5c20e9f358150e_DECLARED
typedef struct mal_detail_repr_product_1e5c20e9f358150e mal_repr_product_1e5c20e9f358150e_t;
#endif
#ifndef MAL_DETAIL_HOST_REPR_65117c10bbbba260_DECLARED
#define MAL_DETAIL_HOST_REPR_65117c10bbbba260_DECLARED
typedef struct mal_detail_repr_product_65117c10bbbba260 mal_repr_product_65117c10bbbba260_t;
#endif
#ifndef MAL_DETAIL_HOST_REPR_bebfef0e190e1724_DECLARED
#define MAL_DETAIL_HOST_REPR_bebfef0e190e1724_DECLARED
typedef struct mal_detail_repr_product_bebfef0e190e1724 mal_repr_product_bebfef0e190e1724_t;
#endif
typedef mal_repr_product_1e5c20e9f358150e_t mal_AllocatedBytes_t;
typedef mal_repr_product_bebfef0e190e1724_t mal_ByteBuffer_t;
typedef mal_repr_product_1e5c20e9f358150e_t mal_ReadableBytes_t;
typedef mal_repr_product_1e5c20e9f358150e_t mal_WritableBytes_t;
typedef mal_repr_product_1e5c20e9f358150e_t mal_OutputBuffer_t;

#ifndef MAL_DETAIL_HOST_REPR_917e57f57ce609a1_DEFINED
#define MAL_DETAIL_HOST_REPR_917e57f57ce609a1_DEFINED
struct mal_detail_repr_product_917e57f57ce609a1 {
    mal_Allocator_t field_0;
    mal_USize_t field_1;
};
#endif

#ifndef MAL_DETAIL_HOST_REPR_1e5c20e9f358150e_DEFINED
#define MAL_DETAIL_HOST_REPR_1e5c20e9f358150e_DEFINED
struct mal_detail_repr_product_1e5c20e9f358150e {
    mal_Address_t field_0;
    mal_USize_t field_1;
};
#endif

#ifndef MAL_DETAIL_HOST_REPR_65117c10bbbba260_DEFINED
#define MAL_DETAIL_HOST_REPR_65117c10bbbba260_DEFINED
struct mal_detail_repr_product_65117c10bbbba260 {
    mal_File_t field_0;
    mal_Address_t field_1;
    mal_USize_t field_2;
    mal_USize_t field_3;
};
#endif

#ifndef MAL_DETAIL_HOST_REPR_bebfef0e190e1724_DEFINED
#define MAL_DETAIL_HOST_REPR_bebfef0e190e1724_DEFINED
struct mal_detail_repr_product_bebfef0e190e1724 {
    mal_Address_t field_0;
    mal_USize_t field_1;
    mal_USize_t field_2;
};
#endif

/* Type helpers */

static inline mal_Allocator_t mal_detail_to_host_Allocator(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, MalType_Allocator value);
static inline MalType_Allocator mal_Allocator_return(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, mal_Allocator_t value);
static inline mal_File_t mal_detail_to_host_File(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, MalType_File value);
static inline MalType_File mal_File_return(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, mal_File_t value);

#ifndef MAL_DETAIL_HOST_REPR_917e57f57ce609a1_HELPERS
#define MAL_DETAIL_HOST_REPR_917e57f57ce609a1_HELPERS
#define MAL_DETAIL_TO_HOST_917e57f57ce609a1 (mal_repr_product_917e57f57ce609a1_t){ .field_0 = (mal_Allocator_t){ .mal_detail_bits = value.field_0.bits }, .field_1 = value.field_1 }
MAL_DETAIL_DEFINE_CONVERSION(mal_detail_to_host_917e57f57ce609a1, mal_repr_product_917e57f57ce609a1_t, MalRepr_Product_917e57f57ce609a1, MAL_DETAIL_TO_HOST_917e57f57ce609a1)
#define MAL_DETAIL_TO_RAW_917e57f57ce609a1 (MalRepr_Product_917e57f57ce609a1){ .field_0 = (MalType_Allocator){ .bits = value.field_0.mal_detail_bits }, .field_1 = value.field_1 }
MAL_DETAIL_DEFINE_CONVERSION(mal_repr_product_917e57f57ce609a1_return, MalRepr_Product_917e57f57ce609a1, mal_repr_product_917e57f57ce609a1_t, MAL_DETAIL_TO_RAW_917e57f57ce609a1)
#endif

#ifndef MAL_DETAIL_HOST_REPR_1e5c20e9f358150e_HELPERS
#define MAL_DETAIL_HOST_REPR_1e5c20e9f358150e_HELPERS
#define MAL_DETAIL_TO_HOST_1e5c20e9f358150e (mal_repr_product_1e5c20e9f358150e_t){ .field_0 = value.field_0, .field_1 = value.field_1 }
MAL_DETAIL_DEFINE_CONVERSION(mal_detail_to_host_1e5c20e9f358150e, mal_repr_product_1e5c20e9f358150e_t, MalRepr_Product_1e5c20e9f358150e, MAL_DETAIL_TO_HOST_1e5c20e9f358150e)
#define MAL_DETAIL_TO_RAW_1e5c20e9f358150e (MalRepr_Product_1e5c20e9f358150e){ .field_0 = mal_Address_return(call, value.field_0), .field_1 = value.field_1 }
MAL_DETAIL_DEFINE_CONVERSION(mal_repr_product_1e5c20e9f358150e_return, MalRepr_Product_1e5c20e9f358150e, mal_repr_product_1e5c20e9f358150e_t, MAL_DETAIL_TO_RAW_1e5c20e9f358150e)
#endif

#ifndef MAL_DETAIL_HOST_REPR_65117c10bbbba260_HELPERS
#define MAL_DETAIL_HOST_REPR_65117c10bbbba260_HELPERS
#define MAL_DETAIL_TO_HOST_65117c10bbbba260 (mal_repr_product_65117c10bbbba260_t){ .field_0 = (mal_File_t){ .mal_detail_bits = value.field_0.bits }, .field_1 = value.field_1, .field_2 = value.field_2, .field_3 = value.field_3 }
MAL_DETAIL_DEFINE_CONVERSION(mal_detail_to_host_65117c10bbbba260, mal_repr_product_65117c10bbbba260_t, MalRepr_Product_65117c10bbbba260, MAL_DETAIL_TO_HOST_65117c10bbbba260)
#define MAL_DETAIL_TO_RAW_65117c10bbbba260 (MalRepr_Product_65117c10bbbba260){ .field_0 = (MalType_File){ .bits = value.field_0.mal_detail_bits }, .field_1 = mal_Address_return(call, value.field_1), .field_2 = value.field_2, .field_3 = value.field_3 }
MAL_DETAIL_DEFINE_CONVERSION(mal_repr_product_65117c10bbbba260_return, MalRepr_Product_65117c10bbbba260, mal_repr_product_65117c10bbbba260_t, MAL_DETAIL_TO_RAW_65117c10bbbba260)
#endif

static inline mal_Allocator_t mal_Allocator_from_bits(uintptr_t bits) {
    return (mal_Allocator_t){ .mal_detail_bits = bits };
}

static inline mal_Allocator_t mal_detail_to_host_Allocator(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, MalType_Allocator value) {
    return mal_Allocator_from_bits(value.bits);
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

static inline mal_File_t mal_detail_to_host_File(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, MalType_File value) {
    return mal_File_from_bits(value.bits);
}

static inline uintptr_t mal_File_to_bits(mal_File_t value) {
    return value.mal_detail_bits;
}

static inline MalType_File mal_File_return(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, mal_File_t value) {
    return (MalType_File){ .bits = value.mal_detail_bits };
}

MAL_DETAIL_DEFINE_CONVERSION(mal_AllocatedBytes_return, MalType_AllocatedBytes, mal_AllocatedBytes_t, MAL_DETAIL_TO_RAW_1e5c20e9f358150e)
MAL_DETAIL_DEFINE_CONVERSION(mal_ReadableBytes_return, MalType_ReadableBytes, mal_ReadableBytes_t, MAL_DETAIL_TO_RAW_1e5c20e9f358150e)
MAL_DETAIL_DEFINE_CONVERSION(mal_OutputBuffer_return, MalType_OutputBuffer, mal_OutputBuffer_t, MAL_DETAIL_TO_RAW_1e5c20e9f358150e)

/* Canonical memory access */

#ifndef MAL_DETAIL_MEMORY_REPR_1e5c20e9f358150e_HELPERS
#define MAL_DETAIL_MEMORY_REPR_1e5c20e9f358150e_HELPERS
#define MAL_DETAIL_MEMORY_FIELDS_1e5c20e9f358150e(unit, value) \
value(field_0, mal_detail_memory_read_Address, mal_detail_memory_write_Address, 0) \
value(field_1, mal_detail_memory_read_USize, mal_detail_memory_write_USize, 8)
MAL_DETAIL_DEFINE_MEMORY_PRODUCT(mal_detail_memory_read_1e5c20e9f358150e, mal_detail_memory_write_1e5c20e9f358150e, mal_repr_product_1e5c20e9f358150e_t, MAL_DETAIL_MEMORY_FIELDS_1e5c20e9f358150e)
#endif

#ifndef MAL_DETAIL_MEMORY_REPR_bebfef0e190e1724_HELPERS
#define MAL_DETAIL_MEMORY_REPR_bebfef0e190e1724_HELPERS
#define MAL_DETAIL_MEMORY_FIELDS_bebfef0e190e1724(unit, value) \
value(field_0, mal_detail_memory_read_Address, mal_detail_memory_write_Address, 0) \
value(field_1, mal_detail_memory_read_USize, mal_detail_memory_write_USize, 8) \
value(field_2, mal_detail_memory_read_USize, mal_detail_memory_write_USize, 16)
MAL_DETAIL_DEFINE_MEMORY_PRODUCT(mal_detail_memory_read_bebfef0e190e1724, mal_detail_memory_write_bebfef0e190e1724, mal_repr_product_bebfef0e190e1724_t, MAL_DETAIL_MEMORY_FIELDS_bebfef0e190e1724)
#endif

MAL_DETAIL_DEFINE_MEMORY_ALIAS(mal_AllocatedBytes_read, mal_AllocatedBytes_write, mal_AllocatedBytes_t, 16, mal_detail_memory_read_1e5c20e9f358150e, mal_detail_memory_write_1e5c20e9f358150e)

MAL_DETAIL_DEFINE_MEMORY_ALIAS(mal_ByteBuffer_read, mal_ByteBuffer_write, mal_ByteBuffer_t, 24, mal_detail_memory_read_bebfef0e190e1724, mal_detail_memory_write_bebfef0e190e1724)

MAL_DETAIL_DEFINE_MEMORY_ALIAS(mal_ReadableBytes_read, mal_ReadableBytes_write, mal_ReadableBytes_t, 16, mal_detail_memory_read_1e5c20e9f358150e, mal_detail_memory_write_1e5c20e9f358150e)

MAL_DETAIL_DEFINE_MEMORY_ALIAS(mal_WritableBytes_read, mal_WritableBytes_write, mal_WritableBytes_t, 16, mal_detail_memory_read_1e5c20e9f358150e, mal_detail_memory_write_1e5c20e9f358150e)

MAL_DETAIL_DEFINE_MEMORY_ALIAS(mal_OutputBuffer_read, mal_OutputBuffer_write, mal_OutputBuffer_t, 16, mal_detail_memory_read_1e5c20e9f358150e, mal_detail_memory_write_1e5c20e9f358150e)

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
static MalType_AllocatedBytes mal_detail_allocateBuffer(mal_call_t *call, mal_repr_product_917e57f57ce609a1_t value); \
MalType_AllocatedBytes mal_ext_allocateBuffer(MalContext *context MAL_DETAIL_MAYBE_UNUSED, MalType_Allocator argument_0, MalType_USize argument_1) { \
    mal_call_t call = (mal_call_t){ .mal_detail_context = context }; \
    return mal_detail_allocateBuffer(&call, mal_detail_to_host_917e57f57ce609a1(&call, (MalRepr_Product_917e57f57ce609a1){ .field_0 = argument_0, .field_1 = argument_1 })); \
} \
static MalType_AllocatedBytes mal_detail_allocateBuffer( \
    mal_call_t *call, \
    mal_repr_product_917e57f57ce609a1_t value \
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
    return mal_detail_openReadWriteCreate(&call, mal_detail_to_host_1e5c20e9f358150e(&call, (MalRepr_Product_1e5c20e9f358150e){ .field_0 = argument_0, .field_1 = argument_1 })); \
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
static MalType_USize mal_detail_readFile(mal_call_t *call, mal_repr_product_65117c10bbbba260_t value); \
MalType_USize mal_ext_readFile(MalContext *context MAL_DETAIL_MAYBE_UNUSED, MalType_File argument_0, MalType_Address argument_1, MalType_USize argument_2, MalType_USize argument_3) { \
    mal_call_t call = (mal_call_t){ .mal_detail_context = context }; \
    return mal_detail_readFile(&call, mal_detail_to_host_65117c10bbbba260(&call, (MalRepr_Product_65117c10bbbba260){ .field_0 = argument_0, .field_1 = argument_1, .field_2 = argument_2, .field_3 = argument_3 })); \
} \
static MalType_USize mal_detail_readFile( \
    mal_call_t *call, \
    mal_repr_product_65117c10bbbba260_t value \
)

#define MAL_HAS_EXTERN_writeFile 1
#define MAL_DEFINE_writeFile(call, value) \
static MalType_USize mal_detail_writeFile(mal_call_t *call, mal_repr_product_65117c10bbbba260_t value); \
MalType_USize mal_ext_writeFile(MalContext *context MAL_DETAIL_MAYBE_UNUSED, MalType_File argument_0, MalType_Address argument_1, MalType_USize argument_2, MalType_USize argument_3) { \
    mal_call_t call = (mal_call_t){ .mal_detail_context = context }; \
    return mal_detail_writeFile(&call, mal_detail_to_host_65117c10bbbba260(&call, (MalRepr_Product_65117c10bbbba260){ .field_0 = argument_0, .field_1 = argument_1, .field_2 = argument_2, .field_3 = argument_3 })); \
} \
static MalType_USize mal_detail_writeFile( \
    mal_call_t *call, \
    mal_repr_product_65117c10bbbba260_t value \
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
    mal_detail_writeStdout(&call, mal_detail_to_host_1e5c20e9f358150e(&call, (MalRepr_Product_1e5c20e9f358150e){ .field_0 = argument_0, .field_1 = argument_1 })); \
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
    mal_detail_writeStderr(&call, mal_detail_to_host_1e5c20e9f358150e(&call, (MalRepr_Product_1e5c20e9f358150e){ .field_0 = argument_0, .field_1 = argument_1 })); \
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

#endif
#endif
