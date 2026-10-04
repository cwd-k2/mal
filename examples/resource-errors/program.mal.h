#ifndef MAL_BUILD_UMBRELLA
#include <mal.h>

_Static_assert(MAL_C_ABI_VERSION == 0x000a00u, "generated header requires mal C ABI 0x000a00");
_Static_assert(sizeof((size_t)0) == 8, "size_t does not match the mal target index width");
_Static_assert(sizeof((void *)0) == 8, "C pointer size does not match the mal target");
_Static_assert(sizeof(*(MalType_Symbol *)0) == 24, "Symbol carrier size does not match the mal target");
_Static_assert(offsetof(MalType_Symbol, owner) == 0, "Symbol owner offset does not match the mal target");
_Static_assert(offsetof(MalType_Symbol, data) == 8, "Symbol data offset does not match the mal target");
_Static_assert(offsetof(MalType_Symbol, length) == 16, "Symbol length offset does not match the mal target");

#ifndef MAL_GENERATED_INTERFACE_2F5B7A7146D5E0EE_H
#define MAL_GENERATED_INTERFACE_2F5B7A7146D5E0EE_H

/* Host-visible types */

typedef struct { uintptr_t bits; } MalType_File;

#ifndef MAL_DETAIL_RAW_REPR_f2135c7115a77eb0_DECLARED
#define MAL_DETAIL_RAW_REPR_f2135c7115a77eb0_DECLARED
typedef struct MalRepr_Sum_f2135c7115a77eb0 MalRepr_Sum_f2135c7115a77eb0;
#endif
#ifndef MAL_DETAIL_RAW_REPR_46708725c872772e_DECLARED
#define MAL_DETAIL_RAW_REPR_46708725c872772e_DECLARED
typedef struct MalRepr_Sum_46708725c872772e MalRepr_Sum_46708725c872772e;
#endif
#ifndef MAL_DETAIL_RAW_REPR_4647c725c84fdeda_DECLARED
#define MAL_DETAIL_RAW_REPR_4647c725c84fdeda_DECLARED
typedef struct MalRepr_Sum_4647c725c84fdeda MalRepr_Sum_4647c725c84fdeda;
#endif

#ifndef MAL_DETAIL_REPR_FIELDS_f2135c7115a77eb0_DEFINED
#define MAL_DETAIL_REPR_FIELDS_f2135c7115a77eb0_DEFINED
#define MAL_DETAIL_REPR_FIELDS_f2135c7115a77eb0(field, context) \
field(context, 0, variant_0, MalType_File, mal_File_t, mal_detail_to_host_File, mal_detail_to_raw_File) \
field(context, 1, variant_1, MalType_UInt32, mal_UInt32_t, mal_detail_to_raw_UInt32, mal_detail_to_raw_UInt32)
#endif
#ifndef MAL_DETAIL_RAW_REPR_f2135c7115a77eb0_DEFINED
#define MAL_DETAIL_RAW_REPR_f2135c7115a77eb0_DEFINED
MAL_DETAIL_DEFINE_SUM_REPR(MalRepr_Sum_f2135c7115a77eb0, MAL_DETAIL_REPR_FIELDS_f2135c7115a77eb0, MAL_DETAIL_RAW_REPR_FIELD)
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

typedef MalRepr_Sum_f2135c7115a77eb0 MalType_OpenResult;
typedef MalRepr_Sum_46708725c872772e MalType_ReadResult;
typedef MalRepr_Sum_4647c725c84fdeda MalType_CloseResult;

typedef struct { uintptr_t mal_detail_bits; } mal_File_t;
#ifndef MAL_DETAIL_HOST_REPR_f2135c7115a77eb0_DECLARED
#define MAL_DETAIL_HOST_REPR_f2135c7115a77eb0_DECLARED
typedef struct mal_detail_repr_sum_f2135c7115a77eb0 mal_repr_sum_f2135c7115a77eb0_t;
#endif
#ifndef MAL_DETAIL_HOST_REPR_46708725c872772e_DECLARED
#define MAL_DETAIL_HOST_REPR_46708725c872772e_DECLARED
typedef struct mal_detail_repr_sum_46708725c872772e mal_repr_sum_46708725c872772e_t;
#endif
#ifndef MAL_DETAIL_HOST_REPR_4647c725c84fdeda_DECLARED
#define MAL_DETAIL_HOST_REPR_4647c725c84fdeda_DECLARED
typedef struct mal_detail_repr_sum_4647c725c84fdeda mal_repr_sum_4647c725c84fdeda_t;
#endif
typedef mal_repr_sum_f2135c7115a77eb0_t mal_OpenResult_t;
typedef mal_repr_sum_46708725c872772e_t mal_ReadResult_t;
typedef mal_repr_sum_4647c725c84fdeda_t mal_CloseResult_t;

#ifndef MAL_DETAIL_HOST_REPR_f2135c7115a77eb0_DEFINED
#define MAL_DETAIL_HOST_REPR_f2135c7115a77eb0_DEFINED
MAL_DETAIL_DEFINE_SUM_REPR(mal_detail_repr_sum_f2135c7115a77eb0, MAL_DETAIL_REPR_FIELDS_f2135c7115a77eb0, MAL_DETAIL_HOST_REPR_FIELD)
typedef void (*mal_detail_sum_key_f2135c7115a77eb0_t)(mal_File_t, mal_UInt32_t);
__attribute__((overloadable)) mal_repr_sum_f2135c7115a77eb0_t *mal_detail_sum_type(mal_detail_sum_key_f2135c7115a77eb0_t);
#endif

#ifndef MAL_DETAIL_HOST_REPR_46708725c872772e_DEFINED
#define MAL_DETAIL_HOST_REPR_46708725c872772e_DEFINED
MAL_DETAIL_DEFINE_SUM_REPR(mal_detail_repr_sum_46708725c872772e, MAL_DETAIL_REPR_FIELDS_46708725c872772e, MAL_DETAIL_HOST_REPR_FIELD)
typedef void (*mal_detail_sum_key_46708725c872772e_t)(mal_Buffer_t, mal_UInt32_t);
__attribute__((overloadable)) mal_repr_sum_46708725c872772e_t *mal_detail_sum_type(mal_detail_sum_key_46708725c872772e_t);
#endif

#ifndef MAL_DETAIL_HOST_REPR_4647c725c84fdeda_DEFINED
#define MAL_DETAIL_HOST_REPR_4647c725c84fdeda_DEFINED
MAL_DETAIL_DEFINE_SUM_REPR(mal_detail_repr_sum_4647c725c84fdeda, MAL_DETAIL_REPR_FIELDS_4647c725c84fdeda, MAL_DETAIL_HOST_REPR_FIELD)
typedef void (*mal_detail_sum_key_4647c725c84fdeda_t)(mal_Unit_t, mal_UInt32_t);
__attribute__((overloadable)) mal_repr_sum_4647c725c84fdeda_t *mal_detail_sum_type(mal_detail_sum_key_4647c725c84fdeda_t);
#endif

/* Type helpers */

#ifndef MAL_DETAIL_HOST_REPR_f2135c7115a77eb0_LIFECYCLE
#define MAL_DETAIL_HOST_REPR_f2135c7115a77eb0_LIFECYCLE
static inline __attribute__((overloadable)) void mal_detail_retain(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, mal_repr_sum_f2135c7115a77eb0_t *value MAL_DETAIL_MAYBE_UNUSED) {
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
static inline __attribute__((overloadable)) void mal_detail_release(mal_repr_sum_f2135c7115a77eb0_t *value MAL_DETAIL_MAYBE_UNUSED) {
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
static inline void mal_detail_cleanup_File(mal_File_t *value) {
    mal_detail_release(value);
    memset(value, 0, sizeof(*value));
}
static inline void mal_detail_cleanup_OpenResult(mal_OpenResult_t *value) {
    mal_detail_release(value);
    memset(value, 0, sizeof(*value));
}
static inline void mal_detail_cleanup_ReadResult(mal_ReadResult_t *value) {
    mal_detail_release(value);
    memset(value, 0, sizeof(*value));
}
static inline void mal_detail_cleanup_CloseResult(mal_CloseResult_t *value) {
    mal_detail_release(value);
    memset(value, 0, sizeof(*value));
}
static inline mal_File_t mal_detail_to_host_File(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, MalType_File value);
static inline MalType_File mal_detail_to_raw_File(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, mal_File_t value);

#ifndef MAL_DETAIL_HOST_REPR_f2135c7115a77eb0_HELPERS
#define MAL_DETAIL_HOST_REPR_f2135c7115a77eb0_HELPERS
MAL_DETAIL_DEFINE_SUM_CONVERSIONS(mal_detail_to_host_f2135c7115a77eb0, mal_detail_to_raw_f2135c7115a77eb0, MalRepr_Sum_f2135c7115a77eb0, mal_repr_sum_f2135c7115a77eb0_t, MAL_DETAIL_REPR_FIELDS_f2135c7115a77eb0)

#endif

#ifndef MAL_DETAIL_HOST_REPR_46708725c872772e_HELPERS
#define MAL_DETAIL_HOST_REPR_46708725c872772e_HELPERS
MAL_DETAIL_DEFINE_SUM_CONVERSIONS(mal_detail_to_host_46708725c872772e, mal_detail_to_raw_46708725c872772e, MalRepr_Sum_46708725c872772e, mal_repr_sum_46708725c872772e_t, MAL_DETAIL_REPR_FIELDS_46708725c872772e)

#endif

#ifndef MAL_DETAIL_HOST_REPR_4647c725c84fdeda_HELPERS
#define MAL_DETAIL_HOST_REPR_4647c725c84fdeda_HELPERS
MAL_DETAIL_DEFINE_SUM_CONVERSIONS(mal_detail_to_host_4647c725c84fdeda, mal_detail_to_raw_4647c725c84fdeda, MalRepr_Sum_4647c725c84fdeda, mal_repr_sum_4647c725c84fdeda_t, MAL_DETAIL_REPR_FIELDS_4647c725c84fdeda)

#endif

static inline mal_File_t mal_detail_to_host_File(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, MalType_File value) {
    return (mal_File_t){ .mal_detail_bits = value.bits };
}

static inline __attribute__((overloadable)) mal_File_t mal_detail_from_bits(mal_File_t *type_marker MAL_DETAIL_MAYBE_UNUSED, uintptr_t bits) {
    return (mal_File_t){ .mal_detail_bits = bits };
}

static inline __attribute__((overloadable)) uintptr_t mal_detail_bits(mal_File_t value) {
    return value.mal_detail_bits;
}

static inline MalType_File mal_detail_to_raw_File(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, mal_File_t value) {
    return (MalType_File){ .bits = value.mal_detail_bits };
}

/* External operations */

MalType_OpenResult mal_ext_openReadOnly(MalContext *context, MalType_Symbol value);
MalType_ReadResult mal_ext_readFile(MalContext *context, MalType_File value);
MalType_CloseResult mal_ext_closeFile(MalContext *context, MalType_File value);
void mal_ext_writeBytes(MalContext *context, MalType_Symbol value);
void mal_ext_writeError(MalContext *context, MalType_UInt32 value);

/* External definition helpers */

#define MAL_HAS_EXTERN_openReadOnly 1
#define MAL_DEFINE_openReadOnly(call, value) \
static mal_OpenResult_t mal_detail_openReadOnly(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, mal_Symbol_t value); \
MalType_OpenResult mal_ext_openReadOnly(MalContext *context MAL_DETAIL_MAYBE_UNUSED, MalType_Symbol value) { \
    mal_call_t call = (mal_call_t){ .mal_detail_context = context }; \
    return mal_detail_to_raw_f2135c7115a77eb0(&call, mal_detail_openReadOnly(&call, value)); \
} \
static mal_OpenResult_t mal_detail_openReadOnly( \
    mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, \
    mal_Symbol_t value \
)

#define MAL_HAS_EXTERN_readFile 1
#define MAL_DEFINE_readFile(call, value) \
static mal_ReadResult_t mal_detail_readFile(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, mal_File_t value); \
MalType_ReadResult mal_ext_readFile(MalContext *context MAL_DETAIL_MAYBE_UNUSED, MalType_File value) { \
    mal_call_t call = (mal_call_t){ .mal_detail_context = context }; \
    return mal_detail_to_raw_46708725c872772e(&call, mal_detail_readFile(&call, (mal_File_t){ .mal_detail_bits = value.bits })); \
} \
static mal_ReadResult_t mal_detail_readFile( \
    mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, \
    mal_File_t value \
)

#define MAL_HAS_EXTERN_closeFile 1
#define MAL_DEFINE_closeFile(call, value) \
static mal_CloseResult_t mal_detail_closeFile(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, mal_File_t value); \
MalType_CloseResult mal_ext_closeFile(MalContext *context MAL_DETAIL_MAYBE_UNUSED, MalType_File value) { \
    mal_call_t call = (mal_call_t){ .mal_detail_context = context }; \
    return mal_detail_to_raw_4647c725c84fdeda(&call, mal_detail_closeFile(&call, (mal_File_t){ .mal_detail_bits = value.bits })); \
} \
static mal_CloseResult_t mal_detail_closeFile( \
    mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, \
    mal_File_t value \
)

#define MAL_HAS_EXTERN_writeBytes 1
#define MAL_DEFINE_writeBytes(call, value) \
static mal_Unit_t mal_detail_writeBytes(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, mal_Symbol_t value); \
void mal_ext_writeBytes(MalContext *context MAL_DETAIL_MAYBE_UNUSED, MalType_Symbol value) { \
    mal_call_t call = (mal_call_t){ .mal_detail_context = context }; \
    mal_detail_writeBytes(&call, value); \
} \
static mal_Unit_t mal_detail_writeBytes( \
    mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, \
    mal_Symbol_t value \
)

#define MAL_HAS_EXTERN_writeError 1
#define MAL_DEFINE_writeError(call, value) \
static mal_Unit_t mal_detail_writeError(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, mal_UInt32_t value); \
void mal_ext_writeError(MalContext *context MAL_DETAIL_MAYBE_UNUSED, MalType_UInt32 value) { \
    mal_call_t call = (mal_call_t){ .mal_detail_context = context }; \
    mal_detail_writeError(&call, value); \
} \
static mal_Unit_t mal_detail_writeError( \
    mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, \
    mal_UInt32_t value \
)

#endif
#endif
