#ifndef MAL_H
#define MAL_H

#include <stddef.h>
#include <stdint.h>
#include <limits.h>
#include <float.h>
#include <string.h>

#define MAL_C_ABI_VERSION 0x000a00u

#if defined(__clang__)
#define MAL_DETAIL_MAYBE_UNUSED __attribute__((unused))
#else
#define MAL_DETAIL_MAYBE_UNUSED
#endif

/* Runtime API */

typedef struct { uint8_t *storage; size_t capacity; } MalControlArena;
typedef struct { MalControlArena control; uintptr_t native_stack_limit; } MalContext;
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
typedef struct { void *owner; const uint8_t *data; size_t length; } MalType_Symbol;
typedef void *MalType_Buffer;
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
typedef MalType_Symbol mal_Symbol_t;
typedef MalType_Buffer mal_Buffer_t;
typedef MalType_ByteSize mal_ByteSize_t;
typedef MalType_USize mal_USize_t;
typedef struct { MalContext *mal_detail_context; } mal_call_t;
typedef void (*MalRuntimeRetain)(MalContext *, void *);
typedef void (*MalRuntimeRelease)(void *);

#define mal_false (mal_Bool_t)UINT8_C(0)
#define mal_true (mal_Bool_t)UINT8_C(1)

_Noreturn void mal_trap(MalContext *context, const char *message);
void *mal_runtime_allocate(MalContext *context, size_t size);
void mal_runtime_deallocate(void *allocation);
void *mal_runtime_owner_retain(MalContext *context, void *owner);
void mal_runtime_owner_release(void *owner);
uint8_t mal_runtime_owner_is_unique(const void *owner);
const uint8_t *mal_runtime_bytes_data(const void *owner);
void *mal_runtime_bytes_read(MalContext *context, const void *source, size_t length);
void *mal_runtime_bytes_retain(MalContext *context, const void *owner);
void mal_runtime_bytes_release(const void *owner);
void *mal_runtime_buffer_make(MalContext *context, size_t stride, size_t capacity);
void *mal_runtime_buffer_make_managed(MalContext *context, size_t stride, size_t capacity, MalRuntimeRetain retain, MalRuntimeRelease release);
void *const *mal_runtime_buffer_data_slot(const void *buffer);
size_t mal_runtime_buffer_count(const void *buffer);
size_t mal_runtime_buffer_new(MalContext *context, void *buffer, const void *value, size_t stride);
size_t mal_runtime_buffer_new_managed_move(MalContext *context, void *buffer, const void *value, size_t stride);
static inline _Noreturn void mal_call_trap(mal_call_t *call, const char *message) {
    mal_trap(call->mal_detail_context, message);
}
static inline mal_Symbol_t mal_Symbol_from_bytes(mal_call_t *call, const void *source, size_t length) {
    void *owner = mal_runtime_bytes_read(call->mal_detail_context, source, length);
    return (mal_Symbol_t){ .owner = owner, .data = mal_runtime_bytes_data(owner), .length = length };
}
static inline mal_Symbol_t mal_Symbol_share(mal_call_t *call, mal_Symbol_t value) {
    mal_runtime_bytes_retain(call->mal_detail_context, value.owner);
    return value;
}
static inline void mal_Symbol_drop(mal_Symbol_t value) {
    mal_runtime_bytes_release(value.owner);
}
static inline mal_Buffer_t mal_Buffer_make(mal_call_t *call, size_t stride, size_t capacity) {
    return mal_runtime_buffer_make(call->mal_detail_context, stride, capacity);
}
static inline mal_Buffer_t mal_Buffer_share(mal_call_t *call, mal_Buffer_t value) {
    return mal_runtime_owner_retain(call->mal_detail_context, value);
}
static inline mal_Buffer_t mal_Buffer_make_managed(mal_call_t *call, size_t stride, size_t capacity, MalRuntimeRetain retain, MalRuntimeRelease release) {
    return mal_runtime_buffer_make_managed(call->mal_detail_context, stride, capacity, retain, release);
}
static inline void mal_Buffer_drop(mal_Buffer_t value) {
    mal_runtime_owner_release(value);
}
static inline void *mal_Buffer_data(mal_Buffer_t value) {
    return *mal_runtime_buffer_data_slot(value);
}
static inline size_t mal_Buffer_count(mal_Buffer_t value) {
    return mal_runtime_buffer_count(value);
}
static inline size_t mal_Buffer_new(mal_call_t *call, mal_Buffer_t value, const void *element, size_t stride) {
    return mal_runtime_buffer_new(call->mal_detail_context, value, element, stride);
}
static inline size_t mal_Buffer_new_managed_move(mal_call_t *call, mal_Buffer_t value, const void *element, size_t stride) {
    return mal_runtime_buffer_new_managed_move(call->mal_detail_context, value, element, stride);
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
static inline MalType_Symbol mal_Symbol_return_move(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, mal_Symbol_t value) {
    return value;
}
static inline MalType_Buffer mal_Buffer_return_move(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, mal_Buffer_t value) {
    return value;
}
static inline MalType_Bool mal_Bool_return(mal_call_t *call, mal_Bool_t value) {
    if ((value != mal_false) && (value != mal_true)) {
        mal_call_trap(call, "invalid Bool result");
    }
    return value;
}
static inline MalType_Unit mal_detail_convert_Unit(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, mal_Unit_t value MAL_DETAIL_MAYBE_UNUSED) {
    return (MalType_Unit){ 0 };
}
/* Generated header templates */

#define MAL_DETAIL_RAW_REPR_FIELD(context, index, member, raw_type, host_type, to_host, to_raw) \
raw_type member;
#define MAL_DETAIL_HOST_REPR_FIELD(context, index, member, raw_type, host_type, to_host, to_raw) \
host_type member;
#define MAL_DETAIL_DEFINE_PRODUCT_REPR(type_tag, fields, field) \
struct type_tag { \
    fields(field, type_tag) \
};
#define MAL_DETAIL_DEFINE_SUM_REPR(type_tag, members, member) \
struct type_tag { \
    uint32_t tag; \
    union { \
        members(member, type_tag) \
    } payload; \
};
#define MAL_DETAIL_DEFINE_EMPTY_SUM_REPR(type_tag) \
struct type_tag { \
    uint32_t tag; \
};
#define MAL_DETAIL_REPR_IDENTITY(call, value) value
#define MAL_DETAIL_PRODUCT_TO_HOST_FIELD(context, index, member, raw_type, host_type, to_host, to_raw) \
.member = to_host(call, value.member),
#define MAL_DETAIL_PRODUCT_TO_RAW_FIELD(context, index, member, raw_type, host_type, to_host, to_raw) \
.member = to_raw(call, value.member),
#define MAL_DETAIL_DEFINE_PRODUCT_CONVERSIONS(to_host_name, to_raw_name, raw_type, host_type, fields) \
static inline host_type to_host_name(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, raw_type value) { \
    return (host_type){ fields(MAL_DETAIL_PRODUCT_TO_HOST_FIELD, host_type) }; \
} \
static inline raw_type to_raw_name(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, host_type value) { \
    return (raw_type){ fields(MAL_DETAIL_PRODUCT_TO_RAW_FIELD, raw_type) }; \
}
#define MAL_DETAIL_DEFINE_CONVERTING_RETURN(function_name, result_type, value_type, converter) \
static inline result_type function_name(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, value_type value) { \
    return converter(call, value); \
}

#define MAL_DETAIL_DEFINE_CONVERSION(function_name, result_type, value_type, conversion) \
static inline result_type function_name(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, value_type value) { \
    return conversion; \
}
#define MAL_DETAIL_SUM_TO_HOST_CASE(result_type, variant_tag, member, raw_type, host_type, to_host, to_raw) \
case UINT32_C(variant_tag): { \
    return (result_type){ .tag = UINT32_C(variant_tag), .payload.member = to_host(call, value.payload.member) }; \
}
#define MAL_DETAIL_SUM_TO_RAW_CASE(result_type, variant_tag, member, raw_type, host_type, to_host, to_raw) \
case UINT32_C(variant_tag): { \
    return (result_type){ .tag = UINT32_C(variant_tag), .payload.member = to_raw(call, value.payload.member) }; \
}
#define MAL_DETAIL_DEFINE_SUM_CONVERSIONS(to_host_name, to_raw_name, raw_type, host_type, members) \
static inline host_type to_host_name(mal_call_t *call, raw_type value) { \
    switch (value.tag) { \
        members(MAL_DETAIL_SUM_TO_HOST_CASE, host_type) \
        default: { \
            mal_call_trap(call, "invalid sum tag"); \
        } \
    } \
} \
static inline raw_type to_raw_name(mal_call_t *call, host_type value) { \
    switch (value.tag) { \
        members(MAL_DETAIL_SUM_TO_RAW_CASE, raw_type) \
        default: { \
            mal_call_trap(call, "invalid sum tag"); \
        } \
    } \
}
#define MAL_DETAIL_DEFINE_SUM_UNIT_API(make_name, return_name, host_type, raw_type, tag_name, member, to_raw) \
static inline host_type make_name(void) { \
    return (host_type){ .tag = tag_name, .payload.member = (mal_Unit_t){ 0 } }; \
} \
static inline raw_type return_name(mal_call_t *call) { \
    return to_raw(call, make_name()); \
}
#define MAL_DETAIL_DEFINE_SUM_VALUE_API(make_name, return_name, host_type, raw_type, value_type, tag_name, member, to_raw) \
static inline host_type make_name(value_type value) { \
    return (host_type){ .tag = tag_name, .payload.member = value }; \
} \
static inline raw_type return_name(mal_call_t *call, value_type value) { \
    return to_raw(call, make_name(value)); \
}

#endif
