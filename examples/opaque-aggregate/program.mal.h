#ifndef MAL_PROGRAM_MAL_H
#define MAL_PROGRAM_MAL_H

#include <stdint.h>

#define MAL_C_ABI_VERSION 0x000500u

#if defined(__clang__) || defined(__GNUC__)
#define MAL_MAYBE_UNUSED __attribute__((unused))
#else
#define MAL_MAYBE_UNUSED
#endif

typedef struct MalContext MalContext;
typedef struct { uint8_t unused; } MalUnit;
typedef struct { const uint8_t *data; uint64_t length; } MalString;
typedef struct { uint8_t *address; } MalPtr;

_Noreturn void mal_trap(MalContext *context, const char *message);
MalString mal_string_copy(MalContext *context, const uint8_t *data, uint64_t length);

static inline MalPtr mal_ptr_from_address(uint8_t *address) { return (MalPtr){ .address = address }; }
static inline uint8_t *mal_ptr_address(MalPtr value) { return value.address; }

typedef struct { uintptr_t bits; } MalOpaque_Mem;

typedef struct MalProduct_0 MalProduct_0;
typedef struct MalSum_1 MalSum_1;

struct MalProduct_0 {
    MalOpaque_Mem field_0;
    uint64_t field_1;
};

struct MalSum_1 {
    uint32_t tag;
    union {
        MalUnit variant_0;
        MalProduct_0 variant_1;
    } payload;
};

static inline MalOpaque_Mem mal_Mem_from_bits(uintptr_t bits) { return (MalOpaque_Mem){ .bits = bits }; }
static inline uintptr_t mal_Mem_bits(MalOpaque_Mem value) { return value.bits; }

typedef MalSum_1 MalType_Response;

#define MAL_TAG_Response_0 UINT32_C(0)
#define MAL_TAG_Response_1 UINT32_C(1)
static inline uint32_t mal_tag_Response(MalType_Response value) { return value.tag; }
static inline uint8_t mal_is_Response_0(MalType_Response value) { return value.tag == MAL_TAG_Response_0; }
static inline MalType_Response mal_make_Response_0(void) { return (MalType_Response){ .tag = MAL_TAG_Response_0, .payload.variant_0 = { .unused = UINT8_C(0) } }; }
static inline uint8_t mal_is_Response_1(MalType_Response value) { return value.tag == MAL_TAG_Response_1; }
static inline MalType_Response mal_make_Response_1(MalOpaque_Mem value_0, uint64_t value_1) { return (MalType_Response){ .tag = MAL_TAG_Response_1, .payload.variant_1 = { .field_0 = value_0, .field_1 = value_1, } }; }
static inline MalOpaque_Mem mal_get_Response_1_0(MalContext *context, MalType_Response value) { if (!mal_is_Response_1(value)) mal_trap(context, "expected Response variant 1"); return value.payload.variant_1.field_0; }
static inline uint64_t mal_get_Response_1_1(MalContext *context, MalType_Response value) { if (!mal_is_Response_1(value)) mal_trap(context, "expected Response variant 1"); return value.payload.variant_1.field_1; }

MalOpaque_Mem mal_ext_allocate(MalContext *context, uint64_t value);
MalType_Response mal_ext_resize(MalContext *context, MalOpaque_Mem argument_0, uint64_t argument_1);
uint64_t mal_ext_handleBits(MalContext *context, MalOpaque_Mem value);

#define MAL_DEFINE_allocate(context, value) MalOpaque_Mem mal_ext_allocate(MalContext *context MAL_MAYBE_UNUSED, uint64_t value)
#define MAL_DEFINE_resize(context, argument_0, argument_1) MalType_Response mal_ext_resize(MalContext *context MAL_MAYBE_UNUSED, MalOpaque_Mem argument_0, uint64_t argument_1)
#define MAL_DEFINE_handleBits(context, value) uint64_t mal_ext_handleBits(MalContext *context MAL_MAYBE_UNUSED, MalOpaque_Mem value)

#endif
