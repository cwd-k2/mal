#ifndef MAL_PROGRAM_MAL_H
#define MAL_PROGRAM_MAL_H

#include <stddef.h>
#include <stdint.h>
#include <limits.h>
#include <float.h>
#include <string.h>

#define MAL_C_ABI_VERSION 0x000800u

#if defined(__clang__)
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
typedef size_t MalType_ByteSize;
typedef size_t MalType_USize;
typedef void *MalType_Address;
_Static_assert((sizeof((size_t)0) * CHAR_BIT) == 64, "size_t does not match the mal target pointer index width");
_Static_assert(((sizeof((float)0) * CHAR_BIT) == 32) && (FLT_MANT_DIG == 24), "float is not IEEE 754 binary32");
_Static_assert(((sizeof((double)0) * CHAR_BIT) == 64) && (DBL_MANT_DIG == 53), "double is not IEEE 754 binary64");
_Static_assert((FLT_HAS_SUBNORM == 1) && (DBL_HAS_SUBNORM == 1), "the target does not preserve subnormal floating-point values");
_Static_assert(FLT_EVAL_METHOD == 0, "floating-point expressions are evaluated with extra precision");
typedef MalType_Unit mal_Unit_t;
typedef MalType_Bool mal_Bool_t;
typedef MalType_Int8 mal_Int8_t;
typedef MalType_Int16 mal_Int16_t;
typedef MalType_Int32 mal_Int32_t;
typedef MalType_Int64 mal_Int64_t;
typedef MalType_UInt8 mal_UInt8_t;
typedef MalType_UInt16 mal_UInt16_t;
typedef MalType_UInt32 mal_UInt32_t;
typedef MalType_UInt64 mal_UInt64_t;
typedef MalType_Float32 mal_Float32_t;
typedef MalType_Float64 mal_Float64_t;
typedef MalType_Address mal_Address_t;
typedef MalType_ByteSize mal_ByteSize_t;
typedef MalType_USize mal_USize_t;
typedef struct { MalContext *mal_detail_context; } mal_call_t;

#define mal_false (mal_Bool_t)UINT8_C(0)
#define mal_true (mal_Bool_t)UINT8_C(1)

_Noreturn void mal_trap(MalContext *context, const char *message);
static inline _Noreturn void mal_call_trap(mal_call_t *call, const char *message) {
    mal_trap(call->mal_detail_context, message);
}
static inline MalType_Unit mal_Unit_return(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED) {
    return (MalType_Unit){ .unused = UINT8_C(0) };
}
static inline MalType_Int8 mal_Int8_return(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, mal_Int8_t value) {
    return value;
}
static inline MalType_Int16 mal_Int16_return(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, mal_Int16_t value) {
    return value;
}
static inline MalType_Int32 mal_Int32_return(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, mal_Int32_t value) {
    return value;
}
static inline MalType_Int64 mal_Int64_return(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, mal_Int64_t value) {
    return value;
}
static inline MalType_UInt8 mal_UInt8_return(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, mal_UInt8_t value) {
    return value;
}
static inline MalType_UInt16 mal_UInt16_return(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, mal_UInt16_t value) {
    return value;
}
static inline MalType_UInt32 mal_UInt32_return(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, mal_UInt32_t value) {
    return value;
}
static inline MalType_UInt64 mal_UInt64_return(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, mal_UInt64_t value) {
    return value;
}
static inline MalType_Float32 mal_Float32_return(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, mal_Float32_t value) {
    return value;
}
static inline MalType_Float64 mal_Float64_return(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, mal_Float64_t value) {
    return value;
}
static inline MalType_ByteSize mal_ByteSize_return(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, mal_ByteSize_t value) {
    return value;
}
static inline MalType_USize mal_USize_return(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, mal_USize_t value) {
    return value;
}
static inline MalType_Address mal_Address_return(mal_call_t *call, mal_Address_t value) {
    if (value == 0) {
        mal_call_trap(call, "invalid Address result");
    }
    return value;
}
static inline MalType_Bool mal_Bool_return(mal_call_t *call, mal_Bool_t value) {
    if ((value != mal_false) && (value != mal_true)) {
        mal_call_trap(call, "invalid Bool result");
    }
    return value;
}

/* Host-visible types */

typedef struct mal_detail_repr_product_0 mal_repr_product_0_t;
typedef mal_repr_product_0_t mal_SampleRecord_t;

struct mal_detail_repr_product_0 {
    mal_Int64_t field_0;
    mal_UInt8_t field_1;
};

/* Canonical memory access */

static inline mal_Int64_t mal_detail_memory_read_Int64(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, const uint8_t *source) {
    mal_Int64_t value;
    memcpy(&value, source, sizeof(value));
    return value;
}

static inline void mal_detail_memory_write_Int64(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, uint8_t *destination, mal_Int64_t value) {
    memcpy(destination, &value, sizeof(value));
}

static inline mal_UInt8_t mal_detail_memory_read_UInt8(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, const uint8_t *source) {
    mal_UInt8_t value;
    memcpy(&value, source, sizeof(value));
    return value;
}

static inline void mal_detail_memory_write_UInt8(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, uint8_t *destination, mal_UInt8_t value) {
    memcpy(destination, &value, sizeof(value));
}

static inline mal_repr_product_0_t mal_detail_memory_read_0(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, const uint8_t *source MAL_DETAIL_MAYBE_UNUSED) {
    mal_repr_product_0_t value;
    value.field_0 = mal_detail_memory_read_Int64(call, source + 0);
    value.field_1 = mal_detail_memory_read_UInt8(call, source + 8);
    return value;
}

static inline void mal_detail_memory_write_0(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, uint8_t *destination MAL_DETAIL_MAYBE_UNUSED, mal_repr_product_0_t value) {
    mal_detail_memory_write_Int64(call, destination + 0, value.field_0);
    mal_detail_memory_write_UInt8(call, destination + 8, value.field_1);
}

static inline mal_SampleRecord_t mal_SampleRecord_read(mal_call_t *call, mal_Address_t address, mal_USize_t index) {
    mal_Address_return(call, address);
    return mal_detail_memory_read_0(call, (const uint8_t *)address + (index * 16));
}

static inline void mal_SampleRecord_write(mal_call_t *call, mal_Address_t address, mal_USize_t index, mal_SampleRecord_t value) {
    mal_Address_return(call, address);
    mal_detail_memory_write_0(call, (uint8_t *)address + (index * 16), value);
}

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
