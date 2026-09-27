#ifndef MAL_PROGRAM_MAL_H
#define MAL_PROGRAM_MAL_H

#include <mal.h>

_Static_assert(MAL_C_ABI_VERSION == 0x000900u, "generated header requires mal C ABI 0x000900");
_Static_assert((sizeof((size_t)0) * CHAR_BIT) == 64, "size_t does not match the mal target pointer index width");

/* Host-visible types */

typedef struct { uintptr_t bits; } MalType_InputAllocation;

typedef struct MalRepr_Product_0 MalRepr_Product_0;
typedef struct MalRepr_Product_1 MalRepr_Product_1;

struct MalRepr_Product_0 {
    MalType_InputAllocation field_0;
    MalType_Address field_1;
    MalType_USize field_2;
};

struct MalRepr_Product_1 {
    MalType_Address field_0;
    MalType_USize field_1;
};

typedef MalRepr_Product_0 MalType_StdinBytes;
typedef MalRepr_Product_1 MalType_OutputBuffer;

typedef struct { uintptr_t mal_detail_bits; } mal_InputAllocation_t;
typedef struct mal_detail_repr_product_0 mal_repr_product_0_t;
typedef struct mal_detail_repr_product_1 mal_repr_product_1_t;
typedef mal_repr_product_0_t mal_StdinBytes_t;
typedef mal_repr_product_1_t mal_OutputBuffer_t;

struct mal_detail_repr_product_0 {
    mal_InputAllocation_t field_0;
    mal_Address_t field_1;
    mal_USize_t field_2;
};

struct mal_detail_repr_product_1 {
    mal_Address_t field_0;
    mal_USize_t field_1;
};

/* Type helpers */

static inline MalRepr_Product_0 mal_repr_product_0_return(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, mal_repr_product_0_t value) {
    return (MalRepr_Product_0){ .field_0 = (MalType_InputAllocation){ .bits = value.field_0.mal_detail_bits }, .field_1 = mal_Address_return(call, value.field_1), .field_2 = value.field_2 };
}

static inline MalRepr_Product_1 mal_repr_product_1_return(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, mal_repr_product_1_t value) {
    return (MalRepr_Product_1){ .field_0 = mal_Address_return(call, value.field_0), .field_1 = value.field_1 };
}

static inline mal_InputAllocation_t mal_InputAllocation_from_bits(uintptr_t bits) {
    return (mal_InputAllocation_t){ .mal_detail_bits = bits };
}

static inline uintptr_t mal_InputAllocation_to_bits(mal_InputAllocation_t value) {
    return value.mal_detail_bits;
}

static inline MalType_InputAllocation mal_InputAllocation_return(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, mal_InputAllocation_t value) {
    return (MalType_InputAllocation){ .bits = value.mal_detail_bits };
}

static inline MalType_StdinBytes mal_StdinBytes_return(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, mal_StdinBytes_t value) {
    return (MalRepr_Product_0){ .field_0 = (MalType_InputAllocation){ .bits = value.field_0.mal_detail_bits }, .field_1 = mal_Address_return(call, value.field_1), .field_2 = value.field_2 };
}

static inline MalType_OutputBuffer mal_OutputBuffer_return(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, mal_OutputBuffer_t value) {
    return (MalRepr_Product_1){ .field_0 = mal_Address_return(call, value.field_0), .field_1 = value.field_1 };
}

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
    mal_detail_writeBytes(&call, (mal_OutputBuffer_t){ .field_0 = ((MalRepr_Product_1){ .field_0 = argument_0, .field_1 = argument_1 }).field_0, .field_1 = ((MalRepr_Product_1){ .field_0 = argument_0, .field_1 = argument_1 }).field_1 }); \
} \
static MalType_Unit mal_detail_writeBytes( \
    mal_call_t *call, \
    mal_OutputBuffer_t value \
)

#endif
