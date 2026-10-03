#ifndef MAL_BUFFER_INTERNAL_H
#define MAL_BUFFER_INTERNAL_H

#include "bytes_internal.h"

typedef struct {
    union {
        MalBytes *owner;
        unsigned char inline_bytes[sizeof(MalBytes *)];
    } storage;
    unsigned char *data;
    size_t count;
    size_t stride;
} MalBuffer;

static inline int mal_buffer_is_inline(const MalBuffer *buffer) {
    return buffer->data == buffer->storage.inline_bytes;
}

static inline MalBytes *mal_buffer_owner(const MalBuffer *buffer) {
    return mal_buffer_is_inline(buffer) ? NULL : buffer->storage.owner;
}

static inline size_t mal_buffer_zeroed_until(const MalBuffer *buffer) {
    if (mal_buffer_is_inline(buffer)) {
        return sizeof buffer->storage.inline_bytes;
    }
    MalBytes *owner = mal_buffer_owner(buffer);
    return owner == NULL ? 0 : ((const MalBytesFlat *)owner)->zeroed_until;
}

// A buffer whose elements own managed values. The plain `MalBuffer` stays first so that every operation that does
// not touch element ownership treats both kinds alike; `retain` and `release` act on one stored element in place.
typedef struct {
    MalBuffer buffer;
    void (*retain)(MalContext *, void *element);
    void (*release)(void *element);
} MalManagedBuffer;

// The byte size of `count` elements of `stride` bytes; traps on overflow.
size_t mal_buffer_bytes(MalContext *context, size_t count, size_t stride);
// A byte Buffer of `count` elements whose storage is `flat`, which it now owns. Bytes past `count` are not assumed zero.
void *mal_buffer_adopt(MalContext *context, MalBytesFlat *flat, size_t count);
// Ensures storage for `required` bytes and records that logical byte length. Inline storage is promoted to a flat owner
// when it must grow. The caller must update the element count after initializing any newly live elements.
void mal_buffer_reserve(MalContext *context, MalBuffer *buffer, size_t required);

#endif
