#ifndef MAL_BUILD_UMBRELLA
#include <mal.h>

_Static_assert(MAL_C_ABI_VERSION == 0x000900u, "generated header requires mal C ABI 0x000900");
_Static_assert((sizeof((size_t)0) * CHAR_BIT) == 64, "size_t does not match the mal target pointer index width");

#ifndef MAL_GENERATED_INTERFACE_B5F8596A1E41BF1A_H
#define MAL_GENERATED_INTERFACE_B5F8596A1E41BF1A_H
/* Host-visible types */

#ifndef MAL_DETAIL_HOST_REPR_1e36b8e9f3384819_DECLARED
#define MAL_DETAIL_HOST_REPR_1e36b8e9f3384819_DECLARED
typedef struct mal_detail_repr_product_1e36b8e9f3384819 mal_repr_product_1e36b8e9f3384819_t;
#endif
typedef mal_repr_product_1e36b8e9f3384819_t mal_SampleRecord_t;

#ifndef MAL_DETAIL_REPR_FIELDS_1e36b8e9f3384819_DEFINED
#define MAL_DETAIL_REPR_FIELDS_1e36b8e9f3384819_DEFINED
#define MAL_DETAIL_REPR_FIELDS_1e36b8e9f3384819(field) \
field(field_0, MalType_Int64, mal_Int64_t) \
field(field_1, MalType_UInt8, mal_UInt8_t)
#endif
#ifndef MAL_DETAIL_HOST_REPR_1e36b8e9f3384819_DEFINED
#define MAL_DETAIL_HOST_REPR_1e36b8e9f3384819_DEFINED
MAL_DETAIL_DEFINE_PRODUCT_REPR(mal_detail_repr_product_1e36b8e9f3384819, MAL_DETAIL_REPR_FIELDS_1e36b8e9f3384819, MAL_DETAIL_HOST_REPR_FIELD)
#endif

/* Canonical memory access */

#ifndef MAL_DETAIL_MEMORY_REPR_1e36b8e9f3384819_HELPERS
#define MAL_DETAIL_MEMORY_REPR_1e36b8e9f3384819_HELPERS
#define MAL_DETAIL_MEMORY_FIELDS_1e36b8e9f3384819(unit, value) \
value(field_0, mal_detail_memory_read_Int64, mal_detail_memory_write_Int64, 0) \
value(field_1, mal_detail_memory_read_UInt8, mal_detail_memory_write_UInt8, 8)
MAL_DETAIL_DEFINE_MEMORY_PRODUCT(mal_detail_memory_read_1e36b8e9f3384819, mal_detail_memory_write_1e36b8e9f3384819, mal_repr_product_1e36b8e9f3384819_t, MAL_DETAIL_MEMORY_FIELDS_1e36b8e9f3384819)
#endif

MAL_DETAIL_DEFINE_MEMORY_ALIAS(mal_SampleRecord_read, mal_SampleRecord_write, mal_SampleRecord_t, 16, mal_detail_memory_read_1e36b8e9f3384819, mal_detail_memory_write_1e36b8e9f3384819)

/* External operations */

MalType_Address mal_ext_unalignedStorage(MalContext *context);
void mal_ext_incrementSample(MalContext *context, MalType_Address value);

/* External definition helpers */

#define MAL_HAS_EXTERN_unalignedStorage 1
#define MAL_DEFINE_unalignedStorage(call) \
static MalType_Address mal_detail_unalignedStorage(mal_call_t *call); \
MalType_Address mal_ext_unalignedStorage(MalContext *context MAL_DETAIL_MAYBE_UNUSED) { \
    mal_call_t call = (mal_call_t){ .mal_detail_context = context }; \
    return mal_detail_unalignedStorage(&call); \
} \
static MalType_Address mal_detail_unalignedStorage( \
    mal_call_t *call \
)

#define MAL_HAS_EXTERN_incrementSample 1
#define MAL_DEFINE_incrementSample(call, value) \
static MalType_Unit mal_detail_incrementSample(mal_call_t *call, mal_Address_t value); \
void mal_ext_incrementSample(MalContext *context MAL_DETAIL_MAYBE_UNUSED, MalType_Address value) { \
    mal_call_t call = (mal_call_t){ .mal_detail_context = context }; \
    mal_detail_incrementSample(&call, value); \
} \
static MalType_Unit mal_detail_incrementSample( \
    mal_call_t *call, \
    mal_Address_t value \
)

#endif
#endif
