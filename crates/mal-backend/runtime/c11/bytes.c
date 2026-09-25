#include "bytes_internal.h"

#include <limits.h>
#include <stdlib.h>
#include <string.h>

_Noreturn void mal_trap(MalContext *context, const char *message);

static size_t mal_bytes_capacity(size_t required);

static void *mal_bytes_allocate(
    MalContext *context,
    size_t size,
    const char *allocation_failure
) {
    void *allocation = malloc(size);
    if (allocation == NULL) {
        mal_trap(context, allocation_failure);
    }
    return allocation;
}

static MalBytesFlat *mal_bytes_flat_allocate(
    MalContext *context,
    size_t length,
    size_t capacity,
    size_t start,
    const char *allocation_failure
) {
    if (capacity > SIZE_MAX - sizeof(MalBytesFlat)
        || length > capacity
        || start > capacity - length) {
        mal_trap(context, "byte owner allocation size overflow");
    }
    MalBytesFlat *flat = mal_bytes_allocate(
        context,
        sizeof(MalBytesFlat) + capacity,
        allocation_failure
    );
    flat->header = (MalBytes){1, (uint64_t)length, MAL_BYTES_FLAT, {0}};
    flat->capacity = capacity;
    flat->start = start;
    return flat;
}

static MalBytesFlat *mal_bytes_flat_allocate_zeroed(
    MalContext *context,
    size_t length,
    size_t capacity,
    const char *allocation_failure
) {
    if (capacity > SIZE_MAX - sizeof(MalBytesFlat) || length > capacity) {
        mal_trap(context, "byte owner allocation size overflow");
    }
    MalBytesFlat *flat = calloc(1, sizeof(MalBytesFlat) + capacity);
    if (flat == NULL) {
        mal_trap(context, allocation_failure);
    }
    flat->header = (MalBytes){1, (uint64_t)length, MAL_BYTES_FLAT, {0}};
    flat->capacity = capacity;
    return flat;
}

MalBytes *mal_bytes_flat_copy(
    MalContext *context,
    const void *source,
    size_t length,
    const char *allocation_failure
) {
    if (length == 0) {
        return NULL;
    }
    MalBytesFlat *result = mal_bytes_flat_allocate(
        context,
        length,
        mal_bytes_capacity(length),
        0,
        allocation_failure
    );
    memcpy(result->bytes, source, length);
    return &result->header;
}

MalBytes *mal_bytes_flat_concatenate(
    MalContext *context,
    const unsigned char *left,
    size_t left_length,
    const unsigned char *right,
    size_t right_length,
    const char *allocation_failure
) {
    if (right_length > SIZE_MAX - left_length) {
        mal_trap(context, "byte owner allocation size overflow");
    }
    size_t length = left_length + right_length;
    MalBytesFlat *result = mal_bytes_flat_allocate(
        context,
        length,
        mal_bytes_capacity(length),
        0,
        allocation_failure
    );
    memcpy(result->bytes, left, left_length);
    memcpy(result->bytes + left_length, right, right_length);
    return &result->header;
}

MalBytes *mal_bytes_retain(MalContext *context, MalBytes *owner) {
    if (owner == NULL || owner->references == UINT64_MAX) {
        return owner;
    }
    if (owner->references == UINT64_MAX - 1) {
        mal_trap(context, "byte owner reference count overflow");
    }
    ++owner->references;
    return owner;
}

void mal_bytes_release(MalBytes *owner) {
    if (owner == NULL || owner->references == UINT64_MAX) {
        return;
    }
    --owner->references;
    if (owner->references == 0) {
        free(owner);
    }
}

const unsigned char *mal_bytes_data(const MalBytes *owner) {
    if (owner == NULL) {
        return NULL;
    }
    if (owner->kind == MAL_BYTES_STATIC) {
        return ((const MalBytesStatic *)owner)->bytes;
    }
    const MalBytesFlat *flat = (const MalBytesFlat *)owner;
    return flat->bytes + flat->start;
}

size_t mal_bytes_offset(const MalBytes *owner, const unsigned char *data) {
    if (owner == NULL) {
        return 0;
    }
    return (size_t)(data - mal_bytes_data(owner));
}

static size_t mal_bytes_capacity(size_t required) {
    if (required <= 16) {
        return 16;
    }
    size_t highest_power = SIZE_MAX - SIZE_MAX / 2;
    if (required > highest_power) {
        return required;
    }
    unsigned int shift = (unsigned int)(sizeof(unsigned long long) * CHAR_BIT)
        - (unsigned int)__builtin_clzll((unsigned long long)(required - 1));
    return (size_t)1 << shift;
}

MalBytes *mal_bytes_append(
    MalContext *context,
    MalBytes *owner,
    size_t offset,
    size_t length,
    const unsigned char *suffix,
    size_t suffix_length,
    const char *allocation_failure
) {
    if (suffix_length > SIZE_MAX - length) {
        mal_trap(context, "byte owner allocation size overflow");
    }
    size_t result_length = length + suffix_length;
    if (owner == NULL || owner->kind != MAL_BYTES_FLAT || owner->references != 1
        || offset != 0 || owner->length != (uint64_t)length) {
        size_t capacity = mal_bytes_capacity(result_length);
        MalBytesFlat *result = mal_bytes_flat_allocate(
            context,
            result_length,
            capacity,
            0,
            allocation_failure
        );
        if (length != 0) {
            memcpy(result->bytes, mal_bytes_data(owner) + offset, length);
        }
        memcpy(result->bytes + length, suffix, suffix_length);
        mal_bytes_release(owner);
        return &result->header;
    }
    MalBytesFlat *flat = (MalBytesFlat *)owner;
    if (flat->start > SIZE_MAX - result_length) {
        mal_trap(context, "byte owner allocation size overflow");
    }
    size_t required = flat->start + result_length;
    if (required > flat->capacity) {
        size_t capacity = mal_bytes_capacity(required);
        if (capacity > SIZE_MAX - sizeof(MalBytesFlat)) {
            mal_trap(context, "byte owner allocation size overflow");
        }
        flat = realloc(flat, sizeof(MalBytesFlat) + capacity);
        if (flat == NULL) {
            mal_trap(context, allocation_failure);
        }
        flat->capacity = capacity;
    }
    memcpy(flat->bytes + flat->start + length, suffix, suffix_length);
    flat->header.length = (uint64_t)result_length;
    return &flat->header;
}

MalBytes *mal_bytes_prepend(
    MalContext *context,
    MalBytes *owner,
    size_t offset,
    size_t length,
    const unsigned char *prefix,
    size_t prefix_length,
    const char *allocation_failure
) {
    if (prefix_length > SIZE_MAX - length) {
        mal_trap(context, "byte owner allocation size overflow");
    }
    size_t result_length = prefix_length + length;
    if (owner == NULL || owner->kind != MAL_BYTES_FLAT || owner->references != 1
        || offset != 0 || owner->length != (uint64_t)length) {
        size_t capacity = mal_bytes_capacity(result_length);
        size_t start = capacity - result_length;
        MalBytesFlat *result = mal_bytes_flat_allocate(
            context,
            result_length,
            capacity,
            start,
            allocation_failure
        );
        memcpy(result->bytes + start, prefix, prefix_length);
        if (length != 0) {
            memcpy(result->bytes + start + prefix_length, mal_bytes_data(owner) + offset, length);
        }
        mal_bytes_release(owner);
        return &result->header;
    }
    MalBytesFlat *flat = (MalBytesFlat *)owner;
    if (prefix_length > flat->start) {
        size_t capacity = mal_bytes_capacity(result_length);
        if (capacity > SIZE_MAX - sizeof(MalBytesFlat)) {
            mal_trap(context, "byte owner allocation size overflow");
        }
        flat = realloc(flat, sizeof(MalBytesFlat) + capacity);
        if (flat == NULL) {
            mal_trap(context, allocation_failure);
        }
        size_t start = capacity - length;
        memmove(flat->bytes + start, flat->bytes + flat->start, length);
        flat->capacity = capacity;
        flat->start = start;
    }
    flat->start -= prefix_length;
    memcpy(flat->bytes + flat->start, prefix, prefix_length);
    flat->header.length = (uint64_t)result_length;
    return &flat->header;
}

const uint8_t *mal_runtime_bytes_data(const void *owner) {
    return mal_bytes_data(owner);
}

void *mal_runtime_bytes_read(MalContext *context, const void *source, size_t length) {
    return mal_bytes_flat_copy(context, source, length, "byte allocation failed");
}

void *mal_runtime_bytes_retain(MalContext *context, const void *owner) {
    return mal_bytes_retain(context, (MalBytes *)owner);
}

void mal_runtime_bytes_release(const void *owner) {
    mal_bytes_release((MalBytes *)owner);
}

void mal_runtime_bytes_write(
    void *destination,
    const void *owner,
    size_t offset,
    size_t length
) {
    if (length != 0) {
        memcpy(destination, mal_bytes_data(owner) + offset, length);
    }
}

typedef struct {
    MalBytes *owner;
    unsigned char *data;
    size_t count;
    size_t stride;
    // Bytes between the logical end and this boundary retain their calloc zero.
    size_t zeroed_until;
    // Set only for elements that own managed values. Every stored element holds one reference, taken by `retain`
    // when the element is written and dropped by `release` when it is overwritten or the buffer dies.
    void (*retain)(MalContext *, void *element);
    void (*release)(void *element);
} MalBuffer;

static MalBuffer *mal_buffer_allocate(
    MalContext *context,
    size_t stride
);

static void mal_buffer_destroy(void *opaque_buffer) {
    MalBuffer *buffer = opaque_buffer;
    if (buffer->release != NULL) {
        for (size_t index = 0; index < buffer->count; ++index) {
            buffer->release(buffer->data + index * buffer->stride);
        }
    }
    mal_bytes_release(buffer->owner);
}

static MalBuffer *mal_buffer_allocate(
    MalContext *context,
    size_t stride
) {
    MalBuffer *buffer = mal_runtime_environment_allocate(
        context,
        sizeof(MalBuffer),
        mal_buffer_destroy
    );
    buffer->owner = NULL;
    buffer->data = NULL;
    buffer->count = 0;
    buffer->stride = stride;
    buffer->zeroed_until = 0;
    buffer->retain = NULL;
    buffer->release = NULL;
    return buffer;
}

static size_t mal_buffer_bytes(
    MalContext *context,
    size_t count,
    size_t stride
) {
    if (stride != 0 && count > SIZE_MAX / stride) {
        mal_trap(context, "buffer byte size overflow");
    }
    return count * stride;
}

void *mal_runtime_buffer_make(
    MalContext *context,
    size_t stride,
    size_t capacity
) {
    MalBuffer *buffer = mal_buffer_allocate(context, stride);
    size_t bytes = mal_buffer_bytes(context, capacity, stride);
    if (bytes != 0) {
        MalBytesFlat *flat = mal_bytes_flat_allocate_zeroed(
            context,
            0,
            bytes,
            "buffer allocation failed"
        );
        buffer->owner = &flat->header;
        buffer->data = flat->bytes;
        buffer->zeroed_until = bytes;
    }
    return buffer;
}


void *mal_runtime_buffer_make_managed(
    MalContext *context,
    size_t stride,
    size_t capacity,
    void (*retain)(MalContext *, void *),
    void (*release)(void *)
) {
    MalBuffer *buffer = mal_runtime_buffer_make(context, stride, capacity);
    buffer->retain = retain;
    buffer->release = release;
    return buffer;
}

__attribute__((noinline))
static MalBytesFlat *mal_buffer_grow_unique(
    MalContext *context,
    MalBytesFlat *flat,
    size_t required
) {
    size_t capacity = mal_bytes_capacity(required);
    if (capacity > SIZE_MAX - sizeof(MalBytesFlat)) {
        mal_trap(context, "byte owner allocation size overflow");
    }
    if (flat == NULL) {
        return mal_bytes_flat_allocate(
            context,
            required,
            capacity,
            0,
            "buffer allocation failed"
        );
    }
    flat = realloc(flat, sizeof(MalBytesFlat) + capacity);
    if (flat == NULL) {
        mal_trap(context, "buffer allocation failed");
    }
    flat->capacity = capacity;
    flat->header.length = (uint64_t)required;
    return flat;
}

__attribute__((always_inline))
size_t mal_runtime_buffer_new(
    MalContext *context,
    void *opaque_buffer,
    const void *value,
    size_t stride
) {
    MalBuffer *buffer = opaque_buffer;
    size_t index = buffer->count;
    if (stride != 0) {
        if (buffer->count >= SIZE_MAX / stride) {
            mal_trap(context, "buffer byte size overflow");
        }
        size_t length = buffer->count * stride;
        size_t required = (buffer->count + 1) * stride;
        MalBytesFlat *flat = (MalBytesFlat *)buffer->owner;
        if (flat == NULL || required > flat->capacity) {
            flat = mal_buffer_grow_unique(context, flat, required);
        } else {
            flat->header.length = (uint64_t)required;
        }
        int value_is_zero = 1;
        const unsigned char *value_bytes = value;
        for (size_t byte = 0; byte < stride; ++byte) {
            if (value_bytes[byte] != 0) {
                value_is_zero = 0;
                break;
            }
        }
        if (!value_is_zero || required > buffer->zeroed_until) {
            memcpy(flat->bytes + length, value, stride);
        }
        buffer->owner = &flat->header;
        buffer->data = flat->bytes;
        if (buffer->retain != NULL) {
            buffer->retain(context, flat->bytes + length);
        }
    } else if (buffer->count == SIZE_MAX) {
        mal_trap(context, "buffer count overflow");
    }
    ++buffer->count;
    return index;
}

// Writes `value` over `[offset, end)`, which lies within the storage. Elements below `old_count` held a reference that
// the write drops.
static void mal_buffer_fill_managed(
    MalContext *context,
    MalBuffer *buffer,
    size_t offset,
    size_t end,
    size_t old_count,
    const void *value
) {
    for (size_t index = offset; index < end; ++index) {
        unsigned char *element = buffer->data + index * buffer->stride;
        if (index < old_count) {
            buffer->release(element);
        }
        memcpy(element, value, buffer->stride);
        buffer->retain(context, element);
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
    size_t old_count = buffer->count;
    if (offset > old_count) {
        mal_trap(context, "buffer fill offset out of bounds");
    }
    if (count > SIZE_MAX - offset) {
        mal_trap(context, "buffer fill range overflow");
    }
    size_t end = offset + count;
    size_t new_count = old_count > end ? old_count : end;
    if (count == 0) {
        return;
    }
    if (stride == 0) {
        buffer->count = new_count;
        return;
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

    if (buffer->release != NULL) {
        mal_buffer_fill_managed(context, buffer, offset, end, old_count, value);
        return;
    }

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

// Accounts for the references of a copy that the caller then performs bytewise. Every source reference is taken before
// any destination one is dropped: the ranges may overlap, so a dropped element can also be a source.
static void mal_buffer_reference_copy(
    MalContext *context,
    MalBuffer *destination,
    size_t destination_offset,
    const MalBuffer *source,
    size_t source_offset,
    size_t count,
    size_t old_count
) {
    size_t stride = destination->stride;
    for (size_t index = 0; index < count; ++index) {
        destination->retain(
            context,
            (void *)(source->data + (source_offset + index) * stride)
        );
    }
    size_t destination_end = destination_offset + count;
    for (size_t index = destination_offset; index < destination_end && index < old_count; ++index) {
        destination->release(destination->data + index * stride);
    }
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
    size_t old_count = destination->count;
    size_t new_count = destination->count > destination_end
        ? destination->count
        : destination_end;
    if (count == 0) {
        return;
    }
    if (stride == 0) {
        destination->count = new_count;
        return;
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
    if (destination->release != NULL) {
        mal_buffer_reference_copy(
            context,
            destination,
            destination_offset,
            source,
            source_offset,
            count,
            old_count
        );
    }
    memmove(
        destination->data + destination_offset * stride,
        source->data + source_offset * stride,
        count * stride
    );
}

__attribute__((always_inline))
void *const *mal_runtime_buffer_data_slot(const void *opaque_buffer) {
    const MalBuffer *buffer = opaque_buffer;
    return (void *const *)&buffer->data;
}

size_t mal_runtime_buffer_count(const void *opaque_buffer) {
    const MalBuffer *buffer = opaque_buffer;
    return buffer->count;
}

void *mal_runtime_buffer_from(
    MalContext *context,
    const void *source,
    size_t offset,
    size_t count,
    size_t stride
) {
    MalBuffer *buffer = mal_runtime_buffer_make(context, stride, count);
    if (stride == 0) {
        buffer->count = count;
        return buffer;
    }
    if (offset > SIZE_MAX - count || offset + count > SIZE_MAX / stride) {
        mal_trap(context, "buffer copy range overflow");
    }
    if (count == 0) {
        return buffer;
    }
    size_t source_offset = offset * stride;
    size_t bytes = count * stride;
    memcpy(buffer->data, (const unsigned char *)source + source_offset, bytes);
    ((MalBytesFlat *)buffer->owner)->header.length = (uint64_t)bytes;
    buffer->count = count;
    return buffer;
}

void *mal_runtime_buffer_from_strings(
    MalContext *context,
    char *const *strings,
    size_t count,
    size_t stride,
    size_t length_offset
) {
    MalBuffer *buffer = mal_runtime_buffer_make(context, stride, count);
    if (count == 0) {
        return buffer;
    }
    for (size_t index = 0; index < count; ++index) {
        unsigned char *element = (unsigned char *)buffer->data + index * stride;
        size_t length = strlen(strings[index]);
        memcpy(element, &strings[index], sizeof strings[index]);
        memcpy(element + length_offset, &length, sizeof length);
    }
    ((MalBytesFlat *)buffer->owner)->header.length = (uint64_t)(count * stride);
    buffer->count = count;
    return buffer;
}

void mal_runtime_buffer_into(
    MalContext *context,
    const void *opaque_buffer,
    void *destination,
    size_t offset,
    size_t count,
    size_t stride
) {
    const MalBuffer *buffer = opaque_buffer;
    if (offset > buffer->count || count > buffer->count - offset) {
        mal_trap(context, "buffer copy range out of bounds");
    }
    if (stride == 0 || count == 0) {
        return;
    }
    if (offset > SIZE_MAX - count || offset + count > SIZE_MAX / stride) {
        mal_trap(context, "buffer copy range overflow");
    }
    memcpy(
        destination,
        buffer->data + offset * stride,
        count * stride
    );
}
