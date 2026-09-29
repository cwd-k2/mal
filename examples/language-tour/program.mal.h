#ifndef MAL_BUILD_UMBRELLA
#include <mal.h>
#include "box.mal.h"

_Static_assert(MAL_C_ABI_VERSION == 0x000900u, "generated header requires mal C ABI 0x000900");
_Static_assert((sizeof((size_t)0) * CHAR_BIT) == 64, "size_t does not match the mal target pointer index width");

#ifndef MAL_GENERATED_INTERFACE_A7E4354306512C9A_H
#define MAL_GENERATED_INTERFACE_A7E4354306512C9A_H

/* Host-visible types */

#ifndef MAL_DETAIL_RAW_REPR_11bb4291f5045bdf_DECLARED
#define MAL_DETAIL_RAW_REPR_11bb4291f5045bdf_DECLARED
typedef struct MalRepr_Product_11bb4291f5045bdf MalRepr_Product_11bb4291f5045bdf;
#endif

#ifndef MAL_DETAIL_REPR_FIELDS_11bb4291f5045bdf_DEFINED
#define MAL_DETAIL_REPR_FIELDS_11bb4291f5045bdf_DEFINED
#define MAL_DETAIL_REPR_FIELDS_11bb4291f5045bdf(field, context) \
field(context, 0, field_0, MalType_UInt64, mal_UInt64_t, MAL_DETAIL_REPR_IDENTITY, MAL_DETAIL_REPR_IDENTITY) \
field(context, 1, field_1, MalType_UInt64, mal_UInt64_t, MAL_DETAIL_REPR_IDENTITY, MAL_DETAIL_REPR_IDENTITY) \
field(context, 2, field_2, MalType_UInt64, mal_UInt64_t, MAL_DETAIL_REPR_IDENTITY, MAL_DETAIL_REPR_IDENTITY) \
field(context, 3, field_3, MalType_Float32, mal_Float32_t, MAL_DETAIL_REPR_IDENTITY, MAL_DETAIL_REPR_IDENTITY) \
field(context, 4, field_4, MalType_Float32, mal_Float32_t, MAL_DETAIL_REPR_IDENTITY, MAL_DETAIL_REPR_IDENTITY) \
field(context, 5, field_5, MalType_Float64, mal_Float64_t, MAL_DETAIL_REPR_IDENTITY, MAL_DETAIL_REPR_IDENTITY) \
field(context, 6, field_6, MalType_Int64, mal_Int64_t, MAL_DETAIL_REPR_IDENTITY, MAL_DETAIL_REPR_IDENTITY)
#endif
#ifndef MAL_DETAIL_RAW_REPR_11bb4291f5045bdf_DEFINED
#define MAL_DETAIL_RAW_REPR_11bb4291f5045bdf_DEFINED
MAL_DETAIL_DEFINE_PRODUCT_REPR(MalRepr_Product_11bb4291f5045bdf, MAL_DETAIL_REPR_FIELDS_11bb4291f5045bdf, MAL_DETAIL_RAW_REPR_FIELD)
#endif

#ifndef MAL_DETAIL_HOST_REPR_11bb4291f5045bdf_DECLARED
#define MAL_DETAIL_HOST_REPR_11bb4291f5045bdf_DECLARED
typedef struct mal_detail_repr_product_11bb4291f5045bdf mal_repr_product_11bb4291f5045bdf_t;
#endif
#ifndef MAL_DETAIL_HOST_REPR_4647c325c84fd80e_DECLARED
#define MAL_DETAIL_HOST_REPR_4647c325c84fd80e_DECLARED
typedef struct mal_detail_repr_sum_4647c325c84fd80e mal_repr_sum_4647c325c84fd80e_t;
#endif
typedef mal_repr_sum_4647c325c84fd80e_t mal_Division_t;

#ifndef MAL_DETAIL_HOST_REPR_11bb4291f5045bdf_DEFINED
#define MAL_DETAIL_HOST_REPR_11bb4291f5045bdf_DEFINED
MAL_DETAIL_DEFINE_PRODUCT_REPR(mal_detail_repr_product_11bb4291f5045bdf, MAL_DETAIL_REPR_FIELDS_11bb4291f5045bdf, MAL_DETAIL_HOST_REPR_FIELD)
#endif

#ifndef MAL_DETAIL_REPR_FIELDS_4647c325c84fd80e_DEFINED
#define MAL_DETAIL_REPR_FIELDS_4647c325c84fd80e_DEFINED
#define MAL_DETAIL_REPR_FIELDS_4647c325c84fd80e(field, context) \
field(context, 0, variant_0, MalType_Unit, mal_Unit_t, mal_detail_convert_Unit, mal_detail_convert_Unit) \
field(context, 1, variant_1, MalType_Int32, mal_Int32_t, mal_Int32_return, mal_Int32_return)
#endif
#ifndef MAL_DETAIL_HOST_REPR_4647c325c84fd80e_DEFINED
#define MAL_DETAIL_HOST_REPR_4647c325c84fd80e_DEFINED
MAL_DETAIL_DEFINE_SUM_REPR(mal_detail_repr_sum_4647c325c84fd80e, MAL_DETAIL_REPR_FIELDS_4647c325c84fd80e, MAL_DETAIL_HOST_REPR_FIELD)
#endif

/* Type helpers */

#ifndef MAL_DETAIL_HOST_REPR_11bb4291f5045bdf_HELPERS
#define MAL_DETAIL_HOST_REPR_11bb4291f5045bdf_HELPERS
MAL_DETAIL_DEFINE_PRODUCT_CONVERSIONS(mal_detail_to_host_11bb4291f5045bdf, mal_repr_product_11bb4291f5045bdf_return, MalRepr_Product_11bb4291f5045bdf, mal_repr_product_11bb4291f5045bdf_t, MAL_DETAIL_REPR_FIELDS_11bb4291f5045bdf)
#endif

/* Canonical memory access */

#ifndef MAL_DETAIL_MEMORY_REPR_4647c325c84fd80e_HELPERS
#define MAL_DETAIL_MEMORY_REPR_4647c325c84fd80e_HELPERS
#define MAL_DETAIL_MEMORY_MEMBERS_4647c325c84fd80e(member) \
member(mal_repr_sum_4647c325c84fd80e_t, mal_UInt8_t, mal_detail_memory_write_UInt8, 0, variant_0, mal_detail_memory_read_Unit, mal_detail_memory_write_Unit, 4) \
member(mal_repr_sum_4647c325c84fd80e_t, mal_UInt8_t, mal_detail_memory_write_UInt8, 1, variant_1, mal_detail_memory_read_Int32, mal_detail_memory_write_Int32, 4)
MAL_DETAIL_DEFINE_MEMORY_SUM(mal_detail_memory_read_4647c325c84fd80e, mal_detail_memory_write_4647c325c84fd80e, mal_repr_sum_4647c325c84fd80e_t, mal_detail_memory_read_UInt8, MAL_DETAIL_MEMORY_MEMBERS_4647c325c84fd80e)
#endif

MAL_DETAIL_DEFINE_MEMORY_ALIAS(mal_Division_read, mal_Division_write, mal_Division_t, 8, mal_detail_memory_read_4647c325c84fd80e, mal_detail_memory_write_4647c325c84fd80e)

/* External operations */

void mal_ext_printInt32(MalContext *context, MalType_Int32 value);
MalType_Int32 mal_ext_inspectNumeric(MalContext *context, MalType_UInt64 argument_0, MalType_UInt64 argument_1, MalType_UInt64 argument_2, MalType_Float32 argument_3, MalType_Float32 argument_4, MalType_Float64 argument_5, MalType_Int64 argument_6);

/* External definition helpers */

#define MAL_HAS_EXTERN_printInt32 1
#define MAL_DEFINE_printInt32(call, value) \
static MalType_Unit mal_detail_printInt32(mal_call_t *call, mal_Int32_t value); \
void mal_ext_printInt32(MalContext *context MAL_DETAIL_MAYBE_UNUSED, MalType_Int32 value) { \
    mal_call_t call = (mal_call_t){ .mal_detail_context = context }; \
    mal_detail_printInt32(&call, value); \
} \
static MalType_Unit mal_detail_printInt32( \
    mal_call_t *call, \
    mal_Int32_t value \
)

#define MAL_HAS_EXTERN_inspectNumeric 1
#define MAL_DEFINE_inspectNumeric(call, value) \
static MalType_Int32 mal_detail_inspectNumeric(mal_call_t *call, mal_repr_product_11bb4291f5045bdf_t value); \
MalType_Int32 mal_ext_inspectNumeric(MalContext *context MAL_DETAIL_MAYBE_UNUSED, MalType_UInt64 argument_0, MalType_UInt64 argument_1, MalType_UInt64 argument_2, MalType_Float32 argument_3, MalType_Float32 argument_4, MalType_Float64 argument_5, MalType_Int64 argument_6) { \
    mal_call_t call = (mal_call_t){ .mal_detail_context = context }; \
    return mal_detail_inspectNumeric(&call, mal_detail_to_host_11bb4291f5045bdf(&call, (MalRepr_Product_11bb4291f5045bdf){ .field_0 = argument_0, .field_1 = argument_1, .field_2 = argument_2, .field_3 = argument_3, .field_4 = argument_4, .field_5 = argument_5, .field_6 = argument_6 })); \
} \
static MalType_Int32 mal_detail_inspectNumeric( \
    mal_call_t *call, \
    mal_repr_product_11bb4291f5045bdf_t value \
)

#endif
#endif
