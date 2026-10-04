#include "buffer_internal.h"

#include <stdint.h>
#include <string.h>

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
    buffer->count = count;
    return buffer;
}

// A Symbol element starts with its owner, which is all the callbacks below need.
static void mal_symbol_element_retain(MalContext *context, void *element) {
    mal_bytes_retain(context, *(MalBytes **)element);
}

static void mal_symbol_element_release(void *element) {
    mal_bytes_release(*(MalBytes **)element);
}

void *mal_runtime_buffer_from_arguments(
    MalContext *context,
    char *const *strings,
    size_t count,
    size_t stride,
    size_t data_offset,
    size_t length_offset
) {
    MalBuffer *buffer = mal_runtime_buffer_make_managed(
        context,
        stride,
        count,
        mal_symbol_element_retain,
        mal_symbol_element_release
    );
    if (count == 0) {
        return buffer;
    }
    for (size_t index = 0; index < count; ++index) {
        unsigned char *element = buffer->data + index * stride;
        size_t length = strlen(strings[index]);
        MalBytes *owner = mal_runtime_bytes_read(context, strings[index], length);
        const unsigned char *data = mal_bytes_data(owner);
        memset(element, 0, stride);
        memcpy(element, &owner, sizeof owner);
        memcpy(element + data_offset, &data, sizeof data);
        memcpy(element + length_offset, &length, sizeof length);
        buffer->count = index + 1;
    }
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
