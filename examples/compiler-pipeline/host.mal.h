#ifndef MAL_BUILD_UMBRELLA
#include <mal.h>

_Static_assert(MAL_C_ABI_VERSION == 0x000900u, "generated header requires mal C ABI 0x000900");
_Static_assert((sizeof((size_t)0) * CHAR_BIT) == 64, "size_t does not match the mal target pointer index width");

#ifndef MAL_GENERATED_INTERFACE_9DB990F806A190AD_H
#define MAL_GENERATED_INTERFACE_9DB990F806A190AD_H

/* Host-visible types */

typedef struct { uintptr_t bits; } MalType_SourceAllocation;

#ifndef MAL_DETAIL_RAW_REPR_1e5c20e9f358150e_DECLARED
#define MAL_DETAIL_RAW_REPR_1e5c20e9f358150e_DECLARED
typedef struct MalRepr_Product_1e5c20e9f358150e MalRepr_Product_1e5c20e9f358150e;
#endif
#ifndef MAL_DETAIL_RAW_REPR_24f7b1f5b59c8be7_DECLARED
#define MAL_DETAIL_RAW_REPR_24f7b1f5b59c8be7_DECLARED
typedef struct MalRepr_Product_24f7b1f5b59c8be7 MalRepr_Product_24f7b1f5b59c8be7;
#endif
#ifndef MAL_DETAIL_RAW_REPR_3cf383e0bf0efe5a_DECLARED
#define MAL_DETAIL_RAW_REPR_3cf383e0bf0efe5a_DECLARED
typedef struct MalRepr_Sum_3cf383e0bf0efe5a MalRepr_Sum_3cf383e0bf0efe5a;
#endif
#ifndef MAL_DETAIL_RAW_REPR_03dc1b0e40257776_DECLARED
#define MAL_DETAIL_RAW_REPR_03dc1b0e40257776_DECLARED
typedef struct MalRepr_Product_03dc1b0e40257776 MalRepr_Product_03dc1b0e40257776;
#endif
#ifndef MAL_DETAIL_RAW_REPR_4647c725c84fdeda_DECLARED
#define MAL_DETAIL_RAW_REPR_4647c725c84fdeda_DECLARED
typedef struct MalRepr_Sum_4647c725c84fdeda MalRepr_Sum_4647c725c84fdeda;
#endif

#ifndef MAL_DETAIL_REPR_FIELDS_1e5c20e9f358150e_DEFINED
#define MAL_DETAIL_REPR_FIELDS_1e5c20e9f358150e_DEFINED
#define MAL_DETAIL_REPR_FIELDS_1e5c20e9f358150e(field, context) \
field(context, 0, field_0, MalType_Address, mal_Address_t, MAL_DETAIL_REPR_IDENTITY, mal_Address_return) \
field(context, 1, field_1, MalType_USize, mal_USize_t, MAL_DETAIL_REPR_IDENTITY, MAL_DETAIL_REPR_IDENTITY)
#endif
#ifndef MAL_DETAIL_RAW_REPR_1e5c20e9f358150e_DEFINED
#define MAL_DETAIL_RAW_REPR_1e5c20e9f358150e_DEFINED
MAL_DETAIL_DEFINE_PRODUCT_REPR(MalRepr_Product_1e5c20e9f358150e, MAL_DETAIL_REPR_FIELDS_1e5c20e9f358150e, MAL_DETAIL_RAW_REPR_FIELD)
#endif

#ifndef MAL_DETAIL_REPR_FIELDS_24f7b1f5b59c8be7_DEFINED
#define MAL_DETAIL_REPR_FIELDS_24f7b1f5b59c8be7_DEFINED
#define MAL_DETAIL_REPR_FIELDS_24f7b1f5b59c8be7(field, context) \
field(context, 0, field_0, MalType_SourceAllocation, mal_SourceAllocation_t, mal_detail_to_host_SourceAllocation, mal_SourceAllocation_return) \
field(context, 1, field_1, MalType_Address, mal_Address_t, MAL_DETAIL_REPR_IDENTITY, mal_Address_return) \
field(context, 2, field_2, MalType_USize, mal_USize_t, MAL_DETAIL_REPR_IDENTITY, MAL_DETAIL_REPR_IDENTITY)
#endif
#ifndef MAL_DETAIL_RAW_REPR_24f7b1f5b59c8be7_DEFINED
#define MAL_DETAIL_RAW_REPR_24f7b1f5b59c8be7_DEFINED
MAL_DETAIL_DEFINE_PRODUCT_REPR(MalRepr_Product_24f7b1f5b59c8be7, MAL_DETAIL_REPR_FIELDS_24f7b1f5b59c8be7, MAL_DETAIL_RAW_REPR_FIELD)
#endif

#ifndef MAL_DETAIL_REPR_FIELDS_3cf383e0bf0efe5a_DEFINED
#define MAL_DETAIL_REPR_FIELDS_3cf383e0bf0efe5a_DEFINED
#define MAL_DETAIL_REPR_FIELDS_3cf383e0bf0efe5a(field, context) \
field(context, 0, variant_0, MalRepr_Product_24f7b1f5b59c8be7, mal_repr_product_24f7b1f5b59c8be7_t, mal_detail_to_host_24f7b1f5b59c8be7, mal_repr_product_24f7b1f5b59c8be7_return) \
field(context, 1, variant_1, MalType_UInt32, mal_UInt32_t, mal_UInt32_return, mal_UInt32_return)
#endif
#ifndef MAL_DETAIL_RAW_REPR_3cf383e0bf0efe5a_DEFINED
#define MAL_DETAIL_RAW_REPR_3cf383e0bf0efe5a_DEFINED
MAL_DETAIL_DEFINE_SUM_REPR(MalRepr_Sum_3cf383e0bf0efe5a, MAL_DETAIL_REPR_FIELDS_3cf383e0bf0efe5a, MAL_DETAIL_RAW_REPR_FIELD)
#endif

#ifndef MAL_DETAIL_REPR_FIELDS_03dc1b0e40257776_DEFINED
#define MAL_DETAIL_REPR_FIELDS_03dc1b0e40257776_DEFINED
#define MAL_DETAIL_REPR_FIELDS_03dc1b0e40257776(field, context) \
field(context, 0, field_0, MalType_Int32, mal_Int32_t, MAL_DETAIL_REPR_IDENTITY, MAL_DETAIL_REPR_IDENTITY) \
field(context, 1, field_1, MalType_Address, mal_Address_t, MAL_DETAIL_REPR_IDENTITY, mal_Address_return) \
field(context, 2, field_2, MalType_USize, mal_USize_t, MAL_DETAIL_REPR_IDENTITY, MAL_DETAIL_REPR_IDENTITY)
#endif
#ifndef MAL_DETAIL_RAW_REPR_03dc1b0e40257776_DEFINED
#define MAL_DETAIL_RAW_REPR_03dc1b0e40257776_DEFINED
MAL_DETAIL_DEFINE_PRODUCT_REPR(MalRepr_Product_03dc1b0e40257776, MAL_DETAIL_REPR_FIELDS_03dc1b0e40257776, MAL_DETAIL_RAW_REPR_FIELD)
#endif

#ifndef MAL_DETAIL_REPR_FIELDS_4647c725c84fdeda_DEFINED
#define MAL_DETAIL_REPR_FIELDS_4647c725c84fdeda_DEFINED
#define MAL_DETAIL_REPR_FIELDS_4647c725c84fdeda(field, context) \
field(context, 0, variant_0, MalType_Unit, mal_Unit_t, mal_detail_convert_Unit, mal_detail_convert_Unit) \
field(context, 1, variant_1, MalType_UInt32, mal_UInt32_t, mal_UInt32_return, mal_UInt32_return)
#endif
#ifndef MAL_DETAIL_RAW_REPR_4647c725c84fdeda_DEFINED
#define MAL_DETAIL_RAW_REPR_4647c725c84fdeda_DEFINED
MAL_DETAIL_DEFINE_SUM_REPR(MalRepr_Sum_4647c725c84fdeda, MAL_DETAIL_REPR_FIELDS_4647c725c84fdeda, MAL_DETAIL_RAW_REPR_FIELD)
#endif

typedef MalRepr_Product_1e5c20e9f358150e MalType_TransferBuffer;
typedef MalRepr_Product_24f7b1f5b59c8be7 MalType_SourceBytes;
typedef MalRepr_Sum_3cf383e0bf0efe5a MalType_ReadResult;
typedef MalRepr_Sum_4647c725c84fdeda MalType_WriteStatus;

typedef struct { uintptr_t mal_detail_bits; } mal_SourceAllocation_t;
#ifndef MAL_DETAIL_HOST_REPR_1e5c20e9f358150e_DECLARED
#define MAL_DETAIL_HOST_REPR_1e5c20e9f358150e_DECLARED
typedef struct mal_detail_repr_product_1e5c20e9f358150e mal_repr_product_1e5c20e9f358150e_t;
#endif
#ifndef MAL_DETAIL_HOST_REPR_24f7b1f5b59c8be7_DECLARED
#define MAL_DETAIL_HOST_REPR_24f7b1f5b59c8be7_DECLARED
typedef struct mal_detail_repr_product_24f7b1f5b59c8be7 mal_repr_product_24f7b1f5b59c8be7_t;
#endif
#ifndef MAL_DETAIL_HOST_REPR_3cf383e0bf0efe5a_DECLARED
#define MAL_DETAIL_HOST_REPR_3cf383e0bf0efe5a_DECLARED
typedef struct mal_detail_repr_sum_3cf383e0bf0efe5a mal_repr_sum_3cf383e0bf0efe5a_t;
#endif
#ifndef MAL_DETAIL_HOST_REPR_03dc1b0e40257776_DECLARED
#define MAL_DETAIL_HOST_REPR_03dc1b0e40257776_DECLARED
typedef struct mal_detail_repr_product_03dc1b0e40257776 mal_repr_product_03dc1b0e40257776_t;
#endif
#ifndef MAL_DETAIL_HOST_REPR_4647c725c84fdeda_DECLARED
#define MAL_DETAIL_HOST_REPR_4647c725c84fdeda_DECLARED
typedef struct mal_detail_repr_sum_4647c725c84fdeda mal_repr_sum_4647c725c84fdeda_t;
#endif
typedef mal_repr_product_1e5c20e9f358150e_t mal_TransferBuffer_t;
typedef mal_repr_product_24f7b1f5b59c8be7_t mal_SourceBytes_t;
typedef mal_repr_sum_3cf383e0bf0efe5a_t mal_ReadResult_t;
typedef mal_repr_sum_4647c725c84fdeda_t mal_WriteStatus_t;

#ifndef MAL_DETAIL_HOST_REPR_1e5c20e9f358150e_DEFINED
#define MAL_DETAIL_HOST_REPR_1e5c20e9f358150e_DEFINED
MAL_DETAIL_DEFINE_PRODUCT_REPR(mal_detail_repr_product_1e5c20e9f358150e, MAL_DETAIL_REPR_FIELDS_1e5c20e9f358150e, MAL_DETAIL_HOST_REPR_FIELD)
#endif

#ifndef MAL_DETAIL_HOST_REPR_24f7b1f5b59c8be7_DEFINED
#define MAL_DETAIL_HOST_REPR_24f7b1f5b59c8be7_DEFINED
MAL_DETAIL_DEFINE_PRODUCT_REPR(mal_detail_repr_product_24f7b1f5b59c8be7, MAL_DETAIL_REPR_FIELDS_24f7b1f5b59c8be7, MAL_DETAIL_HOST_REPR_FIELD)
#endif

#ifndef MAL_DETAIL_HOST_REPR_3cf383e0bf0efe5a_DEFINED
#define MAL_DETAIL_HOST_REPR_3cf383e0bf0efe5a_DEFINED
MAL_DETAIL_DEFINE_SUM_REPR(mal_detail_repr_sum_3cf383e0bf0efe5a, MAL_DETAIL_REPR_FIELDS_3cf383e0bf0efe5a, MAL_DETAIL_HOST_REPR_FIELD)
#endif

#ifndef MAL_DETAIL_HOST_REPR_03dc1b0e40257776_DEFINED
#define MAL_DETAIL_HOST_REPR_03dc1b0e40257776_DEFINED
MAL_DETAIL_DEFINE_PRODUCT_REPR(mal_detail_repr_product_03dc1b0e40257776, MAL_DETAIL_REPR_FIELDS_03dc1b0e40257776, MAL_DETAIL_HOST_REPR_FIELD)
#endif

#ifndef MAL_DETAIL_HOST_REPR_4647c725c84fdeda_DEFINED
#define MAL_DETAIL_HOST_REPR_4647c725c84fdeda_DEFINED
MAL_DETAIL_DEFINE_SUM_REPR(mal_detail_repr_sum_4647c725c84fdeda, MAL_DETAIL_REPR_FIELDS_4647c725c84fdeda, MAL_DETAIL_HOST_REPR_FIELD)
#endif

/* Type helpers */

static inline mal_SourceAllocation_t mal_detail_to_host_SourceAllocation(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, MalType_SourceAllocation value);
static inline MalType_SourceAllocation mal_SourceAllocation_return(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, mal_SourceAllocation_t value);

#ifndef MAL_DETAIL_HOST_REPR_1e5c20e9f358150e_HELPERS
#define MAL_DETAIL_HOST_REPR_1e5c20e9f358150e_HELPERS
MAL_DETAIL_DEFINE_PRODUCT_CONVERSIONS(mal_detail_to_host_1e5c20e9f358150e, mal_repr_product_1e5c20e9f358150e_return, MalRepr_Product_1e5c20e9f358150e, mal_repr_product_1e5c20e9f358150e_t, MAL_DETAIL_REPR_FIELDS_1e5c20e9f358150e)
#endif

#ifndef MAL_DETAIL_HOST_REPR_24f7b1f5b59c8be7_HELPERS
#define MAL_DETAIL_HOST_REPR_24f7b1f5b59c8be7_HELPERS
MAL_DETAIL_DEFINE_PRODUCT_CONVERSIONS(mal_detail_to_host_24f7b1f5b59c8be7, mal_repr_product_24f7b1f5b59c8be7_return, MalRepr_Product_24f7b1f5b59c8be7, mal_repr_product_24f7b1f5b59c8be7_t, MAL_DETAIL_REPR_FIELDS_24f7b1f5b59c8be7)
#endif

#ifndef MAL_DETAIL_HOST_REPR_3cf383e0bf0efe5a_HELPERS
#define MAL_DETAIL_HOST_REPR_3cf383e0bf0efe5a_HELPERS
MAL_DETAIL_DEFINE_SUM_CONVERSIONS(mal_detail_to_host_3cf383e0bf0efe5a, mal_detail_to_raw_3cf383e0bf0efe5a, MalRepr_Sum_3cf383e0bf0efe5a, mal_repr_sum_3cf383e0bf0efe5a_t, MAL_DETAIL_REPR_FIELDS_3cf383e0bf0efe5a)

#define mal_repr_sum_3cf383e0bf0efe5a_tag_0 UINT32_C(0)
#define mal_repr_sum_3cf383e0bf0efe5a_tag_1 UINT32_C(1)
#define MAL_DETAIL_SUM_API_repr_sum_3cf383e0bf0efe5a(unit, value) \
value(mal_repr_sum_3cf383e0bf0efe5a_make_0, mal_repr_sum_3cf383e0bf0efe5a_return_0, mal_repr_sum_3cf383e0bf0efe5a_t, MalRepr_Sum_3cf383e0bf0efe5a, mal_repr_product_24f7b1f5b59c8be7_t, mal_repr_sum_3cf383e0bf0efe5a_tag_0, variant_0, mal_detail_to_raw_3cf383e0bf0efe5a) \
value(mal_repr_sum_3cf383e0bf0efe5a_make_1, mal_repr_sum_3cf383e0bf0efe5a_return_1, mal_repr_sum_3cf383e0bf0efe5a_t, MalRepr_Sum_3cf383e0bf0efe5a, mal_UInt32_t, mal_repr_sum_3cf383e0bf0efe5a_tag_1, variant_1, mal_detail_to_raw_3cf383e0bf0efe5a)
MAL_DETAIL_SUM_API_repr_sum_3cf383e0bf0efe5a(MAL_DETAIL_DEFINE_SUM_UNIT_API, MAL_DETAIL_DEFINE_SUM_VALUE_API)
#endif

#ifndef MAL_DETAIL_HOST_REPR_03dc1b0e40257776_HELPERS
#define MAL_DETAIL_HOST_REPR_03dc1b0e40257776_HELPERS
MAL_DETAIL_DEFINE_PRODUCT_CONVERSIONS(mal_detail_to_host_03dc1b0e40257776, mal_repr_product_03dc1b0e40257776_return, MalRepr_Product_03dc1b0e40257776, mal_repr_product_03dc1b0e40257776_t, MAL_DETAIL_REPR_FIELDS_03dc1b0e40257776)
#endif

#ifndef MAL_DETAIL_HOST_REPR_4647c725c84fdeda_HELPERS
#define MAL_DETAIL_HOST_REPR_4647c725c84fdeda_HELPERS
MAL_DETAIL_DEFINE_SUM_CONVERSIONS(mal_detail_to_host_4647c725c84fdeda, mal_detail_to_raw_4647c725c84fdeda, MalRepr_Sum_4647c725c84fdeda, mal_repr_sum_4647c725c84fdeda_t, MAL_DETAIL_REPR_FIELDS_4647c725c84fdeda)

#define mal_repr_sum_4647c725c84fdeda_tag_0 UINT32_C(0)
#define mal_repr_sum_4647c725c84fdeda_tag_1 UINT32_C(1)
#define MAL_DETAIL_SUM_API_repr_sum_4647c725c84fdeda(unit, value) \
unit(mal_repr_sum_4647c725c84fdeda_make_0, mal_repr_sum_4647c725c84fdeda_return_0, mal_repr_sum_4647c725c84fdeda_t, MalRepr_Sum_4647c725c84fdeda, mal_repr_sum_4647c725c84fdeda_tag_0, variant_0, mal_detail_to_raw_4647c725c84fdeda) \
value(mal_repr_sum_4647c725c84fdeda_make_1, mal_repr_sum_4647c725c84fdeda_return_1, mal_repr_sum_4647c725c84fdeda_t, MalRepr_Sum_4647c725c84fdeda, mal_UInt32_t, mal_repr_sum_4647c725c84fdeda_tag_1, variant_1, mal_detail_to_raw_4647c725c84fdeda)
MAL_DETAIL_SUM_API_repr_sum_4647c725c84fdeda(MAL_DETAIL_DEFINE_SUM_UNIT_API, MAL_DETAIL_DEFINE_SUM_VALUE_API)
#endif

static inline mal_SourceAllocation_t mal_SourceAllocation_from_bits(uintptr_t bits) {
    return (mal_SourceAllocation_t){ .mal_detail_bits = bits };
}

static inline mal_SourceAllocation_t mal_detail_to_host_SourceAllocation(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, MalType_SourceAllocation value) {
    return mal_SourceAllocation_from_bits(value.bits);
}

static inline uintptr_t mal_SourceAllocation_to_bits(mal_SourceAllocation_t value) {
    return value.mal_detail_bits;
}

static inline MalType_SourceAllocation mal_SourceAllocation_return(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, mal_SourceAllocation_t value) {
    return (MalType_SourceAllocation){ .bits = value.mal_detail_bits };
}

MAL_DETAIL_DEFINE_CONVERTING_RETURN(mal_TransferBuffer_return, MalType_TransferBuffer, mal_TransferBuffer_t, mal_repr_product_1e5c20e9f358150e_return)
MAL_DETAIL_DEFINE_CONVERTING_RETURN(mal_SourceBytes_return, MalType_SourceBytes, mal_SourceBytes_t, mal_repr_product_24f7b1f5b59c8be7_return)
#define mal_ReadResult_tag_0 UINT32_C(0)
#define mal_ReadResult_tag_1 UINT32_C(1)
#define MAL_DETAIL_SUM_API_ReadResult(unit, value) \
value(mal_ReadResult_make_0, mal_ReadResult_return_0, mal_ReadResult_t, MalType_ReadResult, mal_SourceBytes_t, mal_ReadResult_tag_0, variant_0, mal_detail_to_raw_3cf383e0bf0efe5a) \
value(mal_ReadResult_make_1, mal_ReadResult_return_1, mal_ReadResult_t, MalType_ReadResult, mal_UInt32_t, mal_ReadResult_tag_1, variant_1, mal_detail_to_raw_3cf383e0bf0efe5a)
MAL_DETAIL_SUM_API_ReadResult(MAL_DETAIL_DEFINE_SUM_UNIT_API, MAL_DETAIL_DEFINE_SUM_VALUE_API)
#define mal_WriteStatus_tag_0 UINT32_C(0)
#define mal_WriteStatus_tag_1 UINT32_C(1)
#define MAL_DETAIL_SUM_API_WriteStatus(unit, value) \
unit(mal_WriteStatus_make_0, mal_WriteStatus_return_0, mal_WriteStatus_t, MalType_WriteStatus, mal_WriteStatus_tag_0, variant_0, mal_detail_to_raw_4647c725c84fdeda) \
value(mal_WriteStatus_make_1, mal_WriteStatus_return_1, mal_WriteStatus_t, MalType_WriteStatus, mal_UInt32_t, mal_WriteStatus_tag_1, variant_1, mal_detail_to_raw_4647c725c84fdeda)
MAL_DETAIL_SUM_API_WriteStatus(MAL_DETAIL_DEFINE_SUM_UNIT_API, MAL_DETAIL_DEFINE_SUM_VALUE_API)

/* Canonical memory access */

#ifndef MAL_DETAIL_MEMORY_REPR_1e5c20e9f358150e_HELPERS
#define MAL_DETAIL_MEMORY_REPR_1e5c20e9f358150e_HELPERS
#define MAL_DETAIL_MEMORY_FIELDS_1e5c20e9f358150e(unit, value) \
value(field_0, mal_detail_memory_read_Address, mal_detail_memory_write_Address, 0) \
value(field_1, mal_detail_memory_read_USize, mal_detail_memory_write_USize, 8)
MAL_DETAIL_DEFINE_MEMORY_PRODUCT(mal_detail_memory_read_1e5c20e9f358150e, mal_detail_memory_write_1e5c20e9f358150e, mal_repr_product_1e5c20e9f358150e_t, MAL_DETAIL_MEMORY_FIELDS_1e5c20e9f358150e)
#endif

#ifndef MAL_DETAIL_MEMORY_REPR_4647c725c84fdeda_HELPERS
#define MAL_DETAIL_MEMORY_REPR_4647c725c84fdeda_HELPERS
#define MAL_DETAIL_MEMORY_MEMBERS_4647c725c84fdeda(member) \
member(mal_repr_sum_4647c725c84fdeda_t, mal_UInt8_t, mal_detail_memory_write_UInt8, 0, variant_0, mal_detail_memory_read_Unit, mal_detail_memory_write_Unit, 4) \
member(mal_repr_sum_4647c725c84fdeda_t, mal_UInt8_t, mal_detail_memory_write_UInt8, 1, variant_1, mal_detail_memory_read_UInt32, mal_detail_memory_write_UInt32, 4)
MAL_DETAIL_DEFINE_MEMORY_SUM(mal_detail_memory_read_4647c725c84fdeda, mal_detail_memory_write_4647c725c84fdeda, mal_repr_sum_4647c725c84fdeda_t, mal_detail_memory_read_UInt8, MAL_DETAIL_MEMORY_MEMBERS_4647c725c84fdeda)
#endif

MAL_DETAIL_DEFINE_MEMORY_ALIAS(mal_TransferBuffer_read, mal_TransferBuffer_write, mal_TransferBuffer_t, 16, mal_detail_memory_read_1e5c20e9f358150e, mal_detail_memory_write_1e5c20e9f358150e)
MAL_DETAIL_DEFINE_MEMORY_ALIAS(mal_WriteStatus_read, mal_WriteStatus_write, mal_WriteStatus_t, 8, mal_detail_memory_read_4647c725c84fdeda, mal_detail_memory_write_4647c725c84fdeda)

/* External operations */

MalType_TransferBuffer mal_ext_transferBuffer(MalContext *context);
MalType_ReadResult mal_ext_readSource(MalContext *context, MalType_Address argument_0, MalType_USize argument_1);
void mal_ext_releaseSource(MalContext *context, MalType_SourceAllocation value);
MalType_WriteStatus mal_ext_writeChunk(MalContext *context, MalType_Int32 argument_0, MalType_Address argument_1, MalType_USize argument_2);

/* External definition helpers */

#define MAL_HAS_EXTERN_transferBuffer 1
#define MAL_DEFINE_transferBuffer(call) \
static MalType_TransferBuffer mal_detail_transferBuffer(mal_call_t *call); \
MalType_TransferBuffer mal_ext_transferBuffer(MalContext *context MAL_DETAIL_MAYBE_UNUSED) { \
    mal_call_t call = (mal_call_t){ .mal_detail_context = context }; \
    return mal_detail_transferBuffer(&call); \
} \
static MalType_TransferBuffer mal_detail_transferBuffer( \
    mal_call_t *call \
)

#define MAL_HAS_EXTERN_readSource 1
#define MAL_DEFINE_readSource(call, value) \
static MalType_ReadResult mal_detail_readSource(mal_call_t *call, mal_repr_product_1e5c20e9f358150e_t value); \
MalType_ReadResult mal_ext_readSource(MalContext *context MAL_DETAIL_MAYBE_UNUSED, MalType_Address argument_0, MalType_USize argument_1) { \
    mal_call_t call = (mal_call_t){ .mal_detail_context = context }; \
    return mal_detail_readSource(&call, mal_detail_to_host_1e5c20e9f358150e(&call, (MalRepr_Product_1e5c20e9f358150e){ .field_0 = argument_0, .field_1 = argument_1 })); \
} \
static MalType_ReadResult mal_detail_readSource( \
    mal_call_t *call, \
    mal_repr_product_1e5c20e9f358150e_t value \
)

#define MAL_HAS_EXTERN_releaseSource 1
#define MAL_DEFINE_releaseSource(call, value) \
static MalType_Unit mal_detail_releaseSource(mal_call_t *call, mal_SourceAllocation_t value); \
void mal_ext_releaseSource(MalContext *context MAL_DETAIL_MAYBE_UNUSED, MalType_SourceAllocation value) { \
    mal_call_t call = (mal_call_t){ .mal_detail_context = context }; \
    mal_detail_releaseSource(&call, (mal_SourceAllocation_t){ .mal_detail_bits = value.bits }); \
} \
static MalType_Unit mal_detail_releaseSource( \
    mal_call_t *call, \
    mal_SourceAllocation_t value \
)

#define MAL_HAS_EXTERN_writeChunk 1
#define MAL_DEFINE_writeChunk(call, value) \
static MalType_WriteStatus mal_detail_writeChunk(mal_call_t *call, mal_repr_product_03dc1b0e40257776_t value); \
MalType_WriteStatus mal_ext_writeChunk(MalContext *context MAL_DETAIL_MAYBE_UNUSED, MalType_Int32 argument_0, MalType_Address argument_1, MalType_USize argument_2) { \
    mal_call_t call = (mal_call_t){ .mal_detail_context = context }; \
    return mal_detail_writeChunk(&call, mal_detail_to_host_03dc1b0e40257776(&call, (MalRepr_Product_03dc1b0e40257776){ .field_0 = argument_0, .field_1 = argument_1, .field_2 = argument_2 })); \
} \
static MalType_WriteStatus mal_detail_writeChunk( \
    mal_call_t *call, \
    mal_repr_product_03dc1b0e40257776_t value \
)

#endif
#endif
