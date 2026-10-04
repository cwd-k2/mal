#ifndef MAL_H
#define MAL_H

#include <stddef.h>
#include <stdint.h>
#include <limits.h>
#include <float.h>
#include <string.h>

#define MAL_C_ABI_VERSION 0x000a00u

#define mal_type(name) MAL_DETAIL_NAMED_TYPE(name)
#define MAL_DETAIL_NAMED_TYPE(name) MAL_DETAIL_NAMED_TYPE_EXPAND(name)
#define MAL_DETAIL_NAMED_TYPE_EXPAND(name) mal_##name##_t
#define mal_product(...) __typeof__(*mal_detail_product_type((void (*)(__VA_ARGS__))0))
#define mal_sum(...) __typeof__(*mal_detail_sum_type((void (*)(__VA_ARGS__))0))

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
typedef void (*mal_detail_sum_key_bool_t)(mal_Unit_t, mal_Unit_t);
__attribute__((overloadable)) mal_Bool_t *mal_detail_sum_type(mal_detail_sum_key_bool_t);
typedef MalContext mal_call_t;
typedef void (*MalRuntimeRetain)(MalContext *, void *);
typedef void (*MalRuntimeRelease)(void *);
typedef struct { size_t size; size_t alignment; MalRuntimeRetain share; MalRuntimeRelease drop; } mal_storage_descriptor_t;

#define mal_false (mal_Bool_t)UINT8_C(0)
#define mal_true (mal_Bool_t)UINT8_C(1)
#define mal_unit (mal_Unit_t){ .unused = UINT8_C(0) }

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
size_t mal_runtime_buffer_new_move(MalContext *context, void *buffer, const void *value);
void mal_runtime_buffer_replace_move(MalContext *context, void *buffer, size_t index, const void *value);
void mal_runtime_buffer_fill_move(MalContext *context, void *buffer, size_t offset, size_t count, const void *value);
void mal_runtime_buffer_copy_values(MalContext *context, void *destination, size_t destination_offset, const void *source, size_t source_offset, size_t count);
void mal_runtime_buffer_append_values(MalContext *context, void *buffer, const void *source, size_t count);
void *mal_runtime_buffer_extend(MalContext *context, void *buffer, size_t count);
void mal_runtime_buffer_truncate(void *buffer, size_t count);
void mal_runtime_buffer_reserve_elements(MalContext *context, void *buffer, size_t capacity);
static inline _Noreturn void mal_call_trap(mal_call_t *call, const char *message) {
    mal_trap(call, message);
}
static inline mal_Symbol_t mal_detail_symbol(mal_call_t *call, const void *source, size_t length) {
    void *owner = mal_runtime_bytes_read(call, source, length);
    return (mal_Symbol_t){ .owner = owner, .data = mal_runtime_bytes_data(owner), .length = length };
}
static inline mal_Buffer_t mal_detail_buffer_make(mal_call_t *call, size_t stride, size_t capacity) {
    return mal_runtime_buffer_make(call, stride, capacity);
}
static inline mal_Buffer_t mal_detail_buffer_make_managed(mal_call_t *call, size_t stride, size_t capacity, MalRuntimeRetain retain, MalRuntimeRelease release) {
    return mal_runtime_buffer_make_managed(call, stride, capacity, retain, release);
}
static inline void *mal_detail_buffer_data(mal_Buffer_t value) {
    return *mal_runtime_buffer_data_slot(value);
}
static inline size_t mal_detail_buffer_count(mal_Buffer_t value) {
    return mal_runtime_buffer_count(value);
}
static inline __attribute__((overloadable)) void mal_detail_retain(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, void *value MAL_DETAIL_MAYBE_UNUSED) {
}
static inline __attribute__((overloadable)) void mal_detail_release(void *value MAL_DETAIL_MAYBE_UNUSED) {
}
static inline __attribute__((overloadable)) void mal_detail_retain(mal_call_t *call, mal_Symbol_t *value) {
    mal_runtime_bytes_retain(call, value->owner);
}
static inline __attribute__((overloadable)) void mal_detail_release(mal_Symbol_t *value) {
    mal_runtime_bytes_release(value->owner);
}
static inline __attribute__((overloadable)) void mal_detail_retain(mal_call_t *call, mal_Buffer_t *value) {
    mal_runtime_owner_retain(call, *value);
}
static inline __attribute__((overloadable)) void mal_detail_release(mal_Buffer_t *value) {
    mal_runtime_owner_release(*value);
}
static inline void mal_detail_symbol_storage_share(MalContext *context, void *carrier) {
    mal_detail_retain(context, (mal_Symbol_t *)carrier);
}
static inline void mal_detail_symbol_storage_drop(void *carrier) {
    mal_detail_release((mal_Symbol_t *)carrier);
}
static inline void mal_detail_buffer_storage_share(MalContext *context, void *carrier) {
    mal_detail_retain(context, (mal_Buffer_t *)carrier);
}
static inline void mal_detail_buffer_storage_drop(void *carrier) {
    mal_detail_release((mal_Buffer_t *)carrier);
}
static inline __attribute__((overloadable)) mal_storage_descriptor_t mal_detail_storage(void *type MAL_DETAIL_MAYBE_UNUSED, size_t size, size_t alignment) {
    return (mal_storage_descriptor_t){ .size = size, .alignment = alignment, .share = NULL, .drop = NULL };
}
static inline __attribute__((overloadable)) mal_storage_descriptor_t mal_detail_storage(mal_Symbol_t *type MAL_DETAIL_MAYBE_UNUSED, size_t size, size_t alignment) {
    return (mal_storage_descriptor_t){ .size = size, .alignment = alignment, .share = mal_detail_symbol_storage_share, .drop = mal_detail_symbol_storage_drop };
}
static inline __attribute__((overloadable)) mal_storage_descriptor_t mal_detail_storage(mal_Buffer_t *type MAL_DETAIL_MAYBE_UNUSED, size_t size, size_t alignment) {
    return (mal_storage_descriptor_t){ .size = size, .alignment = alignment, .share = mal_detail_buffer_storage_share, .drop = mal_detail_buffer_storage_drop };
}
static inline mal_Buffer_t mal_detail_buffer(mal_call_t *call, mal_storage_descriptor_t storage, size_t capacity) {
    if (storage.alignment > _Alignof(max_align_t)) {
        mal_call_trap(call, "unsupported Buffer element alignment");
    }
    if (storage.share != NULL) {
        return mal_detail_buffer_make_managed(call, storage.size, capacity, storage.share, storage.drop);
    }
    return mal_detail_buffer_make(call, storage.size, capacity);
}
static inline size_t mal_detail_buffer_push(mal_call_t *call, mal_Buffer_t buffer, const void *element) {
    return mal_runtime_buffer_new_move(call, buffer, element);
}
static inline mal_Symbol_t mal_detail_buffer_snapshot(mal_call_t *call, mal_Buffer_t buffer) {
    return mal_detail_symbol(call, mal_detail_buffer_data(buffer), mal_detail_buffer_count(buffer));
}
static inline void mal_detail_cleanup_Symbol(mal_Symbol_t *value) {
    mal_detail_release(value);
}
static inline void mal_detail_cleanup_Buffer(mal_Buffer_t *value) {
    mal_detail_release(value);
}
#define MAL_DETAIL_CLEANUP_Symbol mal_detail_cleanup_Symbol
#define MAL_DETAIL_CLEANUP_Buffer mal_detail_cleanup_Buffer
#define mal_storage(type) mal_detail_storage((type *)0, sizeof(type), _Alignof(type))
#define mal_share(call, value) __extension__ ({ \
    __auto_type mal_detail_shared = (value); \
    mal_detail_retain((call), &mal_detail_shared); \
    mal_detail_shared; \
})
#define mal_move(value) __extension__ ({ \
    __auto_type *mal_detail_source = &(value); \
    __auto_type mal_detail_moved = *mal_detail_source; \
    memset(mal_detail_source, 0, sizeof(*mal_detail_source)); \
    mal_detail_moved; \
})
#define mal_drop(value) ((void)__extension__ ({ \
    __auto_type *mal_detail_dropped = &(value); \
    mal_detail_release(mal_detail_dropped); \
    memset(mal_detail_dropped, 0, sizeof(*mal_detail_dropped)); \
}))
#define mal_owned(name) mal_type(name) __attribute__((cleanup(MAL_DETAIL_CLEANUP(name))))
#define MAL_DETAIL_CLEANUP(name) MAL_DETAIL_CLEANUP_EXPAND(name)
#define MAL_DETAIL_CLEANUP_EXPAND(name) MAL_DETAIL_CLEANUP_##name
#define mal_symbol(call, source, length) mal_detail_symbol(call, source, length)
#define mal_buffer(call, element_type, capacity) mal_detail_buffer(call, mal_storage(element_type), capacity)
#define mal_data(buffer) mal_detail_buffer_data(buffer)
#define mal_count(buffer) mal_detail_buffer_count(buffer)
#define mal_from_bits(type, raw_bits) (type){ .bits = (uintptr_t)(raw_bits) }
#define mal_bits(value) (value).bits
#define mal_push(call, buffer, element) __extension__ ({ \
    __auto_type mal_detail_element = (element); \
    mal_detail_buffer_push((call), (buffer), &mal_detail_element); \
})
#define mal_replace(call, buffer, index, element) __extension__ ({ \
    __auto_type mal_detail_element = (element); \
    mal_runtime_buffer_replace_move((call), (buffer), (index), &mal_detail_element); \
})
#define mal_fill(call, buffer, offset, count, element) __extension__ ({ \
    __auto_type mal_detail_element = (element); \
    mal_runtime_buffer_fill_move((call), (buffer), (offset), (count), &mal_detail_element); \
})
#define mal_copy(call, destination, destination_offset, source, source_offset, count) mal_runtime_buffer_copy_values(call, destination, destination_offset, source, source_offset, count)
#define mal_append(call, buffer, source, count) mal_runtime_buffer_append_values(call, buffer, source, count)
#define mal_extend(call, buffer, count) mal_runtime_buffer_extend(call, buffer, count)
#define mal_truncate(buffer, count) mal_runtime_buffer_truncate(buffer, count)
#define mal_reserve(call, buffer, capacity) mal_runtime_buffer_reserve_elements(call, buffer, capacity)
#define mal_snapshot(call, buffer) mal_detail_buffer_snapshot(call, buffer)

/* Generated header templates */

#define MAL_DETAIL_REPR_FIELD(context, index, member, type) \
type member;
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

#define MAL_DETAIL_PRODUCT_RETAIN_FIELD(context, index, member, type) mal_detail_retain(call, &value->member);
#define MAL_DETAIL_PRODUCT_RELEASE_FIELD(context, index, member, type) mal_detail_release(&value->member);
#define MAL_DETAIL_SUM_RETAIN_CASE(context, index, member, type) case UINT32_C(index): { mal_detail_retain(call, &value->payload.member); return; }
#define MAL_DETAIL_SUM_RELEASE_CASE(context, index, member, type) case UINT32_C(index): { mal_detail_release(&value->payload.member); return; }
#define MAL_DETAIL_DEFINE_PRODUCT_LIFECYCLE(host_type, fields, share_name, drop_name, cleanup_name) \
static inline __attribute__((overloadable)) void mal_detail_retain(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, host_type *value MAL_DETAIL_MAYBE_UNUSED) { fields(MAL_DETAIL_PRODUCT_RETAIN_FIELD, host_type) } \
static inline __attribute__((overloadable)) void mal_detail_release(host_type *value MAL_DETAIL_MAYBE_UNUSED) { fields(MAL_DETAIL_PRODUCT_RELEASE_FIELD, host_type) } \
static inline void share_name(MalContext *context, void *carrier) { mal_detail_retain(context, (host_type *)carrier); } \
static inline void drop_name(void *carrier) { mal_detail_release((host_type *)carrier); } \
static inline void cleanup_name(host_type *value) { mal_detail_release(value); } \
static inline __attribute__((overloadable)) mal_storage_descriptor_t mal_detail_storage(host_type *type_marker MAL_DETAIL_MAYBE_UNUSED, size_t size, size_t alignment) { return (mal_storage_descriptor_t){ .size = size, .alignment = alignment, .share = share_name, .drop = drop_name }; }
#define MAL_DETAIL_DEFINE_SUM_LIFECYCLE(host_type, fields, share_name, drop_name, cleanup_name) \
static inline __attribute__((overloadable)) void mal_detail_retain(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, host_type *value MAL_DETAIL_MAYBE_UNUSED) { switch (value->tag) { fields(MAL_DETAIL_SUM_RETAIN_CASE, host_type) default: return; } } \
static inline __attribute__((overloadable)) void mal_detail_release(host_type *value MAL_DETAIL_MAYBE_UNUSED) { switch (value->tag) { fields(MAL_DETAIL_SUM_RELEASE_CASE, host_type) default: return; } } \
static inline void share_name(MalContext *context, void *carrier) { mal_detail_retain(context, (host_type *)carrier); } \
static inline void drop_name(void *carrier) { mal_detail_release((host_type *)carrier); } \
static inline void cleanup_name(host_type *value) { mal_detail_release(value); } \
static inline __attribute__((overloadable)) mal_storage_descriptor_t mal_detail_storage(host_type *type_marker MAL_DETAIL_MAYBE_UNUSED, size_t size, size_t alignment) { return (mal_storage_descriptor_t){ .size = size, .alignment = alignment, .share = share_name, .drop = drop_name }; }

#endif
