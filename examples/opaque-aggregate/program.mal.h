#ifndef MAL_PROGRAM_MAL_H
#define MAL_PROGRAM_MAL_H

#include <stdint.h>

#define MAL_C_ABI_VERSION 0x000500u
#define MAL_TYPE(name) MalType_##name
#define MAL_OPERATION(type, operation) mal_##type##_##operation
#define MAL_TAG(type, variant) MAL_##type##_TAG_##variant
#define MAL_EXTERN(name) mal_ext_##name

#if defined(__clang__) || defined(__GNUC__)
#define MAL_DETAIL_MAYBE_UNUSED __attribute__((unused))
#else
#define MAL_DETAIL_MAYBE_UNUSED
#endif

/* Runtime API */

typedef struct MalContext MalContext;
typedef struct { uint8_t unused; } MalType_Unit;
typedef uint8_t MalType_Bool;
typedef int8_t MalType_Int8;
typedef int16_t MalType_Int16;
typedef int32_t MalType_Int32;
typedef int64_t MalType_Int64;
typedef uint8_t MalType_UInt8;
typedef uint16_t MalType_UInt16;
typedef uint32_t MalType_UInt32;
typedef uint64_t MalType_UInt64;
typedef float MalType_Float32;
typedef double MalType_Float64;
typedef struct { const uint8_t *data; uint64_t length; } MalType_Symbol;
typedef struct { uint8_t *address; } MalType_Ptr;

#define MAL_FALSE ((MalType_Bool)UINT8_C(0))
#define MAL_TRUE ((MalType_Bool)UINT8_C(1))

_Noreturn void mal_trap(MalContext *context, const char *message);
MalType_Symbol mal_Symbol_copy_from_bytes(MalContext *context, const uint8_t *data, uint64_t length);

static inline const uint8_t *mal_Symbol_data(MalType_Symbol value) {
    return value.data;
}

static inline uint64_t mal_Symbol_length(MalType_Symbol value) {
    return value.length;
}

static inline MalType_Ptr mal_Ptr_from_address(uint8_t *address) {
    return (MalType_Ptr){ .address = address };
}

static inline uint8_t *mal_Ptr_address(MalType_Ptr value) {
    return value.address;
}

/* Host-visible types */

typedef struct { uintptr_t bits; } MalType_Mem;

typedef struct MalRepr_Product_0 MalRepr_Product_0;
typedef struct MalRepr_Sum_1 MalRepr_Sum_1;

struct MalRepr_Product_0 {
    MalType_Mem field_0;
    MalType_UInt64 field_1;
};

struct MalRepr_Sum_1 {
    uint32_t tag;
    union {
        MalType_Unit variant_0;
        MalRepr_Product_0 variant_1;
    } payload;
};

typedef MalRepr_Sum_1 MalType_Response;

/* Type helpers */

static inline MalType_Mem mal_Mem_from_bits(uintptr_t bits) {
    return (MalType_Mem){ .bits = bits };
}

static inline uintptr_t mal_Mem_bits(MalType_Mem value) {
    return value.bits;
}

#define MAL_Response_TAG_0 UINT32_C(0)
#define MAL_Response_TAG_1 UINT32_C(1)
static inline uint32_t mal_Response_tag(MalType_Response value) {
    return value.tag;
}

static inline MalType_Bool mal_Response_is_0(MalType_Response value) {
    return value.tag == MAL_Response_TAG_0;
}

static inline MalType_Response mal_Response_make_0(void) {
    return (MalType_Response){ .tag = MAL_Response_TAG_0, .payload.variant_0 = (MalType_Unit){ .unused = UINT8_C(0) } };
}

static inline MalType_Bool mal_Response_is_1(MalType_Response value) {
    return value.tag == MAL_Response_TAG_1;
}

static inline MalType_Response mal_Response_make_1(MalType_Mem value_0, MalType_UInt64 value_1) {
    return (MalType_Response){ .tag = MAL_Response_TAG_1, .payload.variant_1 = (MalRepr_Product_0){ .field_0 = value_0, .field_1 = value_1 } };
}

static inline MalType_Mem mal_Response_expect_1_0(MalContext *context, MalType_Response value) {
    if (!mal_Response_is_1(value)) {
        mal_trap(context, "expected Response variant 1");
    }
    return value.payload.variant_1.field_0;
}

static inline MalType_UInt64 mal_Response_expect_1_1(MalContext *context, MalType_Response value) {
    if (!mal_Response_is_1(value)) {
        mal_trap(context, "expected Response variant 1");
    }
    return value.payload.variant_1.field_1;
}

/* External operations */

MalType_Mem mal_ext_allocate(MalContext *context, MalType_UInt64 value);
MalType_Response mal_ext_resize(MalContext *context, MalType_Mem argument_0, MalType_UInt64 argument_1);
MalType_UInt64 mal_ext_handleBits(MalContext *context, MalType_Mem value);

/* External definition helpers */

#define MAL_HAS_EXTERN_allocate 1
#define MAL_DEFINE_allocate(context, value) \
    MalType_Mem mal_ext_allocate( \
    MalContext *context MAL_DETAIL_MAYBE_UNUSED, \
    MalType_UInt64 value \
    )

#define MAL_HAS_EXTERN_resize 1
#define MAL_DEFINE_resize(context, argument_0, argument_1) \
    MalType_Response mal_ext_resize( \
    MalContext *context MAL_DETAIL_MAYBE_UNUSED, \
    MalType_Mem argument_0, \
    MalType_UInt64 argument_1 \
    )

#define MAL_HAS_EXTERN_handleBits 1
#define MAL_DEFINE_handleBits(context, value) \
    MalType_UInt64 mal_ext_handleBits( \
    MalContext *context MAL_DETAIL_MAYBE_UNUSED, \
    MalType_Mem value \
    )

#endif
