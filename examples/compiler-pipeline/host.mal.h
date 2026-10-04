#ifndef MAL_BUILD_UMBRELLA
#include <mal.h>

_Static_assert(MAL_C_ABI_VERSION == 0x000a00u, "generated header requires mal C ABI 0x000a00");
_Static_assert(sizeof((size_t)0) == 8, "size_t does not match the mal target index width");
_Static_assert(sizeof((void *)0) == 8, "C pointer size does not match the mal target");
_Static_assert(sizeof(*(mal_Symbol_t *)0) == 24, "Symbol carrier size does not match the mal target");
_Static_assert(offsetof(mal_Symbol_t, owner) == 0, "Symbol owner offset does not match the mal target");
_Static_assert(offsetof(mal_Symbol_t, data) == 8, "Symbol data offset does not match the mal target");
_Static_assert(offsetof(mal_Symbol_t, length) == 16, "Symbol length offset does not match the mal target");

#ifndef MAL_GENERATED_INTERFACE_4F6CBADF55D0CD07_H
#define MAL_GENERATED_INTERFACE_4F6CBADF55D0CD07_H

/* Host-visible types */

#ifndef MAL_DETAIL_REPR_46708725c872772e_DECLARED
#define MAL_DETAIL_REPR_46708725c872772e_DECLARED
typedef struct mal_detail_repr_sum_46708725c872772e mal_repr_sum_46708725c872772e_t;
#endif
#ifndef MAL_DETAIL_REPR_1e40e2e9f340e3fc_DECLARED
#define MAL_DETAIL_REPR_1e40e2e9f340e3fc_DECLARED
typedef struct mal_detail_repr_product_1e40e2e9f340e3fc mal_repr_product_1e40e2e9f340e3fc_t;
#endif
#ifndef MAL_DETAIL_REPR_4647c725c84fdeda_DECLARED
#define MAL_DETAIL_REPR_4647c725c84fdeda_DECLARED
typedef struct mal_detail_repr_sum_4647c725c84fdeda mal_repr_sum_4647c725c84fdeda_t;
#endif
typedef mal_repr_sum_46708725c872772e_t mal_ReadResult_t;
typedef mal_repr_sum_4647c725c84fdeda_t mal_WriteStatus_t;

#ifndef MAL_DETAIL_REPR_FIELDS_46708725c872772e_DEFINED
#define MAL_DETAIL_REPR_FIELDS_46708725c872772e_DEFINED
#define MAL_DETAIL_REPR_FIELDS_46708725c872772e(field, context) \
field(context, 0, variant_0, mal_Buffer_t) \
field(context, 1, variant_1, mal_UInt32_t)
#endif
#ifndef MAL_DETAIL_REPR_46708725c872772e_DEFINED
#define MAL_DETAIL_REPR_46708725c872772e_DEFINED
MAL_DETAIL_DEFINE_SUM_REPR(mal_detail_repr_sum_46708725c872772e, MAL_DETAIL_REPR_FIELDS_46708725c872772e, MAL_DETAIL_REPR_FIELD)
typedef void (*mal_detail_sum_key_46708725c872772e_t)(mal_Buffer_t, mal_UInt32_t);
__attribute__((overloadable)) mal_repr_sum_46708725c872772e_t *mal_detail_sum_type(mal_detail_sum_key_46708725c872772e_t);
#endif

#ifndef MAL_DETAIL_REPR_FIELDS_1e40e2e9f340e3fc_DEFINED
#define MAL_DETAIL_REPR_FIELDS_1e40e2e9f340e3fc_DEFINED
#define MAL_DETAIL_REPR_FIELDS_1e40e2e9f340e3fc(field, context) \
field(context, 0, field_0, mal_Int32_t) \
field(context, 1, field_1, mal_Symbol_t)
#endif
#ifndef MAL_DETAIL_REPR_1e40e2e9f340e3fc_DEFINED
#define MAL_DETAIL_REPR_1e40e2e9f340e3fc_DEFINED
MAL_DETAIL_DEFINE_PRODUCT_REPR(mal_detail_repr_product_1e40e2e9f340e3fc, MAL_DETAIL_REPR_FIELDS_1e40e2e9f340e3fc, MAL_DETAIL_REPR_FIELD)
typedef void (*mal_detail_product_key_1e40e2e9f340e3fc_t)(mal_Int32_t, mal_Symbol_t);
__attribute__((overloadable)) mal_repr_product_1e40e2e9f340e3fc_t *mal_detail_product_type(mal_detail_product_key_1e40e2e9f340e3fc_t);
#endif

#ifndef MAL_DETAIL_REPR_FIELDS_4647c725c84fdeda_DEFINED
#define MAL_DETAIL_REPR_FIELDS_4647c725c84fdeda_DEFINED
#define MAL_DETAIL_REPR_FIELDS_4647c725c84fdeda(field, context) \
field(context, 0, variant_0, mal_Unit_t) \
field(context, 1, variant_1, mal_UInt32_t)
#endif
#ifndef MAL_DETAIL_REPR_4647c725c84fdeda_DEFINED
#define MAL_DETAIL_REPR_4647c725c84fdeda_DEFINED
MAL_DETAIL_DEFINE_SUM_REPR(mal_detail_repr_sum_4647c725c84fdeda, MAL_DETAIL_REPR_FIELDS_4647c725c84fdeda, MAL_DETAIL_REPR_FIELD)
typedef void (*mal_detail_sum_key_4647c725c84fdeda_t)(mal_Unit_t, mal_UInt32_t);
__attribute__((overloadable)) mal_repr_sum_4647c725c84fdeda_t *mal_detail_sum_type(mal_detail_sum_key_4647c725c84fdeda_t);
#endif

/* Type helpers */

#ifndef MAL_DETAIL_REPR_46708725c872772e_LIFECYCLE
#define MAL_DETAIL_REPR_46708725c872772e_LIFECYCLE
MAL_DETAIL_DEFINE_SUM_LIFECYCLE(mal_repr_sum_46708725c872772e_t, MAL_DETAIL_REPR_FIELDS_46708725c872772e, mal_detail_storage_share_46708725c872772e, mal_detail_storage_drop_46708725c872772e)
#endif
#ifndef MAL_DETAIL_REPR_1e40e2e9f340e3fc_LIFECYCLE
#define MAL_DETAIL_REPR_1e40e2e9f340e3fc_LIFECYCLE
MAL_DETAIL_DEFINE_PRODUCT_LIFECYCLE(mal_repr_product_1e40e2e9f340e3fc_t, MAL_DETAIL_REPR_FIELDS_1e40e2e9f340e3fc, mal_detail_storage_share_1e40e2e9f340e3fc, mal_detail_storage_drop_1e40e2e9f340e3fc)
#endif
static inline void mal_detail_cleanup_ReadResult(mal_ReadResult_t *value) {
    mal_detail_release(value);
    memset(value, 0, sizeof(*value));
}
static inline void mal_detail_cleanup_WriteStatus(mal_WriteStatus_t *value) {
    mal_detail_release(value);
    memset(value, 0, sizeof(*value));
}

/* External operations */

mal_ReadResult_t mal_ext_readSource(mal_call_t *call, mal_Symbol_t value);
mal_WriteStatus_t mal_ext_writeChunk(mal_call_t *call, mal_repr_product_1e40e2e9f340e3fc_t value);

/* External definition helpers */

#define MAL_HAS_EXTERN_readSource 1
#define MAL_DEFINE_readSource(call, value) mal_ReadResult_t mal_ext_readSource(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, mal_Symbol_t value)

#define MAL_HAS_EXTERN_writeChunk 1
#define MAL_DEFINE_writeChunk(call, value) mal_WriteStatus_t mal_ext_writeChunk(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, mal_repr_product_1e40e2e9f340e3fc_t value)

#endif
#endif
