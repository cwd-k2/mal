#ifndef MAL_H
#define MAL_H

#include <stddef.h>
#include <stdint.h>
#include <limits.h>
#include <float.h>
#include <string.h>

#define MAL_C_ABI_VERSION 0x000900u

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

/* Canonical scalar memory access */

static inline mal_Unit_t mal_detail_memory_read_Unit(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, const uint8_t *source MAL_DETAIL_MAYBE_UNUSED) {
    return (mal_Unit_t){ 0 };
}
static inline void mal_detail_memory_write_Unit(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, uint8_t *destination MAL_DETAIL_MAYBE_UNUSED, mal_Unit_t value) {
    (void)value;
}

static inline mal_Bool_t mal_detail_memory_read_Bool(mal_call_t *call, const uint8_t *source) {
    mal_Bool_t value;
    memcpy(&value, source, sizeof(value));
    return mal_Bool_return(call, value);
}

static inline void mal_detail_memory_write_Bool(mal_call_t *call, uint8_t *destination, mal_Bool_t value) {
    mal_Bool_return(call, value);
    memcpy(destination, &value, sizeof(value));
}

static inline mal_Int8_t mal_detail_memory_read_Int8(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, const uint8_t *source) {
    mal_Int8_t value;
    memcpy(&value, source, sizeof(value));
    return value;
}

static inline void mal_detail_memory_write_Int8(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, uint8_t *destination, mal_Int8_t value) {
    memcpy(destination, &value, sizeof(value));
}

static inline mal_Int16_t mal_detail_memory_read_Int16(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, const uint8_t *source) {
    mal_Int16_t value;
    memcpy(&value, source, sizeof(value));
    return value;
}

static inline void mal_detail_memory_write_Int16(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, uint8_t *destination, mal_Int16_t value) {
    memcpy(destination, &value, sizeof(value));
}

static inline mal_Int32_t mal_detail_memory_read_Int32(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, const uint8_t *source) {
    mal_Int32_t value;
    memcpy(&value, source, sizeof(value));
    return value;
}

static inline void mal_detail_memory_write_Int32(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, uint8_t *destination, mal_Int32_t value) {
    memcpy(destination, &value, sizeof(value));
}

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

static inline mal_UInt16_t mal_detail_memory_read_UInt16(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, const uint8_t *source) {
    mal_UInt16_t value;
    memcpy(&value, source, sizeof(value));
    return value;
}

static inline void mal_detail_memory_write_UInt16(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, uint8_t *destination, mal_UInt16_t value) {
    memcpy(destination, &value, sizeof(value));
}

static inline mal_UInt32_t mal_detail_memory_read_UInt32(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, const uint8_t *source) {
    mal_UInt32_t value;
    memcpy(&value, source, sizeof(value));
    return value;
}

static inline void mal_detail_memory_write_UInt32(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, uint8_t *destination, mal_UInt32_t value) {
    memcpy(destination, &value, sizeof(value));
}

static inline mal_UInt64_t mal_detail_memory_read_UInt64(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, const uint8_t *source) {
    mal_UInt64_t value;
    memcpy(&value, source, sizeof(value));
    return value;
}

static inline void mal_detail_memory_write_UInt64(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, uint8_t *destination, mal_UInt64_t value) {
    memcpy(destination, &value, sizeof(value));
}

static inline mal_Float32_t mal_detail_memory_read_Float32(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, const uint8_t *source) {
    mal_Float32_t value;
    memcpy(&value, source, sizeof(value));
    return value;
}

static inline void mal_detail_memory_write_Float32(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, uint8_t *destination, mal_Float32_t value) {
    memcpy(destination, &value, sizeof(value));
}

static inline mal_Float64_t mal_detail_memory_read_Float64(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, const uint8_t *source) {
    mal_Float64_t value;
    memcpy(&value, source, sizeof(value));
    return value;
}

static inline void mal_detail_memory_write_Float64(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, uint8_t *destination, mal_Float64_t value) {
    memcpy(destination, &value, sizeof(value));
}

static inline mal_Address_t mal_detail_memory_read_Address(mal_call_t *call, const uint8_t *source) {
    mal_Address_t value;
    memcpy(&value, source, sizeof(value));
    return mal_Address_return(call, value);
}

static inline void mal_detail_memory_write_Address(mal_call_t *call, uint8_t *destination, mal_Address_t value) {
    mal_Address_return(call, value);
    memcpy(destination, &value, sizeof(value));
}

static inline mal_ByteSize_t mal_detail_memory_read_ByteSize(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, const uint8_t *source) {
    mal_ByteSize_t value;
    memcpy(&value, source, sizeof(value));
    return value;
}

static inline void mal_detail_memory_write_ByteSize(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, uint8_t *destination, mal_ByteSize_t value) {
    memcpy(destination, &value, sizeof(value));
}

static inline mal_USize_t mal_detail_memory_read_USize(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, const uint8_t *source) {
    mal_USize_t value;
    memcpy(&value, source, sizeof(value));
    return value;
}

static inline void mal_detail_memory_write_USize(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, uint8_t *destination, mal_USize_t value) {
    memcpy(destination, &value, sizeof(value));
}

/* Generated header templates */

#define MAL_DETAIL_DEFINE_CONVERSION(function_name, result_type, value_type, conversion) \
static inline result_type function_name(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, value_type value) { \
    return conversion; \
}
#define MAL_DETAIL_DEFINE_MEMORY_ALIAS(read_name, write_name, value_type, stride, reader, writer) \
static inline value_type read_name(mal_call_t *call, mal_Address_t address, mal_USize_t index) { \
    mal_Address_return(call, address); \
    return reader(call, (const uint8_t *)address + (index * stride)); \
} \
static inline void write_name(mal_call_t *call, mal_Address_t address, mal_USize_t index, value_type value) { \
    mal_Address_return(call, address); \
    writer(call, (uint8_t *)address + (index * stride), value); \
}
#define MAL_DETAIL_MEMORY_PRODUCT_READ_UNIT(member) \
value.member = (mal_Unit_t){ 0 };
#define MAL_DETAIL_MEMORY_PRODUCT_READ_VALUE(member, reader, writer, offset) \
value.member = reader(call, source + offset);
#define MAL_DETAIL_MEMORY_PRODUCT_WRITE_UNIT(member) \
(void)value.member;
#define MAL_DETAIL_MEMORY_PRODUCT_WRITE_VALUE(member, reader, writer, offset) \
writer(call, destination + offset, value.member);
#define MAL_DETAIL_DEFINE_MEMORY_PRODUCT(read_name, write_name, value_type, fields) \
static inline value_type read_name(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, const uint8_t *source MAL_DETAIL_MAYBE_UNUSED) { \
    value_type value; \
    fields(MAL_DETAIL_MEMORY_PRODUCT_READ_UNIT, MAL_DETAIL_MEMORY_PRODUCT_READ_VALUE); \
    return value; \
} \
static inline void write_name(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, uint8_t *destination MAL_DETAIL_MAYBE_UNUSED, value_type value) { \
    fields(MAL_DETAIL_MEMORY_PRODUCT_WRITE_UNIT, MAL_DETAIL_MEMORY_PRODUCT_WRITE_VALUE); \
}
#define MAL_DETAIL_MEMORY_SUM_READ_CASE(value_type, tag_type, tag_writer, variant_tag, member, reader, writer, offset) \
case variant_tag: { \
    return (value_type){ .tag = UINT32_C(variant_tag), .payload.member = reader(call, source + offset) }; \
}
#define MAL_DETAIL_MEMORY_SUM_WRITE_CASE(value_type, tag_type, tag_writer, variant_tag, member, reader, writer, offset) \
case UINT32_C(variant_tag): { \
    tag_writer(call, destination, (tag_type)value.tag); \
    writer(call, destination + offset, value.payload.member); \
    return; \
}
#define MAL_DETAIL_DEFINE_MEMORY_SUM(read_name, write_name, value_type, tag_reader, members) \
static inline value_type read_name(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, const uint8_t *source MAL_DETAIL_MAYBE_UNUSED) { \
    switch (tag_reader(call, source)) { \
        members(MAL_DETAIL_MEMORY_SUM_READ_CASE) \
        default: { \
            mal_call_trap(call, "invalid canonical sum tag"); \
        } \
    } \
} \
static inline void write_name(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, uint8_t *destination MAL_DETAIL_MAYBE_UNUSED, value_type value) { \
    switch (value.tag) { \
        members(MAL_DETAIL_MEMORY_SUM_WRITE_CASE) \
        default: { \
            mal_call_trap(call, "invalid sum tag"); \
        } \
    } \
}

#endif
