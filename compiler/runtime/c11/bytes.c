#include "bytes_internal.h"

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

static size_t mal_bytes_capacity(size_t required) {
    size_t capacity = 16;
    while (capacity < required) {
        if (capacity > SIZE_MAX / 2) {
            return required;
        }
        capacity *= 2;
    }
    return capacity;
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
    return mal_bytes_flat_copy(context, source, length, "packed allocation failed");
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
    size_t offset;
    size_t count;
    size_t stride;
    uint8_t editable;
} MalPackedBuilder;

static MalPackedBuilder *mal_packed_builder_allocate(
    MalContext *context,
    size_t stride
) {
    MalPackedBuilder *builder = mal_runtime_scoped_environment_allocate(
        context,
        sizeof(MalPackedBuilder)
    );
    builder->owner = NULL;
    builder->offset = 0;
    builder->count = 0;
    builder->stride = stride;
    builder->editable = 1;
    return builder;
}

void *mal_runtime_packed_builder_start(MalContext *context, size_t stride) {
    return mal_packed_builder_allocate(context, stride);
}

void *mal_runtime_packed_builder_edit(
    MalContext *context,
    const void *owner,
    size_t offset,
    size_t count,
    size_t stride
) {
    MalPackedBuilder *builder = mal_packed_builder_allocate(context, stride);
    builder->owner = mal_bytes_retain(context, (MalBytes *)owner);
    builder->offset = offset;
    builder->count = count;
    builder->editable = 0;
    return builder;
}

static size_t mal_packed_builder_bytes(
    MalContext *context,
    size_t count,
    size_t stride
) {
    if (stride != 0 && count > SIZE_MAX / stride) {
        mal_trap(context, "packed builder byte size overflow");
    }
    return count * stride;
}

__attribute__((always_inline))
static void mal_packed_builder_make_editable(
    MalContext *context,
    MalPackedBuilder *builder
) {
    if (builder->editable) {
        return;
    }
    size_t bytes = mal_packed_builder_bytes(
        context,
        builder->count,
        builder->stride
    );
    if (builder->owner == NULL) {
        builder->offset = 0;
        builder->editable = 1;
        return;
    }
    if (builder->owner->kind == MAL_BYTES_FLAT
        && builder->owner->references == 1
        && builder->offset == 0
        && builder->owner->length == (uint64_t)bytes) {
        builder->editable = 1;
        return;
    }
    const unsigned char *source = bytes == 0
        ? NULL
        : mal_bytes_data(builder->owner) + builder->offset;
    MalBytes *copy = mal_bytes_flat_copy(
        context,
        source,
        bytes,
        "packed builder allocation failed"
    );
    mal_bytes_release(builder->owner);
    builder->owner = copy;
    builder->offset = 0;
    builder->editable = 1;
}

size_t mal_runtime_packed_builder_new(
    MalContext *context,
    void *opaque_builder,
    const void *value
) {
    MalPackedBuilder *builder = opaque_builder;
    if (builder->count == SIZE_MAX) {
        mal_trap(context, "packed builder count overflow");
    }
    size_t index = builder->count;
    if (builder->stride != 0) {
        mal_packed_builder_make_editable(context, builder);
        size_t bytes = mal_packed_builder_bytes(
            context,
            builder->count,
            builder->stride
        );
        builder->owner = mal_bytes_append(
            context,
            builder->owner,
            0,
            bytes,
            value,
            builder->stride,
            "packed builder allocation failed"
        );
    }
    ++builder->count;
    return index;
}

__attribute__((always_inline))
const void *mal_runtime_packed_builder_get(
    const void *opaque_builder,
    size_t index,
    size_t stride
) {
    const MalPackedBuilder *builder = opaque_builder;
    if (stride == 0) {
        return NULL;
    }
    return mal_bytes_data(builder->owner)
        + builder->offset
        + index * stride;
}

__attribute__((always_inline))
void mal_runtime_packed_builder_put(
    MalContext *context,
    void *opaque_builder,
    size_t index,
    const void *value,
    size_t stride
) {
    MalPackedBuilder *builder = opaque_builder;
    if (stride == 0) {
        return;
    }
    mal_packed_builder_make_editable(context, builder);
    MalBytesFlat *flat = (MalBytesFlat *)builder->owner;
    memcpy(
        flat->bytes + flat->start + index * stride,
        value,
        stride
    );
}

void mal_runtime_packed_builder_finish(MalBytesView *result, void *opaque_builder) {
    MalPackedBuilder *builder = opaque_builder;
    result->owner = builder->owner;
    result->offset = builder->offset;
    result->length = builder->count;
    mal_runtime_scoped_environment_deallocate(builder);
}
