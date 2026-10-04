#ifndef MAL_BUILD_UMBRELLA
#include <mal.h>

_Static_assert(MAL_C_ABI_VERSION == 0x000a00u, "generated header requires mal C ABI 0x000a00");
_Static_assert(sizeof((size_t)0) == 8, "size_t does not match the mal target index width");
_Static_assert(sizeof((void *)0) == 8, "C pointer size does not match the mal target");
_Static_assert(sizeof(*(MalType_Symbol *)0) == 24, "Symbol carrier size does not match the mal target");
_Static_assert(offsetof(MalType_Symbol, owner) == 0, "Symbol owner offset does not match the mal target");
_Static_assert(offsetof(MalType_Symbol, data) == 8, "Symbol data offset does not match the mal target");
_Static_assert(offsetof(MalType_Symbol, length) == 16, "Symbol length offset does not match the mal target");

#ifndef MAL_GENERATED_INTERFACE_CA7721AC16BD928E_H
#define MAL_GENERATED_INTERFACE_CA7721AC16BD928E_H

/* Host-visible types */

#ifndef MAL_DETAIL_RAW_REPR_46708725c872772e_DECLARED
#define MAL_DETAIL_RAW_REPR_46708725c872772e_DECLARED
typedef struct MalRepr_Sum_46708725c872772e MalRepr_Sum_46708725c872772e;
#endif
#ifndef MAL_DETAIL_RAW_REPR_1e40e2e9f340e3fc_DECLARED
#define MAL_DETAIL_RAW_REPR_1e40e2e9f340e3fc_DECLARED
typedef struct MalRepr_Product_1e40e2e9f340e3fc MalRepr_Product_1e40e2e9f340e3fc;
#endif
#ifndef MAL_DETAIL_RAW_REPR_4647c725c84fdeda_DECLARED
#define MAL_DETAIL_RAW_REPR_4647c725c84fdeda_DECLARED
typedef struct MalRepr_Sum_4647c725c84fdeda MalRepr_Sum_4647c725c84fdeda;
#endif

#ifndef MAL_DETAIL_REPR_FIELDS_46708725c872772e_DEFINED
#define MAL_DETAIL_REPR_FIELDS_46708725c872772e_DEFINED
#define MAL_DETAIL_REPR_FIELDS_46708725c872772e(field, context) \
field(context, 0, variant_0, MalType_Buffer, mal_Buffer_t, MAL_DETAIL_REPR_IDENTITY, mal_detail_to_raw_Buffer) \
field(context, 1, variant_1, MalType_UInt32, mal_UInt32_t, mal_detail_to_raw_UInt32, mal_detail_to_raw_UInt32)
#endif
#ifndef MAL_DETAIL_RAW_REPR_46708725c872772e_DEFINED
#define MAL_DETAIL_RAW_REPR_46708725c872772e_DEFINED
MAL_DETAIL_DEFINE_SUM_REPR(MalRepr_Sum_46708725c872772e, MAL_DETAIL_REPR_FIELDS_46708725c872772e, MAL_DETAIL_RAW_REPR_FIELD)
#endif

#ifndef MAL_DETAIL_REPR_FIELDS_1e40e2e9f340e3fc_DEFINED
#define MAL_DETAIL_REPR_FIELDS_1e40e2e9f340e3fc_DEFINED
#define MAL_DETAIL_REPR_FIELDS_1e40e2e9f340e3fc(field, context) \
field(context, 0, field_0, MalType_Int32, mal_Int32_t, MAL_DETAIL_REPR_IDENTITY, MAL_DETAIL_REPR_IDENTITY) \
field(context, 1, field_1, MalType_Symbol, mal_Symbol_t, MAL_DETAIL_REPR_IDENTITY, mal_detail_to_raw_Symbol)
#endif
#ifndef MAL_DETAIL_RAW_REPR_1e40e2e9f340e3fc_DEFINED
#define MAL_DETAIL_RAW_REPR_1e40e2e9f340e3fc_DEFINED
MAL_DETAIL_DEFINE_PRODUCT_REPR(MalRepr_Product_1e40e2e9f340e3fc, MAL_DETAIL_REPR_FIELDS_1e40e2e9f340e3fc, MAL_DETAIL_RAW_REPR_FIELD)
#endif

#ifndef MAL_DETAIL_REPR_FIELDS_4647c725c84fdeda_DEFINED
#define MAL_DETAIL_REPR_FIELDS_4647c725c84fdeda_DEFINED
#define MAL_DETAIL_REPR_FIELDS_4647c725c84fdeda(field, context) \
field(context, 0, variant_0, MalType_Unit, mal_Unit_t, mal_detail_to_raw_Unit, mal_detail_to_raw_Unit) \
field(context, 1, variant_1, MalType_UInt32, mal_UInt32_t, mal_detail_to_raw_UInt32, mal_detail_to_raw_UInt32)
#endif
#ifndef MAL_DETAIL_RAW_REPR_4647c725c84fdeda_DEFINED
#define MAL_DETAIL_RAW_REPR_4647c725c84fdeda_DEFINED
MAL_DETAIL_DEFINE_SUM_REPR(MalRepr_Sum_4647c725c84fdeda, MAL_DETAIL_REPR_FIELDS_4647c725c84fdeda, MAL_DETAIL_RAW_REPR_FIELD)
#endif

typedef MalRepr_Sum_46708725c872772e MalType_ReadResult;
typedef MalRepr_Sum_4647c725c84fdeda MalType_WriteStatus;

#ifndef MAL_DETAIL_HOST_REPR_46708725c872772e_DECLARED
#define MAL_DETAIL_HOST_REPR_46708725c872772e_DECLARED
typedef struct mal_detail_repr_sum_46708725c872772e mal_repr_sum_46708725c872772e_t;
#endif
#ifndef MAL_DETAIL_HOST_REPR_1e40e2e9f340e3fc_DECLARED
#define MAL_DETAIL_HOST_REPR_1e40e2e9f340e3fc_DECLARED
typedef struct mal_detail_repr_product_1e40e2e9f340e3fc mal_repr_product_1e40e2e9f340e3fc_t;
#endif
#ifndef MAL_DETAIL_HOST_REPR_4647c725c84fdeda_DECLARED
#define MAL_DETAIL_HOST_REPR_4647c725c84fdeda_DECLARED
typedef struct mal_detail_repr_sum_4647c725c84fdeda mal_repr_sum_4647c725c84fdeda_t;
#endif
typedef mal_repr_sum_46708725c872772e_t mal_ReadResult_t;
typedef mal_repr_sum_4647c725c84fdeda_t mal_WriteStatus_t;

#ifndef MAL_DETAIL_HOST_REPR_46708725c872772e_DEFINED
#define MAL_DETAIL_HOST_REPR_46708725c872772e_DEFINED
MAL_DETAIL_DEFINE_SUM_REPR(mal_detail_repr_sum_46708725c872772e, MAL_DETAIL_REPR_FIELDS_46708725c872772e, MAL_DETAIL_HOST_REPR_FIELD)
typedef void (*mal_detail_sum_key_46708725c872772e_t)(mal_Buffer_t, mal_UInt32_t);
__attribute__((overloadable)) mal_repr_sum_46708725c872772e_t *mal_detail_sum_type(mal_detail_sum_key_46708725c872772e_t);
#endif

#ifndef MAL_DETAIL_HOST_REPR_1e40e2e9f340e3fc_DEFINED
#define MAL_DETAIL_HOST_REPR_1e40e2e9f340e3fc_DEFINED
MAL_DETAIL_DEFINE_PRODUCT_REPR(mal_detail_repr_product_1e40e2e9f340e3fc, MAL_DETAIL_REPR_FIELDS_1e40e2e9f340e3fc, MAL_DETAIL_HOST_REPR_FIELD)
typedef void (*mal_detail_product_key_1e40e2e9f340e3fc_t)(mal_Int32_t, mal_Symbol_t);
__attribute__((overloadable)) mal_repr_product_1e40e2e9f340e3fc_t *mal_detail_product_type(mal_detail_product_key_1e40e2e9f340e3fc_t);
#endif

#ifndef MAL_DETAIL_HOST_REPR_4647c725c84fdeda_DEFINED
#define MAL_DETAIL_HOST_REPR_4647c725c84fdeda_DEFINED
MAL_DETAIL_DEFINE_SUM_REPR(mal_detail_repr_sum_4647c725c84fdeda, MAL_DETAIL_REPR_FIELDS_4647c725c84fdeda, MAL_DETAIL_HOST_REPR_FIELD)
typedef void (*mal_detail_sum_key_4647c725c84fdeda_t)(mal_Unit_t, mal_UInt32_t);
__attribute__((overloadable)) mal_repr_sum_4647c725c84fdeda_t *mal_detail_sum_type(mal_detail_sum_key_4647c725c84fdeda_t);
#endif

/* Type helpers */

#ifndef MAL_DETAIL_HOST_REPR_46708725c872772e_LIFECYCLE
#define MAL_DETAIL_HOST_REPR_46708725c872772e_LIFECYCLE
static inline __attribute__((overloadable)) void mal_detail_retain(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, mal_repr_sum_46708725c872772e_t *value MAL_DETAIL_MAYBE_UNUSED) {
    switch (value->tag) {
        case UINT32_C(0): {
            mal_detail_retain(call, &value->payload.variant_0);
            return;
        }
        case UINT32_C(1): {
            mal_detail_retain(call, &value->payload.variant_1);
            return;
        }
        default: {
            return;
        }
    }
}
static inline __attribute__((overloadable)) void mal_detail_release(mal_repr_sum_46708725c872772e_t *value MAL_DETAIL_MAYBE_UNUSED) {
    switch (value->tag) {
        case UINT32_C(0): {
            mal_detail_release(&value->payload.variant_0);
            return;
        }
        case UINT32_C(1): {
            mal_detail_release(&value->payload.variant_1);
            return;
        }
        default: {
            return;
        }
    }
}
static inline void mal_detail_storage_share_46708725c872772e(MalContext *context, void *carrier) {
    mal_call_t call = (mal_call_t){ .mal_detail_context = context };
    mal_detail_retain(&call, (mal_repr_sum_46708725c872772e_t *)carrier);
}
static inline void mal_detail_storage_drop_46708725c872772e(void *carrier) {
    mal_detail_release((mal_repr_sum_46708725c872772e_t *)carrier);
}
static inline __attribute__((overloadable)) mal_storage_descriptor_t mal_detail_storage(mal_repr_sum_46708725c872772e_t *type_marker MAL_DETAIL_MAYBE_UNUSED, size_t size, size_t alignment) {
    return (mal_storage_descriptor_t){ .size = size, .alignment = alignment, .share = mal_detail_storage_share_46708725c872772e, .drop = mal_detail_storage_drop_46708725c872772e };
}
#endif
#ifndef MAL_DETAIL_HOST_REPR_1e40e2e9f340e3fc_LIFECYCLE
#define MAL_DETAIL_HOST_REPR_1e40e2e9f340e3fc_LIFECYCLE
static inline __attribute__((overloadable)) void mal_detail_retain(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, mal_repr_product_1e40e2e9f340e3fc_t *value MAL_DETAIL_MAYBE_UNUSED) {
    mal_detail_retain(call, &value->field_0);
    mal_detail_retain(call, &value->field_1);
}
static inline __attribute__((overloadable)) void mal_detail_release(mal_repr_product_1e40e2e9f340e3fc_t *value MAL_DETAIL_MAYBE_UNUSED) {
    mal_detail_release(&value->field_0);
    mal_detail_release(&value->field_1);
}
static inline void mal_detail_storage_share_1e40e2e9f340e3fc(MalContext *context, void *carrier) {
    mal_call_t call = (mal_call_t){ .mal_detail_context = context };
    mal_detail_retain(&call, (mal_repr_product_1e40e2e9f340e3fc_t *)carrier);
}
static inline void mal_detail_storage_drop_1e40e2e9f340e3fc(void *carrier) {
    mal_detail_release((mal_repr_product_1e40e2e9f340e3fc_t *)carrier);
}
static inline __attribute__((overloadable)) mal_storage_descriptor_t mal_detail_storage(mal_repr_product_1e40e2e9f340e3fc_t *type_marker MAL_DETAIL_MAYBE_UNUSED, size_t size, size_t alignment) {
    return (mal_storage_descriptor_t){ .size = size, .alignment = alignment, .share = mal_detail_storage_share_1e40e2e9f340e3fc, .drop = mal_detail_storage_drop_1e40e2e9f340e3fc };
}
#endif
#ifndef MAL_DETAIL_HOST_REPR_4647c725c84fdeda_LIFECYCLE
#define MAL_DETAIL_HOST_REPR_4647c725c84fdeda_LIFECYCLE
static inline __attribute__((overloadable)) void mal_detail_retain(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, mal_repr_sum_4647c725c84fdeda_t *value MAL_DETAIL_MAYBE_UNUSED) {
    switch (value->tag) {
        case UINT32_C(0): {
            mal_detail_retain(call, &value->payload.variant_0);
            return;
        }
        case UINT32_C(1): {
            mal_detail_retain(call, &value->payload.variant_1);
            return;
        }
        default: {
            return;
        }
    }
}
static inline __attribute__((overloadable)) void mal_detail_release(mal_repr_sum_4647c725c84fdeda_t *value MAL_DETAIL_MAYBE_UNUSED) {
    switch (value->tag) {
        case UINT32_C(0): {
            mal_detail_release(&value->payload.variant_0);
            return;
        }
        case UINT32_C(1): {
            mal_detail_release(&value->payload.variant_1);
            return;
        }
        default: {
            return;
        }
    }
}
#endif
static inline void mal_detail_cleanup_ReadResult(mal_ReadResult_t *value) {
    mal_detail_release(value);
    memset(value, 0, sizeof(*value));
}
static inline void mal_detail_cleanup_WriteStatus(mal_WriteStatus_t *value) {
    mal_detail_release(value);
    memset(value, 0, sizeof(*value));
}
#ifndef MAL_DETAIL_HOST_REPR_46708725c872772e_HELPERS
#define MAL_DETAIL_HOST_REPR_46708725c872772e_HELPERS
MAL_DETAIL_DEFINE_SUM_CONVERSIONS(mal_detail_to_host_46708725c872772e, mal_detail_to_raw_46708725c872772e, MalRepr_Sum_46708725c872772e, mal_repr_sum_46708725c872772e_t, MAL_DETAIL_REPR_FIELDS_46708725c872772e)

#endif

#ifndef MAL_DETAIL_HOST_REPR_1e40e2e9f340e3fc_HELPERS
#define MAL_DETAIL_HOST_REPR_1e40e2e9f340e3fc_HELPERS
MAL_DETAIL_DEFINE_PRODUCT_CONVERSIONS(mal_detail_to_host_1e40e2e9f340e3fc, mal_detail_to_raw_1e40e2e9f340e3fc, MalRepr_Product_1e40e2e9f340e3fc, mal_repr_product_1e40e2e9f340e3fc_t, MAL_DETAIL_REPR_FIELDS_1e40e2e9f340e3fc)
#endif

#ifndef MAL_DETAIL_HOST_REPR_4647c725c84fdeda_HELPERS
#define MAL_DETAIL_HOST_REPR_4647c725c84fdeda_HELPERS
MAL_DETAIL_DEFINE_SUM_CONVERSIONS(mal_detail_to_host_4647c725c84fdeda, mal_detail_to_raw_4647c725c84fdeda, MalRepr_Sum_4647c725c84fdeda, mal_repr_sum_4647c725c84fdeda_t, MAL_DETAIL_REPR_FIELDS_4647c725c84fdeda)

#endif

/* External operations */

MalType_ReadResult mal_ext_readSource(MalContext *context, MalType_Symbol value);
MalType_WriteStatus mal_ext_writeChunk(MalContext *context, MalType_Int32 argument_0, MalType_Symbol argument_1);

/* External definition helpers */

#define MAL_HAS_EXTERN_readSource 1
#define MAL_DEFINE_readSource(call, value) \
static mal_ReadResult_t mal_detail_readSource(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, mal_Symbol_t value); \
MalType_ReadResult mal_ext_readSource(MalContext *context MAL_DETAIL_MAYBE_UNUSED, MalType_Symbol value) { \
    mal_call_t call = (mal_call_t){ .mal_detail_context = context }; \
    return mal_detail_to_raw_46708725c872772e(&call, mal_detail_readSource(&call, value)); \
} \
static mal_ReadResult_t mal_detail_readSource( \
    mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, \
    mal_Symbol_t value \
)

#define MAL_HAS_EXTERN_writeChunk 1
#define MAL_DEFINE_writeChunk(call, value) \
static mal_WriteStatus_t mal_detail_writeChunk(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, mal_repr_product_1e40e2e9f340e3fc_t value); \
MalType_WriteStatus mal_ext_writeChunk(MalContext *context MAL_DETAIL_MAYBE_UNUSED, MalType_Int32 argument_0, MalType_Symbol argument_1) { \
    mal_call_t call = (mal_call_t){ .mal_detail_context = context }; \
    return mal_detail_to_raw_4647c725c84fdeda(&call, mal_detail_writeChunk(&call, mal_detail_to_host_1e40e2e9f340e3fc(&call, (MalRepr_Product_1e40e2e9f340e3fc){ .field_0 = argument_0, .field_1 = argument_1 }))); \
} \
static mal_WriteStatus_t mal_detail_writeChunk( \
    mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, \
    mal_repr_product_1e40e2e9f340e3fc_t value \
)

#endif
#endif
