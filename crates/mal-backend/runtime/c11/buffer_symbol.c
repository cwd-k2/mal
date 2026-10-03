#include "buffer_internal.h"

// Both conversions take the caller's reference to their operand, which is at its last use. When nothing else holds the
// operand or its byte owner, the owner changes hands instead of being copied; otherwise the bytes are copied and the
// reference is dropped, which is what the borrowing conversion followed by the operand's release would do.

void *mal_runtime_buffer_into_symbol(MalContext *context, void *opaque_buffer) {
    MalBuffer *buffer = opaque_buffer;
    size_t count = buffer->count;
    MalBytes *buffer_owner = mal_buffer_owner(buffer);
    MalBytes *owner;
    if (count != 0
        && buffer_owner != NULL
        && mal_runtime_owner_is_unique(buffer)
        && buffer_owner->references == 1) {
        ((MalBytesFlat *)buffer_owner)->start = 0;
        owner = buffer_owner;
        owner->length = (uint64_t)count;
        buffer->storage.owner = NULL;
        buffer->data = NULL;
        buffer->count = 0;
    } else {
        owner = mal_bytes_flat_copy(context, buffer->data, count, "byte allocation failed");
    }
    mal_runtime_owner_release(buffer);
    return owner;
}

void *mal_runtime_symbol_into_buffer(
    MalContext *context,
    void *opaque_owner,
    const uint8_t *data,
    size_t length
) {
    MalBytes *owner = opaque_owner;
    if (length != 0 && owner != NULL && owner->kind == MAL_BYTES_FLAT && owner->references == 1) {
        MalBytesFlat *flat = (MalBytesFlat *)owner;
        // A Buffer's elements start at the owner's first byte, so only a view from that byte can be adopted.
        if (flat->start == 0 && data == flat->bytes) {
            return mal_buffer_adopt(context, flat, length);
        }
    }
    void *buffer = mal_runtime_buffer_from(context, data, 0, length, 1);
    mal_bytes_release(owner);
    return buffer;
}
