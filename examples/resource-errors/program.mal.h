#ifndef MAL_BUILD_UMBRELLA
#include <mal.h>

_Static_assert(MAL_C_ABI_VERSION == 0x000a00u, "generated header requires mal C ABI 0x000a00");
_Static_assert(sizeof((size_t)0) == 8, "size_t does not match the mal target index width");
_Static_assert(sizeof((void *)0) == 8, "C pointer size does not match the mal target");
_Static_assert(sizeof(*(MalType_Symbol *)0) == 24, "Symbol carrier size does not match the mal target");
_Static_assert(offsetof(MalType_Symbol, owner) == 0, "Symbol owner offset does not match the mal target");
_Static_assert(offsetof(MalType_Symbol, data) == 8, "Symbol data offset does not match the mal target");
_Static_assert(offsetof(MalType_Symbol, length) == 16, "Symbol length offset does not match the mal target");

#ifndef MAL_GENERATED_INTERFACE_C17CA20CD732541D_H
#define MAL_GENERATED_INTERFACE_C17CA20CD732541D_H

/* Host-visible types */

typedef struct { uintptr_t bits; } MalType_File;

#ifndef MAL_DETAIL_RAW_REPR_f2135c7115a77eb0_DECLARED
#define MAL_DETAIL_RAW_REPR_f2135c7115a77eb0_DECLARED
typedef struct MalRepr_Sum_f2135c7115a77eb0 MalRepr_Sum_f2135c7115a77eb0;
#endif
#ifndef MAL_DETAIL_RAW_REPR_9d9d35c5fd9628bc_DECLARED
#define MAL_DETAIL_RAW_REPR_9d9d35c5fd9628bc_DECLARED
typedef struct MalRepr_Sum_9d9d35c5fd9628bc MalRepr_Sum_9d9d35c5fd9628bc;
#endif
#ifndef MAL_DETAIL_RAW_REPR_4647c725c84fdeda_DECLARED
#define MAL_DETAIL_RAW_REPR_4647c725c84fdeda_DECLARED
typedef struct MalRepr_Sum_4647c725c84fdeda MalRepr_Sum_4647c725c84fdeda;
#endif

#ifndef MAL_DETAIL_REPR_FIELDS_f2135c7115a77eb0_DEFINED
#define MAL_DETAIL_REPR_FIELDS_f2135c7115a77eb0_DEFINED
#define MAL_DETAIL_REPR_FIELDS_f2135c7115a77eb0(field, context) \
field(context, 0, variant_0, MalType_File, mal_File_t, mal_detail_to_host_File, mal_File_return) \
field(context, 1, variant_1, MalType_UInt32, mal_UInt32_t, mal_UInt32_return, mal_UInt32_return)
#endif
#ifndef MAL_DETAIL_RAW_REPR_f2135c7115a77eb0_DEFINED
#define MAL_DETAIL_RAW_REPR_f2135c7115a77eb0_DEFINED
MAL_DETAIL_DEFINE_SUM_REPR(MalRepr_Sum_f2135c7115a77eb0, MAL_DETAIL_REPR_FIELDS_f2135c7115a77eb0, MAL_DETAIL_RAW_REPR_FIELD)
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

typedef MalRepr_Sum_f2135c7115a77eb0 MalType_OpenResult;
typedef MalRepr_Sum_9d9d35c5fd9628bc MalType_ReadResult;
typedef MalRepr_Sum_4647c725c84fdeda MalType_CloseResult;

typedef struct { uintptr_t mal_detail_bits; } mal_File_t;
#ifndef MAL_DETAIL_HOST_REPR_f2135c7115a77eb0_DECLARED
#define MAL_DETAIL_HOST_REPR_f2135c7115a77eb0_DECLARED
typedef struct mal_detail_repr_sum_f2135c7115a77eb0 mal_repr_sum_f2135c7115a77eb0_t;
#endif
#ifndef MAL_DETAIL_HOST_REPR_9d9d35c5fd9628bc_DECLARED
#define MAL_DETAIL_HOST_REPR_9d9d35c5fd9628bc_DECLARED
typedef struct mal_detail_repr_sum_9d9d35c5fd9628bc mal_repr_sum_9d9d35c5fd9628bc_t;
#endif
#ifndef MAL_DETAIL_HOST_REPR_4647c725c84fdeda_DECLARED
#define MAL_DETAIL_HOST_REPR_4647c725c84fdeda_DECLARED
typedef struct mal_detail_repr_sum_4647c725c84fdeda mal_repr_sum_4647c725c84fdeda_t;
#endif
typedef mal_repr_sum_f2135c7115a77eb0_t mal_OpenResult_t;
typedef mal_repr_sum_9d9d35c5fd9628bc_t mal_ReadResult_t;
typedef mal_repr_sum_4647c725c84fdeda_t mal_CloseResult_t;

#ifndef MAL_DETAIL_HOST_REPR_f2135c7115a77eb0_DEFINED
#define MAL_DETAIL_HOST_REPR_f2135c7115a77eb0_DEFINED
MAL_DETAIL_DEFINE_SUM_REPR(mal_detail_repr_sum_f2135c7115a77eb0, MAL_DETAIL_REPR_FIELDS_f2135c7115a77eb0, MAL_DETAIL_HOST_REPR_FIELD)
#endif

#ifndef MAL_DETAIL_HOST_REPR_9d9d35c5fd9628bc_DEFINED
#define MAL_DETAIL_HOST_REPR_9d9d35c5fd9628bc_DEFINED
MAL_DETAIL_DEFINE_SUM_REPR(mal_detail_repr_sum_9d9d35c5fd9628bc, MAL_DETAIL_REPR_FIELDS_9d9d35c5fd9628bc, MAL_DETAIL_HOST_REPR_FIELD)
#endif

#ifndef MAL_DETAIL_HOST_REPR_4647c725c84fdeda_DEFINED
#define MAL_DETAIL_HOST_REPR_4647c725c84fdeda_DEFINED
MAL_DETAIL_DEFINE_SUM_REPR(mal_detail_repr_sum_4647c725c84fdeda, MAL_DETAIL_REPR_FIELDS_4647c725c84fdeda, MAL_DETAIL_HOST_REPR_FIELD)
#endif

/* Type helpers */

static inline mal_File_t mal_detail_to_host_File(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, MalType_File value);
static inline MalType_File mal_File_return(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, mal_File_t value);

#ifndef MAL_DETAIL_HOST_REPR_f2135c7115a77eb0_HELPERS
#define MAL_DETAIL_HOST_REPR_f2135c7115a77eb0_HELPERS
MAL_DETAIL_DEFINE_SUM_CONVERSIONS(mal_detail_to_host_f2135c7115a77eb0, mal_detail_to_raw_f2135c7115a77eb0, MalRepr_Sum_f2135c7115a77eb0, mal_repr_sum_f2135c7115a77eb0_t, MAL_DETAIL_REPR_FIELDS_f2135c7115a77eb0)

#define mal_repr_sum_f2135c7115a77eb0_tag_0 UINT32_C(0)
#define mal_repr_sum_f2135c7115a77eb0_tag_1 UINT32_C(1)
#define MAL_DETAIL_SUM_API_repr_sum_f2135c7115a77eb0(unit, value) \
value(mal_repr_sum_f2135c7115a77eb0_make_0, mal_repr_sum_f2135c7115a77eb0_return_0, mal_repr_sum_f2135c7115a77eb0_t, MalRepr_Sum_f2135c7115a77eb0, mal_File_t, mal_repr_sum_f2135c7115a77eb0_tag_0, variant_0, mal_detail_to_raw_f2135c7115a77eb0) \
value(mal_repr_sum_f2135c7115a77eb0_make_1, mal_repr_sum_f2135c7115a77eb0_return_1, mal_repr_sum_f2135c7115a77eb0_t, MalRepr_Sum_f2135c7115a77eb0, mal_UInt32_t, mal_repr_sum_f2135c7115a77eb0_tag_1, variant_1, mal_detail_to_raw_f2135c7115a77eb0)
MAL_DETAIL_SUM_API_repr_sum_f2135c7115a77eb0(MAL_DETAIL_DEFINE_SUM_UNIT_API, MAL_DETAIL_DEFINE_SUM_VALUE_API)
#endif

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

#define mal_OpenResult_tag_0 UINT32_C(0)
#define mal_OpenResult_tag_1 UINT32_C(1)
#define MAL_DETAIL_SUM_API_OpenResult(unit, value) \
value(mal_OpenResult_make_0, mal_OpenResult_return_0, mal_OpenResult_t, MalType_OpenResult, mal_File_t, mal_OpenResult_tag_0, variant_0, mal_detail_to_raw_f2135c7115a77eb0) \
value(mal_OpenResult_make_1, mal_OpenResult_return_1, mal_OpenResult_t, MalType_OpenResult, mal_UInt32_t, mal_OpenResult_tag_1, variant_1, mal_detail_to_raw_f2135c7115a77eb0)
MAL_DETAIL_SUM_API_OpenResult(MAL_DETAIL_DEFINE_SUM_UNIT_API, MAL_DETAIL_DEFINE_SUM_VALUE_API)
#define mal_ReadResult_tag_0 UINT32_C(0)
#define mal_ReadResult_tag_1 UINT32_C(1)
#define MAL_DETAIL_SUM_API_ReadResult(unit, value) \
value(mal_ReadResult_make_0, mal_ReadResult_return_0, mal_ReadResult_t, MalType_ReadResult, mal_Buffer_t, mal_ReadResult_tag_0, variant_0, mal_detail_to_raw_9d9d35c5fd9628bc) \
value(mal_ReadResult_make_1, mal_ReadResult_return_1, mal_ReadResult_t, MalType_ReadResult, mal_UInt32_t, mal_ReadResult_tag_1, variant_1, mal_detail_to_raw_9d9d35c5fd9628bc)
MAL_DETAIL_SUM_API_ReadResult(MAL_DETAIL_DEFINE_SUM_UNIT_API, MAL_DETAIL_DEFINE_SUM_VALUE_API)
#define mal_CloseResult_tag_0 UINT32_C(0)
#define mal_CloseResult_tag_1 UINT32_C(1)
#define MAL_DETAIL_SUM_API_CloseResult(unit, value) \
unit(mal_CloseResult_make_0, mal_CloseResult_return_0, mal_CloseResult_t, MalType_CloseResult, mal_CloseResult_tag_0, variant_0, mal_detail_to_raw_4647c725c84fdeda) \
value(mal_CloseResult_make_1, mal_CloseResult_return_1, mal_CloseResult_t, MalType_CloseResult, mal_UInt32_t, mal_CloseResult_tag_1, variant_1, mal_detail_to_raw_4647c725c84fdeda)
MAL_DETAIL_SUM_API_CloseResult(MAL_DETAIL_DEFINE_SUM_UNIT_API, MAL_DETAIL_DEFINE_SUM_VALUE_API)

/* External operations */

MalType_OpenResult mal_ext_openReadOnly(MalContext *context, MalType_Symbol value);
MalType_ReadResult mal_ext_readFile(MalContext *context, MalType_File value);
MalType_CloseResult mal_ext_closeFile(MalContext *context, MalType_File value);
void mal_ext_writeBytes(MalContext *context, MalType_Symbol value);
void mal_ext_writeError(MalContext *context, MalType_UInt32 value);

/* External definition helpers */

#define MAL_HAS_EXTERN_openReadOnly 1
#define MAL_DEFINE_openReadOnly(call, value) \
static MalType_OpenResult mal_detail_openReadOnly(mal_call_t *call, mal_Symbol_t value); \
MalType_OpenResult mal_ext_openReadOnly(MalContext *context MAL_DETAIL_MAYBE_UNUSED, MalType_Symbol value) { \
    mal_call_t call = (mal_call_t){ .mal_detail_context = context }; \
    return mal_detail_openReadOnly(&call, value); \
} \
static MalType_OpenResult mal_detail_openReadOnly( \
    mal_call_t *call, \
    mal_Symbol_t value \
)

#define MAL_HAS_EXTERN_readFile 1
#define MAL_DEFINE_readFile(call, value) \
static MalType_ReadResult mal_detail_readFile(mal_call_t *call, mal_File_t value); \
MalType_ReadResult mal_ext_readFile(MalContext *context MAL_DETAIL_MAYBE_UNUSED, MalType_File value) { \
    mal_call_t call = (mal_call_t){ .mal_detail_context = context }; \
    return mal_detail_readFile(&call, (mal_File_t){ .mal_detail_bits = value.bits }); \
} \
static MalType_ReadResult mal_detail_readFile( \
    mal_call_t *call, \
    mal_File_t value \
)

#define MAL_HAS_EXTERN_closeFile 1
#define MAL_DEFINE_closeFile(call, value) \
static MalType_CloseResult mal_detail_closeFile(mal_call_t *call, mal_File_t value); \
MalType_CloseResult mal_ext_closeFile(MalContext *context MAL_DETAIL_MAYBE_UNUSED, MalType_File value) { \
    mal_call_t call = (mal_call_t){ .mal_detail_context = context }; \
    return mal_detail_closeFile(&call, (mal_File_t){ .mal_detail_bits = value.bits }); \
} \
static MalType_CloseResult mal_detail_closeFile( \
    mal_call_t *call, \
    mal_File_t value \
)

#define MAL_HAS_EXTERN_writeBytes 1
#define MAL_DEFINE_writeBytes(call, value) \
static MalType_Unit mal_detail_writeBytes(mal_call_t *call, mal_Symbol_t value); \
void mal_ext_writeBytes(MalContext *context MAL_DETAIL_MAYBE_UNUSED, MalType_Symbol value) { \
    mal_call_t call = (mal_call_t){ .mal_detail_context = context }; \
    mal_detail_writeBytes(&call, value); \
} \
static MalType_Unit mal_detail_writeBytes( \
    mal_call_t *call, \
    mal_Symbol_t value \
)

#define MAL_HAS_EXTERN_writeError 1
#define MAL_DEFINE_writeError(call, value) \
static MalType_Unit mal_detail_writeError(mal_call_t *call, mal_UInt32_t value); \
void mal_ext_writeError(MalContext *context MAL_DETAIL_MAYBE_UNUSED, MalType_UInt32 value) { \
    mal_call_t call = (mal_call_t){ .mal_detail_context = context }; \
    mal_detail_writeError(&call, value); \
} \
static MalType_Unit mal_detail_writeError( \
    mal_call_t *call, \
    mal_UInt32_t value \
)

#endif
#endif
