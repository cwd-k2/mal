#ifndef MAL_BUILD_UMBRELLA
#include <mal.h>
#include <string.h>

_Static_assert(MAL_C_ABI_VERSION == 0x000900u, "generated header requires mal C ABI 0x000900");
_Static_assert((sizeof((size_t)0) * CHAR_BIT) == 64, "size_t does not match the mal target pointer index width");

#ifndef MAL_GENERATED_INTERFACE_CA23A925448F8AEC_H
#define MAL_GENERATED_INTERFACE_CA23A925448F8AEC_H
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

#ifndef MAL_DETAIL_HOST_REPR_917e57f57ce609a1_HELPERS
#define MAL_DETAIL_HOST_REPR_917e57f57ce609a1_HELPERS
static inline MalRepr_Product_917e57f57ce609a1 mal_repr_product_917e57f57ce609a1_return(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, mal_repr_product_917e57f57ce609a1_t value) {
    return (MalRepr_Product_917e57f57ce609a1){ .field_0 = (MalType_Allocator){ .bits = value.field_0.mal_detail_bits }, .field_1 = value.field_1 };
}

#endif

#ifndef MAL_DETAIL_HOST_REPR_1e5c20e9f358150e_HELPERS
#define MAL_DETAIL_HOST_REPR_1e5c20e9f358150e_HELPERS
static inline MalRepr_Product_1e5c20e9f358150e mal_repr_product_1e5c20e9f358150e_return(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, mal_repr_product_1e5c20e9f358150e_t value) {
    return (MalRepr_Product_1e5c20e9f358150e){ .field_0 = mal_Address_return(call, value.field_0), .field_1 = value.field_1 };
}

#endif

#ifndef MAL_DETAIL_HOST_REPR_65117c10bbbba260_HELPERS
#define MAL_DETAIL_HOST_REPR_65117c10bbbba260_HELPERS
static inline MalRepr_Product_65117c10bbbba260 mal_repr_product_65117c10bbbba260_return(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, mal_repr_product_65117c10bbbba260_t value) {
    return (MalRepr_Product_65117c10bbbba260){ .field_0 = (MalType_File){ .bits = value.field_0.mal_detail_bits }, .field_1 = mal_Address_return(call, value.field_1), .field_2 = value.field_2, .field_3 = value.field_3 };
}

#endif

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
    return (MalRepr_Product_1e5c20e9f358150e){ .field_0 = mal_Address_return(call, value.field_0), .field_1 = value.field_1 };
}

static inline MalType_ReadableBytes mal_ReadableBytes_return(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, mal_ReadableBytes_t value) {
    return (MalRepr_Product_1e5c20e9f358150e){ .field_0 = mal_Address_return(call, value.field_0), .field_1 = value.field_1 };
}

static inline MalType_OutputBuffer mal_OutputBuffer_return(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, mal_OutputBuffer_t value) {
    return (MalRepr_Product_1e5c20e9f358150e){ .field_0 = mal_Address_return(call, value.field_0), .field_1 = value.field_1 };
}

/* Canonical memory access */

#ifndef MAL_DETAIL_MEMORY_Address_HELPERS
#define MAL_DETAIL_MEMORY_Address_HELPERS
static inline mal_Address_t mal_detail_memory_read_Address(mal_call_t *call, const uint8_t *source) {
    mal_Address_t value;
    memcpy(&value, source, sizeof(value));
    return mal_Address_return(call, value);
}

static inline void mal_detail_memory_write_Address(mal_call_t *call, uint8_t *destination, mal_Address_t value) {
    mal_Address_return(call, value);
    memcpy(destination, &value, sizeof(value));
}

#endif

#ifndef MAL_DETAIL_MEMORY_USize_HELPERS
#define MAL_DETAIL_MEMORY_USize_HELPERS
static inline mal_USize_t mal_detail_memory_read_USize(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, const uint8_t *source) {
    mal_USize_t value;
    memcpy(&value, source, sizeof(value));
    return value;
}

static inline void mal_detail_memory_write_USize(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, uint8_t *destination, mal_USize_t value) {
    memcpy(destination, &value, sizeof(value));
}

#endif

#ifndef MAL_DETAIL_MEMORY_REPR_1e5c20e9f358150e_HELPERS
#define MAL_DETAIL_MEMORY_REPR_1e5c20e9f358150e_HELPERS
static inline mal_repr_product_1e5c20e9f358150e_t mal_detail_memory_read_1e5c20e9f358150e(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, const uint8_t *source MAL_DETAIL_MAYBE_UNUSED) {
    mal_repr_product_1e5c20e9f358150e_t value;
    value.field_0 = mal_detail_memory_read_Address(call, source + 0);
    value.field_1 = mal_detail_memory_read_USize(call, source + 8);
    return value;
}

static inline void mal_detail_memory_write_1e5c20e9f358150e(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, uint8_t *destination MAL_DETAIL_MAYBE_UNUSED, mal_repr_product_1e5c20e9f358150e_t value) {
    mal_detail_memory_write_Address(call, destination + 0, value.field_0);
    mal_detail_memory_write_USize(call, destination + 8, value.field_1);
}

#endif

#ifndef MAL_DETAIL_MEMORY_REPR_bebfef0e190e1724_HELPERS
#define MAL_DETAIL_MEMORY_REPR_bebfef0e190e1724_HELPERS
static inline mal_repr_product_bebfef0e190e1724_t mal_detail_memory_read_bebfef0e190e1724(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, const uint8_t *source MAL_DETAIL_MAYBE_UNUSED) {
    mal_repr_product_bebfef0e190e1724_t value;
    value.field_0 = mal_detail_memory_read_Address(call, source + 0);
    value.field_1 = mal_detail_memory_read_USize(call, source + 8);
    value.field_2 = mal_detail_memory_read_USize(call, source + 16);
    return value;
}

static inline void mal_detail_memory_write_bebfef0e190e1724(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, uint8_t *destination MAL_DETAIL_MAYBE_UNUSED, mal_repr_product_bebfef0e190e1724_t value) {
    mal_detail_memory_write_Address(call, destination + 0, value.field_0);
    mal_detail_memory_write_USize(call, destination + 8, value.field_1);
    mal_detail_memory_write_USize(call, destination + 16, value.field_2);
}

#endif

static inline mal_AllocatedBytes_t mal_AllocatedBytes_read(mal_call_t *call, mal_Address_t address, mal_USize_t index) {
    mal_Address_return(call, address);
    return mal_detail_memory_read_1e5c20e9f358150e(call, (const uint8_t *)address + (index * 16));
}

static inline void mal_AllocatedBytes_write(mal_call_t *call, mal_Address_t address, mal_USize_t index, mal_AllocatedBytes_t value) {
    mal_Address_return(call, address);
    mal_detail_memory_write_1e5c20e9f358150e(call, (uint8_t *)address + (index * 16), value);
}

static inline mal_ByteBuffer_t mal_ByteBuffer_read(mal_call_t *call, mal_Address_t address, mal_USize_t index) {
    mal_Address_return(call, address);
    return mal_detail_memory_read_bebfef0e190e1724(call, (const uint8_t *)address + (index * 24));
}

static inline void mal_ByteBuffer_write(mal_call_t *call, mal_Address_t address, mal_USize_t index, mal_ByteBuffer_t value) {
    mal_Address_return(call, address);
    mal_detail_memory_write_bebfef0e190e1724(call, (uint8_t *)address + (index * 24), value);
}

static inline mal_ReadableBytes_t mal_ReadableBytes_read(mal_call_t *call, mal_Address_t address, mal_USize_t index) {
    mal_Address_return(call, address);
    return mal_detail_memory_read_1e5c20e9f358150e(call, (const uint8_t *)address + (index * 16));
}

static inline void mal_ReadableBytes_write(mal_call_t *call, mal_Address_t address, mal_USize_t index, mal_ReadableBytes_t value) {
    mal_Address_return(call, address);
    mal_detail_memory_write_1e5c20e9f358150e(call, (uint8_t *)address + (index * 16), value);
}

static inline mal_WritableBytes_t mal_WritableBytes_read(mal_call_t *call, mal_Address_t address, mal_USize_t index) {
    mal_Address_return(call, address);
    return mal_detail_memory_read_1e5c20e9f358150e(call, (const uint8_t *)address + (index * 16));
}

static inline void mal_WritableBytes_write(mal_call_t *call, mal_Address_t address, mal_USize_t index, mal_WritableBytes_t value) {
    mal_Address_return(call, address);
    mal_detail_memory_write_1e5c20e9f358150e(call, (uint8_t *)address + (index * 16), value);
}

static inline mal_OutputBuffer_t mal_OutputBuffer_read(mal_call_t *call, mal_Address_t address, mal_USize_t index) {
    mal_Address_return(call, address);
    return mal_detail_memory_read_1e5c20e9f358150e(call, (const uint8_t *)address + (index * 16));
}

static inline void mal_OutputBuffer_write(mal_call_t *call, mal_Address_t address, mal_USize_t index, mal_OutputBuffer_t value) {
    mal_Address_return(call, address);
    mal_detail_memory_write_1e5c20e9f358150e(call, (uint8_t *)address + (index * 16), value);
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
    return mal_detail_allocateBuffer(&call, (mal_repr_product_917e57f57ce609a1_t){ .field_0 = (mal_Allocator_t){ .mal_detail_bits = ((MalRepr_Product_917e57f57ce609a1){ .field_0 = argument_0, .field_1 = argument_1 }).field_0.bits }, .field_1 = ((MalRepr_Product_917e57f57ce609a1){ .field_0 = argument_0, .field_1 = argument_1 }).field_1 }); \
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
    return mal_detail_openReadWriteCreate(&call, (mal_ReadableBytes_t){ .field_0 = ((MalRepr_Product_1e5c20e9f358150e){ .field_0 = argument_0, .field_1 = argument_1 }).field_0, .field_1 = ((MalRepr_Product_1e5c20e9f358150e){ .field_0 = argument_0, .field_1 = argument_1 }).field_1 }); \
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
    return mal_detail_readFile(&call, (mal_repr_product_65117c10bbbba260_t){ .field_0 = (mal_File_t){ .mal_detail_bits = ((MalRepr_Product_65117c10bbbba260){ .field_0 = argument_0, .field_1 = argument_1, .field_2 = argument_2, .field_3 = argument_3 }).field_0.bits }, .field_1 = ((MalRepr_Product_65117c10bbbba260){ .field_0 = argument_0, .field_1 = argument_1, .field_2 = argument_2, .field_3 = argument_3 }).field_1, .field_2 = ((MalRepr_Product_65117c10bbbba260){ .field_0 = argument_0, .field_1 = argument_1, .field_2 = argument_2, .field_3 = argument_3 }).field_2, .field_3 = ((MalRepr_Product_65117c10bbbba260){ .field_0 = argument_0, .field_1 = argument_1, .field_2 = argument_2, .field_3 = argument_3 }).field_3 }); \
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
    return mal_detail_writeFile(&call, (mal_repr_product_65117c10bbbba260_t){ .field_0 = (mal_File_t){ .mal_detail_bits = ((MalRepr_Product_65117c10bbbba260){ .field_0 = argument_0, .field_1 = argument_1, .field_2 = argument_2, .field_3 = argument_3 }).field_0.bits }, .field_1 = ((MalRepr_Product_65117c10bbbba260){ .field_0 = argument_0, .field_1 = argument_1, .field_2 = argument_2, .field_3 = argument_3 }).field_1, .field_2 = ((MalRepr_Product_65117c10bbbba260){ .field_0 = argument_0, .field_1 = argument_1, .field_2 = argument_2, .field_3 = argument_3 }).field_2, .field_3 = ((MalRepr_Product_65117c10bbbba260){ .field_0 = argument_0, .field_1 = argument_1, .field_2 = argument_2, .field_3 = argument_3 }).field_3 }); \
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
    mal_detail_writeStdout(&call, (mal_ReadableBytes_t){ .field_0 = ((MalRepr_Product_1e5c20e9f358150e){ .field_0 = argument_0, .field_1 = argument_1 }).field_0, .field_1 = ((MalRepr_Product_1e5c20e9f358150e){ .field_0 = argument_0, .field_1 = argument_1 }).field_1 }); \
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
    mal_detail_writeStderr(&call, (mal_ReadableBytes_t){ .field_0 = ((MalRepr_Product_1e5c20e9f358150e){ .field_0 = argument_0, .field_1 = argument_1 }).field_0, .field_1 = ((MalRepr_Product_1e5c20e9f358150e){ .field_0 = argument_0, .field_1 = argument_1 }).field_1 }); \
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
