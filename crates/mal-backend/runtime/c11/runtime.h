#ifndef MAL_RUNTIME_H
#define MAL_RUNTIME_H

#include <stddef.h>
#include <stdint.h>

/* Internal ABI between generated LLVM code and the runtime. A context belongs to one thread. Every function that
 * allocates traps on failure and on size overflow instead of returning an error. */

/* Growable storage for suspended calls of recursive regions. */
typedef struct {
    unsigned char *storage;
    size_t capacity;
} MalControlArena;

typedef struct MalContext {
    MalControlArena control;
} MalContext;

/* A window onto bytes kept alive by `owner`. A NULL owner or an immortal static owner needs no reference counting. */
typedef struct {
    void *owner;
    const unsigned char *data;
    size_t length;
} MalBytesView;

/* Prints "mal trap: <message>" to stderr and aborts. */
_Noreturn void mal_trap(MalContext *context, const char *message);
/* malloc that traps on failure. A size of zero allocates nothing and returns NULL. */
void *mal_runtime_allocate(MalContext *context, size_t size);
void mal_runtime_deallocate(void *allocation);
/* Allocates a reference-counted closure environment of `size` bytes with one reference. `destroy` releases what the
 * environment holds and runs when the last reference is released, before the storage is freed. NULL and pointers with the
 * lowest bit set denote environments without storage: retain and release ignore them and is_unique is false. */
void *mal_runtime_environment_allocate(
    MalContext *context,
    size_t size,
    void (*destroy)(void *)
);
/* Environment storage without a reference count for a closure that does not escape; the caller frees it. */
void *mal_runtime_scoped_environment_allocate(MalContext *context, size_t size);
void mal_runtime_scoped_environment_deallocate(void *environment);
void *mal_runtime_environment_retain(MalContext *context, void *environment);
void mal_runtime_environment_release(void *environment);
uint8_t mal_runtime_environment_is_unique(const void *environment);
/* The control arena is grown by reserve_frame and freed here. */
void mal_control_destroy(MalContext *context);
/* Makes room for `frame_size` bytes above `current_bytes` and returns the storage base. Growth can move the storage, so
 * callers derive frame addresses from the returned base and reload it after anything that may call back here. */
void *mal_control_reserve_frame(
    MalContext *context,
    size_t current_bytes,
    size_t frame_size
);
void *mal_control_storage(MalContext *context);
size_t mal_control_capacity(MalContext *context);

/* Byte owners: reference-counted, immutable once shared. Static owners emitted for literals are immortal. */
const uint8_t *mal_runtime_bytes_data(const void *owner);
/* Copies `length` bytes of host memory into a new owner with one reference. */
void *mal_runtime_bytes_read(MalContext *context, const void *source, size_t length);
void *mal_runtime_bytes_retain(MalContext *context, const void *owner);
void mal_runtime_bytes_release(const void *owner);
/* Symbol operations. The index must be below the length; that precondition is not checked. */
uint8_t mal_runtime_symbol_at(const void *data, size_t index);
/* Concatenation borrows both operands. The result view owns fresh storage, or shares the other operand's owner when one
 * side is empty. The consuming variants take over one operand's reference and may extend its storage in place when
 * nothing else shares it, so the caller must not use the consumed owner afterwards. */
void mal_runtime_symbol_concatenate(
    MalContext *context,
    MalBytesView *result,
    const void *left_owner,
    const void *left_data,
    size_t left_length,
    const void *right_owner,
    const void *right_data,
    size_t right_length
);
void mal_runtime_symbol_concatenate_consuming_left(
    MalContext *context,
    MalBytesView *result,
    void *left_owner,
    const void *left_data,
    size_t left_length,
    const void *right_owner,
    const void *right_data,
    size_t right_length
);
void mal_runtime_symbol_concatenate_consuming_right(
    MalContext *context,
    MalBytesView *result,
    const void *left_owner,
    const void *left_data,
    size_t left_length,
    void *right_owner,
    const void *right_data,
    size_t right_length
);
uint8_t mal_runtime_symbol_equal(
    const void *left_data,
    size_t left_length,
    const void *right_data,
    size_t right_length
);
/* Copies `length` bytes of the owner's data, starting at `offset`, out to host memory. */
void mal_runtime_bytes_write(
    void *destination,
    const void *owner,
    size_t offset,
    size_t length
);
/* Buffers are shared mutable sequences addressed by pointer. `make` returns an empty buffer whose storage is preallocated for
 * `capacity` elements of `stride` bytes; capacity is not the count. `new` appends a copy of `value` and returns its index.
 * `fill` and `copy` overwrite a range and extend the count when the range ends past it. Any of these can move the element
 * storage. */
void *mal_runtime_buffer_make(
    MalContext *context,
    size_t stride,
    size_t capacity
);
/* Like `make`, for elements that own managed values. `retain` and `release` act on one stored element in place: the
 * buffer takes a reference for every element it writes and drops it when the element is overwritten or the buffer dies. */
void *mal_runtime_buffer_make_managed(
    MalContext *context,
    size_t stride,
    size_t capacity,
    void (*retain)(MalContext *, void *element),
    void (*release)(void *element)
);
size_t mal_runtime_buffer_new(
    MalContext *context,
    void *buffer,
    const void *value,
    size_t stride
);
void mal_runtime_buffer_fill(
    MalContext *context,
    void *buffer,
    size_t offset,
    size_t count,
    const void *value,
    size_t stride
);
void mal_runtime_buffer_copy(
    MalContext *context,
    void *destination,
    size_t destination_offset,
    const void *source,
    size_t source_offset,
    size_t count,
    size_t stride
);
/* The address of the element storage pointer, for generated code that reloads it after an operation that may grow the buffer. */
void *const *mal_runtime_buffer_data_slot(const void *buffer);
size_t mal_runtime_buffer_count(const void *buffer);
/* Host memory holds canonical element bytes. `from` copies `count` elements starting at element `offset` into a new buffer;
 * `into` copies them the other way and leaves the buffer unchanged. */
void *mal_runtime_buffer_from(
    MalContext *context,
    const void *source,
    size_t offset,
    size_t count,
    size_t stride
);
/* Builds a buffer of `(address, length)` elements from NUL-terminated strings. The address is the first field of an element
 * of `stride` bytes and the length, without the terminator, lies at `length_offset`. */
void *mal_runtime_buffer_from_strings(
    MalContext *context,
    char *const *strings,
    size_t count,
    size_t stride,
    size_t length_offset
);
void mal_runtime_buffer_into(
    MalContext *context,
    const void *buffer,
    void *destination,
    size_t offset,
    size_t count,
    size_t stride
);

#endif
