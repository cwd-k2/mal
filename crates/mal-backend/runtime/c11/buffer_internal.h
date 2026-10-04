#ifndef MAL_BUFFER_INTERNAL_H
#define MAL_BUFFER_INTERNAL_H

#include "bytes_internal.h"

typedef struct {
    union {
        MalBytes *owner;
        _Alignas(max_align_t) unsigned char inline_bytes[sizeof(MalBytes *)];
    } storage;
    unsigned char *data;
    size_t count;
    size_t stride;
    MalRuntimeRetain retain;
    MalRuntimeRelease release;
} MalBuffer;

static inline int mal_buffer_is_inline(const MalBuffer *buffer) {
    return buffer->data == buffer->storage.inline_bytes;
}

static inline MalBytes *mal_buffer_owner(const MalBuffer *buffer) {
    return mal_buffer_is_inline(buffer) ? NULL : buffer->storage.owner;
}

// The byte size of `count` elements of `stride` bytes; traps on overflow.
size_t mal_buffer_bytes(MalContext *context, size_t count, size_t stride);
// A byte Buffer of `count` elements whose storage is `flat`, which it now owns. Bytes past `count` are not assumed zero.
void *mal_buffer_adopt(MalContext *context, MalBytesFlat *flat, size_t count);
// Ensures storage for `required` bytes and records that logical byte length. Inline storage is promoted to a flat owner
// when it must grow. The caller must update the element count after initializing any newly live elements.
void mal_buffer_reserve(MalContext *context, MalBuffer *buffer, size_t required);

#endif
