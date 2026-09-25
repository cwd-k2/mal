#include "buffer_internal.h"

#include <stdint.h>
#include <string.h>

// Checks a fill of `[offset, offset + count)`, grows the storage to hold it, and sets the new count. Returns whether any
// element is written; `old_count` receives the count before the call.
static int mal_buffer_fill_extend(
    MalContext *context,
    MalBuffer *buffer,
    size_t offset,
    size_t count,
    size_t stride,
    size_t *old_count
) {
    *old_count = buffer->count;
    if (offset > *old_count) {
        mal_trap(context, "buffer fill offset out of bounds");
    }
    if (count > SIZE_MAX - offset) {
        mal_trap(context, "buffer fill range overflow");
    }
    size_t end = offset + count;
    size_t new_count = *old_count > end ? *old_count : end;
    if (count == 0) {
        return 0;
    }
    if (stride == 0) {
        buffer->count = new_count;
        return 0;
    }

    size_t required = mal_buffer_bytes(context, new_count, stride);
    MalBytesFlat *flat = (MalBytesFlat *)buffer->owner;
    if (flat == NULL || required > flat->capacity) {
        flat = mal_buffer_grow_unique(context, flat, required);
    } else {
        flat->header.length = (uint64_t)required;
    }
    buffer->owner = &flat->header;
    buffer->data = flat->bytes;
    buffer->count = new_count;
    return 1;
}

void mal_runtime_buffer_fill_managed(
    MalContext *context,
    void *opaque_buffer,
    size_t offset,
    size_t count,
    const void *value,
    size_t stride
) {
    MalManagedBuffer *managed = opaque_buffer;
    size_t old_count;
    if (!mal_buffer_fill_extend(context, &managed->buffer, offset, count, stride, &old_count)) {
        return;
    }
    // Elements below `old_count` held a reference that the write drops.
    for (size_t index = offset; index < offset + count; ++index) {
        unsigned char *element = managed->buffer.data + index * stride;
        if (index < old_count) {
            managed->release(element);
        }
        memcpy(element, value, stride);
        managed->retain(context, element);
    }
}

void mal_runtime_buffer_fill(
    MalContext *context,
    void *opaque_buffer,
    size_t offset,
    size_t count,
    const void *value,
    size_t stride
) {
    MalBuffer *buffer = opaque_buffer;
    size_t old_count;
    if (!mal_buffer_fill_extend(context, buffer, offset, count, stride, &old_count)) {
        return;
    }
    size_t end = offset + count;
    MalBytesFlat *flat = (MalBytesFlat *)buffer->owner;

    size_t start_byte = offset * stride;
    size_t byte_count = count * stride;
    const unsigned char *value_bytes = value;
    int value_is_zero = 1;
    for (size_t byte = 0; byte < stride; ++byte) {
        if (value_bytes[byte] != 0) {
            value_is_zero = 0;
            break;
        }
    }
    if (value_is_zero) {
        size_t existing_end = old_count < end ? old_count : end;
        size_t existing_count = existing_end - offset;
        if (existing_count != 0) {
            memset(flat->bytes + start_byte, 0, existing_count * stride);
        }
        size_t existing_bytes = existing_count * stride;
        size_t unwritten_start = start_byte + existing_bytes;
        size_t range_end = start_byte + byte_count;
        if (range_end > buffer->zeroed_until) {
            size_t zero_start = unwritten_start > buffer->zeroed_until
                ? unwritten_start
                : buffer->zeroed_until;
            if (zero_start < range_end) {
                memset(flat->bytes + zero_start, 0, range_end - zero_start);
            }
        }
        return;
    }
    if (stride == 1) {
        memset(flat->bytes + start_byte, value_bytes[0], count);
        return;
    }
    unsigned char *destination = flat->bytes + start_byte;
    memcpy(destination, value, stride);
    size_t initialized = stride;
    while (initialized < byte_count) {
        size_t remaining = byte_count - initialized;
        size_t chunk = initialized < remaining ? initialized : remaining;
        memcpy(destination + initialized, destination, chunk);
        initialized += chunk;
    }
}

// Checks a copy of `count` elements, grows the destination to hold them, and sets its new count. Returns whether any
// element is written; `old_count` receives the destination count before the call.
static int mal_buffer_copy_extend(
    MalContext *context,
    MalBuffer *destination,
    size_t destination_offset,
    const MalBuffer *source,
    size_t source_offset,
    size_t count,
    size_t stride,
    size_t *old_count
) {
    *old_count = destination->count;
    if (destination_offset > destination->count) {
        mal_trap(context, "buffer copy destination offset out of bounds");
    }
    if (source_offset > source->count || count > source->count - source_offset) {
        mal_trap(context, "buffer copy source range out of bounds");
    }
    if (count > SIZE_MAX - destination_offset) {
        mal_trap(context, "buffer copy destination range overflow");
    }
    size_t destination_end = destination_offset + count;
    size_t new_count = destination->count > destination_end
        ? destination->count
        : destination_end;
    if (count == 0) {
        return 0;
    }
    if (stride == 0) {
        destination->count = new_count;
        return 0;
    }

    size_t required = mal_buffer_bytes(context, new_count, stride);
    MalBytesFlat *flat = (MalBytesFlat *)destination->owner;
    if (flat == NULL || required > flat->capacity) {
        flat = mal_buffer_grow_unique(context, flat, required);
    } else {
        flat->header.length = (uint64_t)required;
    }
    destination->owner = &flat->header;
    destination->data = flat->bytes;
    destination->count = new_count;
    return 1;
}

// Every source reference is taken before any destination one is dropped: the ranges may overlap, so a dropped element
// can also be a source.
void mal_runtime_buffer_copy_managed(
    MalContext *context,
    void *opaque_destination,
    size_t destination_offset,
    const void *opaque_source,
    size_t source_offset,
    size_t count,
    size_t stride
) {
    MalManagedBuffer *destination = opaque_destination;
    const MalBuffer *source = opaque_source;
    size_t old_count;
    if (!mal_buffer_copy_extend(
            context,
            &destination->buffer,
            destination_offset,
            source,
            source_offset,
            count,
            stride,
            &old_count
        )) {
        return;
    }
    for (size_t index = 0; index < count; ++index) {
        destination->retain(
            context,
            (void *)(source->data + (source_offset + index) * stride)
        );
    }
    for (size_t index = destination_offset;
         index < destination_offset + count && index < old_count;
         ++index) {
        destination->release(destination->buffer.data + index * stride);
    }
    memmove(
        destination->buffer.data + destination_offset * stride,
        source->data + source_offset * stride,
        count * stride
    );
}

void mal_runtime_buffer_copy(
    MalContext *context,
    void *opaque_destination,
    size_t destination_offset,
    const void *opaque_source,
    size_t source_offset,
    size_t count,
    size_t stride
) {
    MalBuffer *destination = opaque_destination;
    const MalBuffer *source = opaque_source;
    size_t old_count;
    if (!mal_buffer_copy_extend(
            context,
            destination,
            destination_offset,
            source,
            source_offset,
            count,
            stride,
            &old_count
        )) {
        return;
    }
    memmove(
        destination->data + destination_offset * stride,
        source->data + source_offset * stride,
        count * stride
    );
}
