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
    mal_bytes_release(mal_buffer_owner(buffer));
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
    buffer->storage.owner = NULL;
    buffer->data = NULL;
    buffer->count = 0;
    buffer->stride = stride;
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
    size_t bytes = mal_buffer_bytes(context, capacity, stride);
    MalBuffer *buffer = mal_buffer_allocate(context, stride, size, destroy);
    if (bytes <= sizeof buffer->storage.inline_bytes) {
        if (bytes != 0) {
            memset(buffer->storage.inline_bytes, 0, sizeof buffer->storage.inline_bytes);
            buffer->data = buffer->storage.inline_bytes;
        }
        return buffer;
    }
    MalBytesFlat *flat = mal_bytes_flat_allocate_zeroed(
        context,
        0,
        bytes,
        "buffer allocation failed"
    );
    flat->zeroed_until = bytes;
    buffer->storage.owner = &flat->header;
    buffer->data = flat->bytes;
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


void *mal_buffer_adopt(MalContext *context, MalBytesFlat *flat, size_t count) {
    MalBuffer *buffer = mal_buffer_allocate(context, 1, sizeof(MalBuffer), mal_buffer_destroy);
    flat->header.length = (uint64_t)count;
    flat->zeroed_until = count;
    buffer->storage.owner = &flat->header;
    buffer->data = flat->bytes;
    buffer->count = count;
    return buffer;
}

__attribute__((always_inline))
void mal_buffer_reserve(
    MalContext *context,
    MalBuffer *buffer,
    size_t required
) {
    int was_inline = mal_buffer_is_inline(buffer);
    MalBytesFlat *flat = (MalBytesFlat *)mal_buffer_owner(buffer);
    size_t capacity = was_inline ? sizeof buffer->storage.inline_bytes : 0;
    if (flat != NULL) {
        capacity = flat->capacity;
    }
    if (required <= capacity) {
        if (flat != NULL) {
            flat->header.length = (uint64_t)required;
        }
        return;
    }

    size_t zeroed_until = mal_buffer_zeroed_until(buffer);
    capacity = mal_bytes_capacity(required);
    if (capacity > SIZE_MAX - sizeof(MalBytesFlat)) {
        mal_trap(context, "byte owner allocation size overflow");
    }
    if (flat == NULL) {
        flat = mal_bytes_flat_allocate(
            context,
            required,
            capacity,
            0,
            "buffer allocation failed"
        );
        flat->zeroed_until = zeroed_until;
        if (was_inline && zeroed_until != 0) {
            memcpy(flat->bytes, buffer->data, zeroed_until);
        }
    } else {
        flat = realloc(flat, sizeof(MalBytesFlat) + capacity);
        if (flat == NULL) {
            mal_trap(context, "buffer allocation failed");
        }
        flat->capacity = capacity;
        flat->header.length = (uint64_t)required;
    }
    buffer->storage.owner = &flat->header;
    buffer->data = flat->bytes;
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
        mal_buffer_reserve(context, buffer, required);
        int value_is_zero = 1;
        const unsigned char *value_bytes = value;
        for (size_t byte = 0; byte < stride; ++byte) {
            if (value_bytes[byte] != 0) {
                value_is_zero = 0;
                break;
            }
        }
        if (!value_is_zero || required > mal_buffer_zeroed_until(buffer)) {
            memcpy(buffer->data + length, value, stride);
        }
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

size_t mal_runtime_buffer_new_managed_move(
    MalContext *context,
    void *opaque_buffer,
    const void *value,
    size_t stride
) {
    MalManagedBuffer *managed = opaque_buffer;
    return mal_buffer_append(context, &managed->buffer, value, stride);
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
