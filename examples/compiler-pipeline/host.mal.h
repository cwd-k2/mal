#ifndef MAL_BUILD_UMBRELLA
#include <mal.h>

_Static_assert(MAL_C_ABI_VERSION == 0x000a00u, "generated header requires mal C ABI 0x000a00");
_Static_assert(sizeof((size_t)0) == 8, "size_t does not match the mal target index width");
_Static_assert(sizeof((void *)0) == 8, "C pointer size does not match the mal target");
_Static_assert(sizeof(*(MalType_Symbol *)0) == 24, "Symbol carrier size does not match the mal target");
_Static_assert(offsetof(MalType_Symbol, owner) == 0, "Symbol owner offset does not match the mal target");
_Static_assert(offsetof(MalType_Symbol, data) == 8, "Symbol data offset does not match the mal target");
_Static_assert(offsetof(MalType_Symbol, length) == 16, "Symbol length offset does not match the mal target");

#ifndef MAL_GENERATED_INTERFACE_74AE8C1E3F9CF7FB_H
#define MAL_GENERATED_INTERFACE_74AE8C1E3F9CF7FB_H

/* Host-visible types */

#ifndef MAL_DETAIL_RAW_REPR_9d9d35c5fd9628bc_DECLARED
#define MAL_DETAIL_RAW_REPR_9d9d35c5fd9628bc_DECLARED
typedef struct MalRepr_Sum_9d9d35c5fd9628bc MalRepr_Sum_9d9d35c5fd9628bc;
#endif
#ifndef MAL_DETAIL_RAW_REPR_1e40e2e9f340e3fc_DECLARED
#define MAL_DETAIL_RAW_REPR_1e40e2e9f340e3fc_DECLARED
typedef struct MalRepr_Product_1e40e2e9f340e3fc MalRepr_Product_1e40e2e9f340e3fc;
#endif
#ifndef MAL_DETAIL_RAW_REPR_4647c725c84fdeda_DECLARED
#define MAL_DETAIL_RAW_REPR_4647c725c84fdeda_DECLARED
typedef struct MalRepr_Sum_4647c725c84fdeda MalRepr_Sum_4647c725c84fdeda;
#endif

#ifndef MAL_DETAIL_REPR_FIELDS_9d9d35c5fd9628bc_DEFINED
#define MAL_DETAIL_REPR_FIELDS_9d9d35c5fd9628bc_DEFINED
#define MAL_DETAIL_REPR_FIELDS_9d9d35c5fd9628bc(field, context) \
field(context, 0, variant_0, MalType_Buffer, mal_Buffer_t, MAL_DETAIL_REPR_IDENTITY, mal_Buffer_return_move) \
field(context, 1, variant_1, MalType_UInt32, mal_UInt32_t, mal_UInt32_return, mal_UInt32_return)
#endif
#ifndef MAL_DETAIL_RAW_REPR_9d9d35c5fd9628bc_DEFINED
#define MAL_DETAIL_RAW_REPR_9d9d35c5fd9628bc_DEFINED
MAL_DETAIL_DEFINE_SUM_REPR(MalRepr_Sum_9d9d35c5fd9628bc, MAL_DETAIL_REPR_FIELDS_9d9d35c5fd9628bc, MAL_DETAIL_RAW_REPR_FIELD)
#endif

#ifndef MAL_DETAIL_REPR_FIELDS_1e40e2e9f340e3fc_DEFINED
#define MAL_DETAIL_REPR_FIELDS_1e40e2e9f340e3fc_DEFINED
#define MAL_DETAIL_REPR_FIELDS_1e40e2e9f340e3fc(field, context) \
field(context, 0, field_0, MalType_Int32, mal_Int32_t, MAL_DETAIL_REPR_IDENTITY, MAL_DETAIL_REPR_IDENTITY) \
field(context, 1, field_1, MalType_Symbol, mal_Symbol_t, MAL_DETAIL_REPR_IDENTITY, mal_Symbol_return_move)
#endif
#ifndef MAL_DETAIL_RAW_REPR_1e40e2e9f340e3fc_DEFINED
#define MAL_DETAIL_RAW_REPR_1e40e2e9f340e3fc_DEFINED
MAL_DETAIL_DEFINE_PRODUCT_REPR(MalRepr_Product_1e40e2e9f340e3fc, MAL_DETAIL_REPR_FIELDS_1e40e2e9f340e3fc, MAL_DETAIL_RAW_REPR_FIELD)
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

typedef MalRepr_Sum_9d9d35c5fd9628bc MalType_ReadResult;
typedef MalRepr_Sum_4647c725c84fdeda MalType_WriteStatus;

#ifndef MAL_DETAIL_HOST_REPR_9d9d35c5fd9628bc_DECLARED
#define MAL_DETAIL_HOST_REPR_9d9d35c5fd9628bc_DECLARED
typedef struct mal_detail_repr_sum_9d9d35c5fd9628bc mal_repr_sum_9d9d35c5fd9628bc_t;
#endif
#ifndef MAL_DETAIL_HOST_REPR_1e40e2e9f340e3fc_DECLARED
#define MAL_DETAIL_HOST_REPR_1e40e2e9f340e3fc_DECLARED
typedef struct mal_detail_repr_product_1e40e2e9f340e3fc mal_repr_product_1e40e2e9f340e3fc_t;
#endif
#ifndef MAL_DETAIL_HOST_REPR_4647c725c84fdeda_DECLARED
#define MAL_DETAIL_HOST_REPR_4647c725c84fdeda_DECLARED
typedef struct mal_detail_repr_sum_4647c725c84fdeda mal_repr_sum_4647c725c84fdeda_t;
#endif
typedef mal_repr_sum_9d9d35c5fd9628bc_t mal_ReadResult_t;
typedef mal_repr_sum_4647c725c84fdeda_t mal_WriteStatus_t;

#ifndef MAL_DETAIL_HOST_REPR_9d9d35c5fd9628bc_DEFINED
#define MAL_DETAIL_HOST_REPR_9d9d35c5fd9628bc_DEFINED
MAL_DETAIL_DEFINE_SUM_REPR(mal_detail_repr_sum_9d9d35c5fd9628bc, MAL_DETAIL_REPR_FIELDS_9d9d35c5fd9628bc, MAL_DETAIL_HOST_REPR_FIELD)
#endif

#ifndef MAL_DETAIL_HOST_REPR_1e40e2e9f340e3fc_DEFINED
#define MAL_DETAIL_HOST_REPR_1e40e2e9f340e3fc_DEFINED
MAL_DETAIL_DEFINE_PRODUCT_REPR(mal_detail_repr_product_1e40e2e9f340e3fc, MAL_DETAIL_REPR_FIELDS_1e40e2e9f340e3fc, MAL_DETAIL_HOST_REPR_FIELD)
#endif

#ifndef MAL_DETAIL_HOST_REPR_4647c725c84fdeda_DEFINED
#define MAL_DETAIL_HOST_REPR_4647c725c84fdeda_DEFINED
MAL_DETAIL_DEFINE_SUM_REPR(mal_detail_repr_sum_4647c725c84fdeda, MAL_DETAIL_REPR_FIELDS_4647c725c84fdeda, MAL_DETAIL_HOST_REPR_FIELD)
#endif

/* Type helpers */

#ifndef MAL_DETAIL_HOST_REPR_9d9d35c5fd9628bc_HELPERS
#define MAL_DETAIL_HOST_REPR_9d9d35c5fd9628bc_HELPERS
MAL_DETAIL_DEFINE_SUM_CONVERSIONS(mal_detail_to_host_9d9d35c5fd9628bc, mal_detail_to_raw_9d9d35c5fd9628bc, MalRepr_Sum_9d9d35c5fd9628bc, mal_repr_sum_9d9d35c5fd9628bc_t, MAL_DETAIL_REPR_FIELDS_9d9d35c5fd9628bc)

#define mal_repr_sum_9d9d35c5fd9628bc_tag_0 UINT32_C(0)
#define mal_repr_sum_9d9d35c5fd9628bc_tag_1 UINT32_C(1)
#define MAL_DETAIL_SUM_API_repr_sum_9d9d35c5fd9628bc(unit, value) \
value(mal_repr_sum_9d9d35c5fd9628bc_make_0, mal_repr_sum_9d9d35c5fd9628bc_return_0, mal_repr_sum_9d9d35c5fd9628bc_t, MalRepr_Sum_9d9d35c5fd9628bc, mal_Buffer_t, mal_repr_sum_9d9d35c5fd9628bc_tag_0, variant_0, mal_detail_to_raw_9d9d35c5fd9628bc) \
value(mal_repr_sum_9d9d35c5fd9628bc_make_1, mal_repr_sum_9d9d35c5fd9628bc_return_1, mal_repr_sum_9d9d35c5fd9628bc_t, MalRepr_Sum_9d9d35c5fd9628bc, mal_UInt32_t, mal_repr_sum_9d9d35c5fd9628bc_tag_1, variant_1, mal_detail_to_raw_9d9d35c5fd9628bc)
MAL_DETAIL_SUM_API_repr_sum_9d9d35c5fd9628bc(MAL_DETAIL_DEFINE_SUM_UNIT_API, MAL_DETAIL_DEFINE_SUM_VALUE_API)
#endif

#ifndef MAL_DETAIL_HOST_REPR_1e40e2e9f340e3fc_HELPERS
#define MAL_DETAIL_HOST_REPR_1e40e2e9f340e3fc_HELPERS
MAL_DETAIL_DEFINE_PRODUCT_CONVERSIONS(mal_detail_to_host_1e40e2e9f340e3fc, mal_repr_product_1e40e2e9f340e3fc_return, MalRepr_Product_1e40e2e9f340e3fc, mal_repr_product_1e40e2e9f340e3fc_t, MAL_DETAIL_REPR_FIELDS_1e40e2e9f340e3fc)
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

#define mal_ReadResult_tag_0 UINT32_C(0)
#define mal_ReadResult_tag_1 UINT32_C(1)
#define MAL_DETAIL_SUM_API_ReadResult(unit, value) \
value(mal_ReadResult_make_0, mal_ReadResult_return_0, mal_ReadResult_t, MalType_ReadResult, mal_Buffer_t, mal_ReadResult_tag_0, variant_0, mal_detail_to_raw_9d9d35c5fd9628bc) \
value(mal_ReadResult_make_1, mal_ReadResult_return_1, mal_ReadResult_t, MalType_ReadResult, mal_UInt32_t, mal_ReadResult_tag_1, variant_1, mal_detail_to_raw_9d9d35c5fd9628bc)
MAL_DETAIL_SUM_API_ReadResult(MAL_DETAIL_DEFINE_SUM_UNIT_API, MAL_DETAIL_DEFINE_SUM_VALUE_API)
#define mal_WriteStatus_tag_0 UINT32_C(0)
#define mal_WriteStatus_tag_1 UINT32_C(1)
#define MAL_DETAIL_SUM_API_WriteStatus(unit, value) \
unit(mal_WriteStatus_make_0, mal_WriteStatus_return_0, mal_WriteStatus_t, MalType_WriteStatus, mal_WriteStatus_tag_0, variant_0, mal_detail_to_raw_4647c725c84fdeda) \
value(mal_WriteStatus_make_1, mal_WriteStatus_return_1, mal_WriteStatus_t, MalType_WriteStatus, mal_UInt32_t, mal_WriteStatus_tag_1, variant_1, mal_detail_to_raw_4647c725c84fdeda)
MAL_DETAIL_SUM_API_WriteStatus(MAL_DETAIL_DEFINE_SUM_UNIT_API, MAL_DETAIL_DEFINE_SUM_VALUE_API)

/* External operations */

MalType_ReadResult mal_ext_readSource(MalContext *context, MalType_Symbol value);
MalType_WriteStatus mal_ext_writeChunk(MalContext *context, MalType_Int32 argument_0, MalType_Symbol argument_1);

/* External definition helpers */

#define MAL_HAS_EXTERN_readSource 1
#define MAL_DEFINE_readSource(call, value) \
static MalType_ReadResult mal_detail_readSource(mal_call_t *call, mal_Symbol_t value); \
MalType_ReadResult mal_ext_readSource(MalContext *context MAL_DETAIL_MAYBE_UNUSED, MalType_Symbol value) { \
    mal_call_t call = (mal_call_t){ .mal_detail_context = context }; \
    return mal_detail_readSource(&call, value); \
} \
static MalType_ReadResult mal_detail_readSource( \
    mal_call_t *call, \
    mal_Symbol_t value \
)

#define MAL_HAS_EXTERN_writeChunk 1
#define MAL_DEFINE_writeChunk(call, value) \
static MalType_WriteStatus mal_detail_writeChunk(mal_call_t *call, mal_repr_product_1e40e2e9f340e3fc_t value); \
MalType_WriteStatus mal_ext_writeChunk(MalContext *context MAL_DETAIL_MAYBE_UNUSED, MalType_Int32 argument_0, MalType_Symbol argument_1) { \
    mal_call_t call = (mal_call_t){ .mal_detail_context = context }; \
    return mal_detail_writeChunk(&call, mal_detail_to_host_1e40e2e9f340e3fc(&call, (MalRepr_Product_1e40e2e9f340e3fc){ .field_0 = argument_0, .field_1 = argument_1 })); \
} \
static MalType_WriteStatus mal_detail_writeChunk( \
    mal_call_t *call, \
    mal_repr_product_1e40e2e9f340e3fc_t value \
)

#endif
#endif
