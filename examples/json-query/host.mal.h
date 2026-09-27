#ifndef MAL_BUILD_UMBRELLA
#include <mal.h>
#include "bytes.mal.h"

_Static_assert(MAL_C_ABI_VERSION == 0x000900u, "generated header requires mal C ABI 0x000900");
_Static_assert((sizeof((size_t)0) * CHAR_BIT) == 64, "size_t does not match the mal target pointer index width");

#ifndef MAL_GENERATED_INTERFACE_291FC7EBD04D6D9D_H
#define MAL_GENERATED_INTERFACE_291FC7EBD04D6D9D_H
/* Host-visible types */

typedef struct { uintptr_t bits; } MalType_InputAllocation;

#ifndef MAL_DETAIL_RAW_REPR_588c78fec824022e_DECLARED
#define MAL_DETAIL_RAW_REPR_588c78fec824022e_DECLARED
typedef struct MalRepr_Product_588c78fec824022e MalRepr_Product_588c78fec824022e;
#endif
#ifndef MAL_DETAIL_RAW_REPR_1e5c20e9f358150e_DECLARED
#define MAL_DETAIL_RAW_REPR_1e5c20e9f358150e_DECLARED
typedef struct MalRepr_Product_1e5c20e9f358150e MalRepr_Product_1e5c20e9f358150e;
#endif

#ifndef MAL_DETAIL_RAW_REPR_588c78fec824022e_DEFINED
#define MAL_DETAIL_RAW_REPR_588c78fec824022e_DEFINED
struct MalRepr_Product_588c78fec824022e {
    MalType_InputAllocation field_0;
    MalType_Address field_1;
    MalType_USize field_2;
};

#endif

#ifndef MAL_DETAIL_RAW_REPR_1e5c20e9f358150e_DEFINED
#define MAL_DETAIL_RAW_REPR_1e5c20e9f358150e_DEFINED
struct MalRepr_Product_1e5c20e9f358150e {
    MalType_Address field_0;
    MalType_USize field_1;
};

#endif

typedef MalRepr_Product_588c78fec824022e MalType_StdinBytes;
typedef MalRepr_Product_1e5c20e9f358150e MalType_OutputBuffer;

typedef struct { uintptr_t mal_detail_bits; } mal_InputAllocation_t;
#ifndef MAL_DETAIL_HOST_REPR_588c78fec824022e_DECLARED
#define MAL_DETAIL_HOST_REPR_588c78fec824022e_DECLARED
typedef struct mal_detail_repr_product_588c78fec824022e mal_repr_product_588c78fec824022e_t;
#endif
#ifndef MAL_DETAIL_HOST_REPR_1e5c20e9f358150e_DECLARED
#define MAL_DETAIL_HOST_REPR_1e5c20e9f358150e_DECLARED
typedef struct mal_detail_repr_product_1e5c20e9f358150e mal_repr_product_1e5c20e9f358150e_t;
#endif
typedef mal_repr_product_588c78fec824022e_t mal_StdinBytes_t;
typedef mal_repr_product_1e5c20e9f358150e_t mal_OutputBuffer_t;

#ifndef MAL_DETAIL_HOST_REPR_588c78fec824022e_DEFINED
#define MAL_DETAIL_HOST_REPR_588c78fec824022e_DEFINED
struct mal_detail_repr_product_588c78fec824022e {
    mal_InputAllocation_t field_0;
    mal_Address_t field_1;
    mal_USize_t field_2;
};
#endif

#ifndef MAL_DETAIL_HOST_REPR_1e5c20e9f358150e_DEFINED
#define MAL_DETAIL_HOST_REPR_1e5c20e9f358150e_DEFINED
struct mal_detail_repr_product_1e5c20e9f358150e {
    mal_Address_t field_0;
    mal_USize_t field_1;
};
#endif

/* Type helpers */

static inline mal_InputAllocation_t mal_detail_to_host_InputAllocation(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, MalType_InputAllocation value);
static inline MalType_InputAllocation mal_InputAllocation_return(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, mal_InputAllocation_t value);

#ifndef MAL_DETAIL_HOST_REPR_588c78fec824022e_HELPERS
#define MAL_DETAIL_HOST_REPR_588c78fec824022e_HELPERS
#define MAL_DETAIL_TO_HOST_588c78fec824022e (mal_repr_product_588c78fec824022e_t){ .field_0 = (mal_InputAllocation_t){ .mal_detail_bits = value.field_0.bits }, .field_1 = value.field_1, .field_2 = value.field_2 }
MAL_DETAIL_DEFINE_CONVERSION(mal_detail_to_host_588c78fec824022e, mal_repr_product_588c78fec824022e_t, MalRepr_Product_588c78fec824022e, MAL_DETAIL_TO_HOST_588c78fec824022e)
#define MAL_DETAIL_TO_RAW_588c78fec824022e (MalRepr_Product_588c78fec824022e){ .field_0 = (MalType_InputAllocation){ .bits = value.field_0.mal_detail_bits }, .field_1 = mal_Address_return(call, value.field_1), .field_2 = value.field_2 }
MAL_DETAIL_DEFINE_CONVERSION(mal_repr_product_588c78fec824022e_return, MalRepr_Product_588c78fec824022e, mal_repr_product_588c78fec824022e_t, MAL_DETAIL_TO_RAW_588c78fec824022e)
#endif

#ifndef MAL_DETAIL_HOST_REPR_1e5c20e9f358150e_HELPERS
#define MAL_DETAIL_HOST_REPR_1e5c20e9f358150e_HELPERS
#define MAL_DETAIL_TO_HOST_1e5c20e9f358150e (mal_repr_product_1e5c20e9f358150e_t){ .field_0 = value.field_0, .field_1 = value.field_1 }
MAL_DETAIL_DEFINE_CONVERSION(mal_detail_to_host_1e5c20e9f358150e, mal_repr_product_1e5c20e9f358150e_t, MalRepr_Product_1e5c20e9f358150e, MAL_DETAIL_TO_HOST_1e5c20e9f358150e)
#define MAL_DETAIL_TO_RAW_1e5c20e9f358150e (MalRepr_Product_1e5c20e9f358150e){ .field_0 = mal_Address_return(call, value.field_0), .field_1 = value.field_1 }
MAL_DETAIL_DEFINE_CONVERSION(mal_repr_product_1e5c20e9f358150e_return, MalRepr_Product_1e5c20e9f358150e, mal_repr_product_1e5c20e9f358150e_t, MAL_DETAIL_TO_RAW_1e5c20e9f358150e)
#endif

static inline mal_InputAllocation_t mal_InputAllocation_from_bits(uintptr_t bits) {
    return (mal_InputAllocation_t){ .mal_detail_bits = bits };
}

static inline mal_InputAllocation_t mal_detail_to_host_InputAllocation(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, MalType_InputAllocation value) {
    return mal_InputAllocation_from_bits(value.bits);
}

static inline uintptr_t mal_InputAllocation_to_bits(mal_InputAllocation_t value) {
    return value.mal_detail_bits;
}

static inline MalType_InputAllocation mal_InputAllocation_return(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, mal_InputAllocation_t value) {
    return (MalType_InputAllocation){ .bits = value.mal_detail_bits };
}

MAL_DETAIL_DEFINE_CONVERSION(mal_StdinBytes_return, MalType_StdinBytes, mal_StdinBytes_t, MAL_DETAIL_TO_RAW_588c78fec824022e)
MAL_DETAIL_DEFINE_CONVERSION(mal_OutputBuffer_return, MalType_OutputBuffer, mal_OutputBuffer_t, MAL_DETAIL_TO_RAW_1e5c20e9f358150e)

/* Canonical memory access */

#ifndef MAL_DETAIL_MEMORY_REPR_1e5c20e9f358150e_HELPERS
#define MAL_DETAIL_MEMORY_REPR_1e5c20e9f358150e_HELPERS
#define MAL_DETAIL_MEMORY_FIELDS_1e5c20e9f358150e(unit, value) \
value(field_0, mal_detail_memory_read_Address, mal_detail_memory_write_Address, 0) \
value(field_1, mal_detail_memory_read_USize, mal_detail_memory_write_USize, 8)
MAL_DETAIL_DEFINE_MEMORY_PRODUCT(mal_detail_memory_read_1e5c20e9f358150e, mal_detail_memory_write_1e5c20e9f358150e, mal_repr_product_1e5c20e9f358150e_t, MAL_DETAIL_MEMORY_FIELDS_1e5c20e9f358150e)
#endif

MAL_DETAIL_DEFINE_MEMORY_ALIAS(mal_OutputBuffer_read, mal_OutputBuffer_write, mal_OutputBuffer_t, 16, mal_detail_memory_read_1e5c20e9f358150e, mal_detail_memory_write_1e5c20e9f358150e)

/* External operations */

MalType_StdinBytes mal_ext_readStdin(MalContext *context);
void mal_ext_releaseInput(MalContext *context, MalType_InputAllocation value);
MalType_OutputBuffer mal_ext_outputBuffer(MalContext *context);
void mal_ext_writeBytes(MalContext *context, MalType_Address argument_0, MalType_USize argument_1);

/* External definition helpers */

#define MAL_HAS_EXTERN_readStdin 1
#define MAL_DEFINE_readStdin(call) \
static MalType_StdinBytes mal_detail_readStdin(mal_call_t *call); \
MalType_StdinBytes mal_ext_readStdin(MalContext *context MAL_DETAIL_MAYBE_UNUSED) { \
    mal_call_t call = (mal_call_t){ .mal_detail_context = context }; \
    return mal_detail_readStdin(&call); \
} \
static MalType_StdinBytes mal_detail_readStdin( \
    mal_call_t *call \
)

#define MAL_HAS_EXTERN_releaseInput 1
#define MAL_DEFINE_releaseInput(call, value) \
static MalType_Unit mal_detail_releaseInput(mal_call_t *call, mal_InputAllocation_t value); \
void mal_ext_releaseInput(MalContext *context MAL_DETAIL_MAYBE_UNUSED, MalType_InputAllocation value) { \
    mal_call_t call = (mal_call_t){ .mal_detail_context = context }; \
    mal_detail_releaseInput(&call, (mal_InputAllocation_t){ .mal_detail_bits = value.bits }); \
} \
static MalType_Unit mal_detail_releaseInput( \
    mal_call_t *call, \
    mal_InputAllocation_t value \
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
    mal_detail_writeBytes(&call, mal_detail_to_host_1e5c20e9f358150e(&call, (MalRepr_Product_1e5c20e9f358150e){ .field_0 = argument_0, .field_1 = argument_1 })); \
} \
static MalType_Unit mal_detail_writeBytes( \
    mal_call_t *call, \
    mal_OutputBuffer_t value \
)

#endif
#endif
