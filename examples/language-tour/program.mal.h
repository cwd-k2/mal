#ifndef MAL_BUILD_UMBRELLA
#include <mal.h>

_Static_assert(MAL_C_ABI_VERSION == 0x000900u, "generated header requires mal C ABI 0x000900");
_Static_assert((sizeof((size_t)0) * CHAR_BIT) == 64, "size_t does not match the mal target pointer index width");

#ifndef MAL_GENERATED_INTERFACE_B7E0FF89EB1D2F08_H
#define MAL_GENERATED_INTERFACE_B7E0FF89EB1D2F08_H

/* Host-visible types */

#ifndef MAL_DETAIL_HOST_REPR_4647c325c84fd80e_DECLARED
#define MAL_DETAIL_HOST_REPR_4647c325c84fd80e_DECLARED
typedef struct mal_detail_repr_sum_4647c325c84fd80e mal_repr_sum_4647c325c84fd80e_t;
#endif
typedef mal_repr_sum_4647c325c84fd80e_t mal_Division_t;

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

#endif
#endif
