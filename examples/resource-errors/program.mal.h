#ifndef MAL_BUILD_UMBRELLA
#include <mal.h>

_Static_assert(MAL_C_ABI_VERSION == 0x000a00u, "generated header requires mal C ABI 0x000a00");
_Static_assert(sizeof((size_t)0) == 8, "size_t does not match the mal target index width");
_Static_assert(sizeof((void *)0) == 8, "C pointer size does not match the mal target");
_Static_assert(sizeof(*(mal_Symbol_t *)0) == 24, "Symbol carrier size does not match the mal target");
_Static_assert(offsetof(mal_Symbol_t, owner) == 0, "Symbol owner offset does not match the mal target");
_Static_assert(offsetof(mal_Symbol_t, data) == 8, "Symbol data offset does not match the mal target");
_Static_assert(offsetof(mal_Symbol_t, length) == 16, "Symbol length offset does not match the mal target");

#ifndef MAL_GENERATED_INTERFACE_C3307B5EE72485CB_H
#define MAL_GENERATED_INTERFACE_C3307B5EE72485CB_H

/* Host-visible types */

typedef struct { uintptr_t bits; } mal_File_t;
#ifndef MAL_DETAIL_REPR_f2135c7115a77eb0_DECLARED
#define MAL_DETAIL_REPR_f2135c7115a77eb0_DECLARED
typedef struct mal_detail_repr_sum_f2135c7115a77eb0 mal_repr_sum_f2135c7115a77eb0_t;
#endif
#ifndef MAL_DETAIL_REPR_46708725c872772e_DECLARED
#define MAL_DETAIL_REPR_46708725c872772e_DECLARED
typedef struct mal_detail_repr_sum_46708725c872772e mal_repr_sum_46708725c872772e_t;
#endif
#ifndef MAL_DETAIL_REPR_4647c725c84fdeda_DECLARED
#define MAL_DETAIL_REPR_4647c725c84fdeda_DECLARED
typedef struct mal_detail_repr_sum_4647c725c84fdeda mal_repr_sum_4647c725c84fdeda_t;
#endif
typedef mal_repr_sum_f2135c7115a77eb0_t mal_OpenResult_t;
typedef mal_repr_sum_46708725c872772e_t mal_ReadResult_t;
typedef mal_repr_sum_4647c725c84fdeda_t mal_CloseResult_t;

#ifndef MAL_DETAIL_REPR_FIELDS_f2135c7115a77eb0_DEFINED
#define MAL_DETAIL_REPR_FIELDS_f2135c7115a77eb0_DEFINED
#define MAL_DETAIL_REPR_FIELDS_f2135c7115a77eb0(field, context) \
field(context, 0, variant_0, mal_File_t) \
field(context, 1, variant_1, mal_UInt32_t)
#endif
#ifndef MAL_DETAIL_REPR_f2135c7115a77eb0_DEFINED
#define MAL_DETAIL_REPR_f2135c7115a77eb0_DEFINED
MAL_DETAIL_DEFINE_SUM_REPR(mal_detail_repr_sum_f2135c7115a77eb0, MAL_DETAIL_REPR_FIELDS_f2135c7115a77eb0, MAL_DETAIL_REPR_FIELD)
typedef void (*mal_detail_sum_key_f2135c7115a77eb0_t)(mal_File_t, mal_UInt32_t);
__attribute__((overloadable)) mal_repr_sum_f2135c7115a77eb0_t *mal_detail_sum_type(mal_detail_sum_key_f2135c7115a77eb0_t);
#endif

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
MAL_DETAIL_DEFINE_SUM_LIFECYCLE(mal_repr_sum_46708725c872772e_t, MAL_DETAIL_REPR_FIELDS_46708725c872772e, mal_detail_storage_share_46708725c872772e, mal_detail_storage_drop_46708725c872772e, mal_detail_cleanup_46708725c872772e)
#endif
#define MAL_DETAIL_CLEANUP_ReadResult mal_detail_cleanup_46708725c872772e

/* External operations */

mal_OpenResult_t mal_ext_openReadOnly(mal_call_t *call, mal_Symbol_t value);
mal_ReadResult_t mal_ext_readFile(mal_call_t *call, mal_File_t value);
mal_CloseResult_t mal_ext_closeFile(mal_call_t *call, mal_File_t value);
mal_Unit_t mal_ext_writeBytes(mal_call_t *call, mal_Symbol_t value);
mal_Unit_t mal_ext_writeError(mal_call_t *call, mal_UInt32_t value);

/* External definition helpers */

#define MAL_HAS_EXTERN_openReadOnly 1
#define MAL_DEFINE_openReadOnly(call, value) mal_OpenResult_t mal_ext_openReadOnly(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, mal_Symbol_t value)

#define MAL_HAS_EXTERN_readFile 1
#define MAL_DEFINE_readFile(call, value) mal_ReadResult_t mal_ext_readFile(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, mal_File_t value)

#define MAL_HAS_EXTERN_closeFile 1
#define MAL_DEFINE_closeFile(call, value) mal_CloseResult_t mal_ext_closeFile(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, mal_File_t value)

#define MAL_HAS_EXTERN_writeBytes 1
#define MAL_DEFINE_writeBytes(call, value) mal_Unit_t mal_ext_writeBytes(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, mal_Symbol_t value)

#define MAL_HAS_EXTERN_writeError 1
#define MAL_DEFINE_writeError(call, value) mal_Unit_t mal_ext_writeError(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, mal_UInt32_t value)

#endif
#endif
