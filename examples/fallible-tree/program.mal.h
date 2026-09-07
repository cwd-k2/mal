#ifndef MAL_PROGRAM_MAL_H
#define MAL_PROGRAM_MAL_H

#include <stdint.h>

#define MAL_C_ABI_VERSION 0x000500u
#define MAL_TYPE(name) MalType_##name
#define MAL_OPERATION(type, operation) mal_##type##_##operation
#define MAL_TAG(type, variant) MAL_##type##_TAG_##variant
#define MAL_EXTERN(name) mal_ext_##name
#define MAL_CLONE(owner) mal_##owner##_clone
#define MAL_MOVE(owner) mal_##owner##_take
#define MAL_DROP(owner) mal_##owner##_drop

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
typedef struct { const uint8_t *data; uint64_t length; void *ownership; } MalType_Symbol;
typedef struct { uint8_t *address; } MalType_Ptr;

#define MAL_FALSE (MalType_Bool)UINT8_C(0)
#define MAL_TRUE (MalType_Bool)UINT8_C(1)

_Noreturn void mal_trap(MalContext *context, const char *message);
MalType_Symbol mal_Symbol_copy_from_bytes(MalContext *context, const uint8_t *data, uint64_t length);
MalType_Symbol mal_Symbol_clone(MalContext *context, MalType_Symbol value);
MalType_Symbol mal_Symbol_take(MalType_Symbol *value);
void mal_Symbol_drop(MalContext *context, MalType_Symbol *value);

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

typedef struct { uintptr_t bits; } MalType_Allocator;

typedef struct MalRepr_Product_0 MalRepr_Product_0;
typedef struct MalRepr_Sum_1 MalRepr_Sum_1;
typedef struct MalRepr_Product_2 MalRepr_Product_2;

struct MalRepr_Product_0 {
    MalType_Allocator field_0;
    MalType_UInt64 field_1;
};

struct MalRepr_Sum_1 {
    uint32_t tag;
    union {
        MalType_Ptr variant_0;
        MalType_Unit variant_1;
    } payload;
};

struct MalRepr_Product_2 {
    MalType_Allocator field_0;
    MalType_Ptr field_1;
};

typedef MalRepr_Sum_1 MalType_NodeResult;

/* Type helpers */

static inline MalType_Allocator mal_Allocator_from_bits(uintptr_t bits) {
    return (MalType_Allocator){ .bits = bits };
}

static inline uintptr_t mal_Allocator_bits(MalType_Allocator value) {
    return value.bits;
}

#define MAL_NodeResult_TAG_0 UINT32_C(0)
#define MAL_NodeResult_TAG_1 UINT32_C(1)
static inline uint32_t mal_NodeResult_tag(MalType_NodeResult value) {
    return value.tag;
}

static inline MalType_Bool mal_NodeResult_is_0(MalType_NodeResult value) {
    return value.tag == MAL_NodeResult_TAG_0;
}

static inline MalType_NodeResult mal_NodeResult_make_0(MalType_Ptr value) {
    return (MalType_NodeResult){ .tag = MAL_NodeResult_TAG_0, .payload.variant_0 = value };
}

static inline MalType_Ptr mal_NodeResult_expect_0(MalContext *context, MalType_NodeResult value) {
    if (!mal_NodeResult_is_0(value)) {
        mal_trap(context, "expected NodeResult variant 0");
    }
    return value.payload.variant_0;
}

static inline MalType_Bool mal_NodeResult_is_1(MalType_NodeResult value) {
    return value.tag == MAL_NodeResult_TAG_1;
}

static inline MalType_NodeResult mal_NodeResult_make_1(void) {
    return (MalType_NodeResult){ .tag = MAL_NodeResult_TAG_1, .payload.variant_1 = (MalType_Unit){ .unused = UINT8_C(0) } };
}

/* External operations */

MalType_Allocator mal_ext_createAllocator(MalContext *context, MalType_UInt64 value);
MalType_NodeResult mal_ext_allocateNode(MalContext *context, MalType_Allocator argument_0, MalType_UInt64 argument_1);
void mal_ext_releaseNode(MalContext *context, MalType_Allocator argument_0, MalType_Ptr argument_1);
void mal_ext_destroyAllocator(MalContext *context, MalType_Allocator value);

/* External definition helpers */

#define MAL_HAS_EXTERN_createAllocator 1
#define MAL_DEFINE_createAllocator(context, value) \
MalType_Allocator mal_ext_createAllocator( \
    MalContext *context MAL_DETAIL_MAYBE_UNUSED, \
    MalType_UInt64 value \
)

#define MAL_HAS_EXTERN_allocateNode 1
#define MAL_DEFINE_allocateNode(context, argument_0, argument_1) \
MalType_NodeResult mal_ext_allocateNode( \
    MalContext *context MAL_DETAIL_MAYBE_UNUSED, \
    MalType_Allocator argument_0, \
    MalType_UInt64 argument_1 \
)

#define MAL_HAS_EXTERN_releaseNode 1
#define MAL_DEFINE_releaseNode(context, argument_0, argument_1) \
void mal_ext_releaseNode( \
    MalContext *context MAL_DETAIL_MAYBE_UNUSED, \
    MalType_Allocator argument_0, \
    MalType_Ptr argument_1 \
)

#define MAL_HAS_EXTERN_destroyAllocator 1
#define MAL_DEFINE_destroyAllocator(context, value) \
void mal_ext_destroyAllocator( \
    MalContext *context MAL_DETAIL_MAYBE_UNUSED, \
    MalType_Allocator value \
)

#endif
