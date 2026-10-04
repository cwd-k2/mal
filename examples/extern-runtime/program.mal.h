#ifndef MAL_BUILD_UMBRELLA
#include <mal.h>

_Static_assert(MAL_C_ABI_VERSION == 0x000a00u, "generated header requires mal C ABI 0x000a00");
_Static_assert(sizeof((size_t)0) == 8, "size_t does not match the mal target index width");
_Static_assert(sizeof((void *)0) == 8, "C pointer size does not match the mal target");
_Static_assert(sizeof(*(MalType_Symbol *)0) == 24, "Symbol carrier size does not match the mal target");
_Static_assert(offsetof(MalType_Symbol, owner) == 0, "Symbol owner offset does not match the mal target");
_Static_assert(offsetof(MalType_Symbol, data) == 8, "Symbol data offset does not match the mal target");
_Static_assert(offsetof(MalType_Symbol, length) == 16, "Symbol length offset does not match the mal target");

#ifndef MAL_GENERATED_INTERFACE_5B556A22983DDCE3_H
#define MAL_GENERATED_INTERFACE_5B556A22983DDCE3_H

/* Host-visible types */

#ifndef MAL_DETAIL_RAW_REPR_1e36b8e9f3384819_DECLARED
#define MAL_DETAIL_RAW_REPR_1e36b8e9f3384819_DECLARED
typedef struct MalRepr_Product_1e36b8e9f3384819 MalRepr_Product_1e36b8e9f3384819;
#endif

#ifndef MAL_DETAIL_REPR_FIELDS_1e36b8e9f3384819_DEFINED
#define MAL_DETAIL_REPR_FIELDS_1e36b8e9f3384819_DEFINED
#define MAL_DETAIL_REPR_FIELDS_1e36b8e9f3384819(field, context) \
field(context, 0, field_0, MalType_Int64, mal_Int64_t, MAL_DETAIL_REPR_IDENTITY, MAL_DETAIL_REPR_IDENTITY) \
field(context, 1, field_1, MalType_UInt8, mal_UInt8_t, MAL_DETAIL_REPR_IDENTITY, MAL_DETAIL_REPR_IDENTITY)
#endif
#ifndef MAL_DETAIL_RAW_REPR_1e36b8e9f3384819_DEFINED
#define MAL_DETAIL_RAW_REPR_1e36b8e9f3384819_DEFINED
MAL_DETAIL_DEFINE_PRODUCT_REPR(MalRepr_Product_1e36b8e9f3384819, MAL_DETAIL_REPR_FIELDS_1e36b8e9f3384819, MAL_DETAIL_RAW_REPR_FIELD)
#endif

typedef MalRepr_Product_1e36b8e9f3384819 MalType_Sample;
typedef MalType_Buffer MalType_Samples;

#ifndef MAL_DETAIL_HOST_REPR_1e36b8e9f3384819_DECLARED
#define MAL_DETAIL_HOST_REPR_1e36b8e9f3384819_DECLARED
typedef struct mal_detail_repr_product_1e36b8e9f3384819 mal_repr_product_1e36b8e9f3384819_t;
#endif
typedef mal_repr_product_1e36b8e9f3384819_t mal_Sample_t;
typedef mal_Buffer_t mal_Samples_t;

#ifndef MAL_DETAIL_HOST_REPR_1e36b8e9f3384819_DEFINED
#define MAL_DETAIL_HOST_REPR_1e36b8e9f3384819_DEFINED
MAL_DETAIL_DEFINE_PRODUCT_REPR(mal_detail_repr_product_1e36b8e9f3384819, MAL_DETAIL_REPR_FIELDS_1e36b8e9f3384819, MAL_DETAIL_HOST_REPR_FIELD)
typedef void (*mal_detail_product_key_1e36b8e9f3384819_t)(mal_Int64_t, mal_UInt8_t);
__attribute__((overloadable)) mal_repr_product_1e36b8e9f3384819_t *mal_detail_product_type(mal_detail_product_key_1e36b8e9f3384819_t);
#endif

/* Type helpers */

#ifndef MAL_DETAIL_HOST_REPR_1e36b8e9f3384819_LIFECYCLE
#define MAL_DETAIL_HOST_REPR_1e36b8e9f3384819_LIFECYCLE
static inline __attribute__((overloadable)) void mal_detail_retain(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, mal_repr_product_1e36b8e9f3384819_t *value MAL_DETAIL_MAYBE_UNUSED) {
    mal_detail_retain(call, &value->field_0);
    mal_detail_retain(call, &value->field_1);
}
static inline __attribute__((overloadable)) void mal_detail_release(mal_repr_product_1e36b8e9f3384819_t *value MAL_DETAIL_MAYBE_UNUSED) {
    mal_detail_release(&value->field_0);
    mal_detail_release(&value->field_1);
}
#endif
static inline void mal_detail_cleanup_Sample(mal_Sample_t *value) {
    mal_detail_release(value);
    memset(value, 0, sizeof(*value));
}
static inline void mal_detail_cleanup_Samples(mal_Samples_t *value) {
    mal_detail_release(value);
    memset(value, 0, sizeof(*value));
}
#ifndef MAL_DETAIL_HOST_REPR_1e36b8e9f3384819_HELPERS
#define MAL_DETAIL_HOST_REPR_1e36b8e9f3384819_HELPERS
MAL_DETAIL_DEFINE_PRODUCT_CONVERSIONS(mal_detail_to_host_1e36b8e9f3384819, mal_detail_to_raw_1e36b8e9f3384819, MalRepr_Product_1e36b8e9f3384819, mal_repr_product_1e36b8e9f3384819_t, MAL_DETAIL_REPR_FIELDS_1e36b8e9f3384819)
#endif

/* External operations */

MalType_Samples mal_ext_sampleStorage(MalContext *context);
MalType_Int32 mal_ext_inspectSamples(MalContext *context, MalType_Samples value);

/* External definition helpers */

#define MAL_HAS_EXTERN_sampleStorage 1
#define MAL_DEFINE_sampleStorage(call) \
static mal_Samples_t mal_detail_sampleStorage(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED); \
MalType_Samples mal_ext_sampleStorage(MalContext *context MAL_DETAIL_MAYBE_UNUSED) { \
    mal_call_t call = (mal_call_t){ .mal_detail_context = context }; \
    return mal_detail_sampleStorage(&call); \
} \
static mal_Samples_t mal_detail_sampleStorage( \
    mal_call_t *call MAL_DETAIL_MAYBE_UNUSED \
)

#define MAL_HAS_EXTERN_inspectSamples 1
#define MAL_DEFINE_inspectSamples(call, value) \
static mal_Int32_t mal_detail_inspectSamples(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, mal_Samples_t value); \
MalType_Int32 mal_ext_inspectSamples(MalContext *context MAL_DETAIL_MAYBE_UNUSED, MalType_Samples value) { \
    mal_call_t call = (mal_call_t){ .mal_detail_context = context }; \
    return mal_detail_inspectSamples(&call, value); \
} \
static mal_Int32_t mal_detail_inspectSamples( \
    mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, \
    mal_Samples_t value \
)

#endif
#endif
