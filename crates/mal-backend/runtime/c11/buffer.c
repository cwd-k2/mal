#include "buffer_internal.h"

#include <stdint.h>
#include <stdlib.h>
#include <string.h>

static MalBuffer *mal_buffer_allocate(
    MalContext *context,
    size_t stride,
    size_t size,
    void (*destroy)(void *)
);

static void mal_buffer_destroy(void *opaque_buffer) {
    MalBuffer *buffer = opaque_buffer;
    mal_bytes_release(buffer->owner);
}

static void mal_managed_buffer_destroy(void *opaque_buffer) {
    MalManagedBuffer *managed = opaque_buffer;
    for (size_t index = 0; index < managed->buffer.count; ++index) {
        managed->release(managed->buffer.data + index * managed->buffer.stride);
    }
    mal_buffer_destroy(&managed->buffer);
}

static MalBuffer *mal_buffer_allocate(
    MalContext *context,
    size_t stride,
    size_t size,
    void (*destroy)(void *)
) {
    MalBuffer *buffer = mal_runtime_environment_allocate(
        context,
        size,
        destroy
    );
    buffer->owner = NULL;
    buffer->data = NULL;
    buffer->count = 0;
    buffer->stride = stride;
    buffer->zeroed_until = 0;
    return buffer;
}

size_t mal_buffer_bytes(
    MalContext *context,
    size_t count,
    size_t stride
) {
    if (stride != 0 && count > SIZE_MAX / stride) {
        mal_trap(context, "buffer byte size overflow");
    }
    return count * stride;
}

static MalBuffer *mal_buffer_make(
    MalContext *context,
    size_t stride,
    size_t capacity,
    size_t size,
    void (*destroy)(void *)
) {
    MalBuffer *buffer = mal_buffer_allocate(context, stride, size, destroy);
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

void *mal_runtime_buffer_make(
    MalContext *context,
    size_t stride,
    size_t capacity
) {
    return mal_buffer_make(
        context,
        stride,
        capacity,
        sizeof(MalBuffer),
        mal_buffer_destroy
    );
}

void *mal_runtime_buffer_make_managed(
    MalContext *context,
    size_t stride,
    size_t capacity,
    void (*retain)(MalContext *, void *),
    void (*release)(void *)
) {
    MalManagedBuffer *managed = (MalManagedBuffer *)mal_buffer_make(
        context,
        stride,
        capacity,
        sizeof(MalManagedBuffer),
        mal_managed_buffer_destroy
    );
    managed->retain = retain;
    managed->release = release;
    return managed;
}


__attribute__((noinline))
MalBytesFlat *mal_buffer_grow_unique(
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
static inline size_t mal_buffer_append(
    MalContext *context,
    MalBuffer *buffer,
    const void *value,
    size_t stride
) {
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
    } else if (buffer->count == SIZE_MAX) {
        mal_trap(context, "buffer count overflow");
    }
    ++buffer->count;
    return index;
}

__attribute__((always_inline))
size_t mal_runtime_buffer_new(
    MalContext *context,
    void *buffer,
    const void *value,
    size_t stride
) {
    return mal_buffer_append(context, buffer, value, stride);
}

size_t mal_runtime_buffer_new_managed(
    MalContext *context,
    void *opaque_buffer,
    const void *value,
    size_t stride
) {
    MalManagedBuffer *managed = opaque_buffer;
    size_t index = mal_buffer_append(context, &managed->buffer, value, stride);
    managed->retain(context, managed->buffer.data + index * stride);
    return index;
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
