#ifndef MAL_BUFFER_INTERNAL_H
#define MAL_BUFFER_INTERNAL_H

#include "bytes_internal.h"

typedef struct {
    MalBytes *owner;
    unsigned char *data;
    size_t count;
    size_t stride;
    // Bytes between the logical end and this boundary retain their calloc zero.
    size_t zeroed_until;
} MalBuffer;

// A buffer whose elements own managed values. The plain `MalBuffer` stays first so that every operation that does
// not touch element ownership treats both kinds alike; `retain` and `release` act on one stored element in place.
typedef struct {
    MalBuffer buffer;
    void (*retain)(MalContext *, void *element);
    void (*release)(void *element);
} MalManagedBuffer;

// The byte size of `count` elements of `stride` bytes; traps on overflow.
size_t mal_buffer_bytes(MalContext *context, size_t count, size_t stride);
// Grows storage that only one buffer owns to hold `required` bytes and sets its length; `flat` may be NULL.
MalBytesFlat *mal_buffer_grow_unique(MalContext *context, MalBytesFlat *flat, size_t required);

#endif
