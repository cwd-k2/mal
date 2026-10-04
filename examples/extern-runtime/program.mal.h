#ifndef MAL_BUILD_UMBRELLA
#include <mal.h>

_Static_assert(MAL_C_ABI_VERSION == 0x000a00u, "generated header requires mal C ABI 0x000a00");
_Static_assert(sizeof((size_t)0) == 8, "size_t does not match the mal target index width");
_Static_assert(sizeof((void *)0) == 8, "C pointer size does not match the mal target");
_Static_assert(sizeof(*(mal_Symbol_t *)0) == 24, "Symbol carrier size does not match the mal target");
_Static_assert(offsetof(mal_Symbol_t, owner) == 0, "Symbol owner offset does not match the mal target");
_Static_assert(offsetof(mal_Symbol_t, data) == 8, "Symbol data offset does not match the mal target");
_Static_assert(offsetof(mal_Symbol_t, length) == 16, "Symbol length offset does not match the mal target");

#ifndef MAL_GENERATED_INTERFACE_A581DF12273206BF_H
#define MAL_GENERATED_INTERFACE_A581DF12273206BF_H

/* Host-visible types */

#ifndef MAL_DETAIL_REPR_1e36b8e9f3384819_DECLARED
#define MAL_DETAIL_REPR_1e36b8e9f3384819_DECLARED
typedef struct mal_detail_repr_product_1e36b8e9f3384819 mal_repr_product_1e36b8e9f3384819_t;
#endif
typedef mal_repr_product_1e36b8e9f3384819_t mal_Sample_t;
typedef mal_Buffer_t mal_Samples_t;

#ifndef MAL_DETAIL_REPR_FIELDS_1e36b8e9f3384819_DEFINED
#define MAL_DETAIL_REPR_FIELDS_1e36b8e9f3384819_DEFINED
#define MAL_DETAIL_REPR_FIELDS_1e36b8e9f3384819(field, context) \
field(context, 0, field_0, mal_Int64_t) \
field(context, 1, field_1, mal_UInt8_t)
#endif
#ifndef MAL_DETAIL_REPR_1e36b8e9f3384819_DEFINED
#define MAL_DETAIL_REPR_1e36b8e9f3384819_DEFINED
MAL_DETAIL_DEFINE_PRODUCT_REPR(mal_detail_repr_product_1e36b8e9f3384819, MAL_DETAIL_REPR_FIELDS_1e36b8e9f3384819, MAL_DETAIL_REPR_FIELD)
typedef void (*mal_detail_product_key_1e36b8e9f3384819_t)(mal_Int64_t, mal_UInt8_t);
__attribute__((overloadable)) mal_repr_product_1e36b8e9f3384819_t *mal_detail_product_type(mal_detail_product_key_1e36b8e9f3384819_t);
#endif

/* Type helpers */

static inline void mal_detail_cleanup_Sample(mal_Sample_t *value) {
    mal_detail_release(value);
    memset(value, 0, sizeof(*value));
}
static inline void mal_detail_cleanup_Samples(mal_Samples_t *value) {
    mal_detail_release(value);
    memset(value, 0, sizeof(*value));
}

/* External operations */

mal_Samples_t mal_ext_sampleStorage(mal_call_t *call);
mal_Int32_t mal_ext_inspectSamples(mal_call_t *call, mal_Samples_t value);

/* External definition helpers */

#define MAL_HAS_EXTERN_sampleStorage 1
#define MAL_DEFINE_sampleStorage(call) mal_Samples_t mal_ext_sampleStorage(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED)

#define MAL_HAS_EXTERN_inspectSamples 1
#define MAL_DEFINE_inspectSamples(call, value) mal_Int32_t mal_ext_inspectSamples(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, mal_Samples_t value)

#endif
#endif
