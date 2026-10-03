#include "pool_internal.h"

#include <stdalign.h>
#include <stdint.h>
#include <stdlib.h>
#include <string.h>

static size_t mal_pool_bitmap_size(size_t capacity) {
    return capacity / 8 + (capacity % 8 != 0);
}

static size_t mal_pool_payload_offset(MalContext *context, size_t capacity) {
    size_t bitmap_size = mal_pool_bitmap_size(capacity);
    size_t padding = alignof(max_align_t) - 1;
    if (bitmap_size > SIZE_MAX - padding) {
        mal_trap(context, "pool occupancy size overflow");
    }
    return (bitmap_size + padding) & ~padding;
}

static size_t mal_pool_payload_size(
    MalContext *context,
    size_t capacity,
    size_t stride
) {
    if (stride != 0 && capacity > SIZE_MAX / stride) {
        mal_trap(context, "pool payload size overflow");
    }
    return capacity * stride;
}

static int mal_pool_is_live(const MalPool *pool, size_t index) {
    size_t byte = index / 8;
    uint8_t mask = (uint8_t)(1U << (index % 8));
    return (pool->backing[byte] & mask) != 0;
}

static void mal_pool_set_live(MalPool *pool, size_t index, int live) {
    size_t byte = index / 8;
    uint8_t mask = (uint8_t)(1U << (index % 8));
    if (live) {
        pool->backing[byte] |= mask;
    } else {
        pool->backing[byte] &= (uint8_t)~mask;
    }
}

static void mal_pool_destroy(void *opaque_pool) {
    MalPool *pool = opaque_pool;
    free(pool->metadata_owner);
    free(pool->backing);
}

static void mal_managed_pool_destroy(void *opaque_pool) {
    MalManagedPool *managed = opaque_pool;
    MalPool *pool = &managed->pool;
    if (managed->element_release != NULL && pool->stride != 0) {
        for (size_t index = 0; index < pool->capacity; ++index) {
            if (mal_pool_is_live(pool, index)) {
                managed->element_release(pool->payload + index * pool->stride);
            }
        }
    }
    if (managed->metadata_release != NULL) {
        managed->metadata_release(pool->metadata);
    }
    mal_pool_destroy(pool);
}

static MalPool *mal_pool_make(
    MalContext *context,
    const void *metadata,
    size_t metadata_size,
    size_t stride,
    size_t object_size,
    void (*destroy)(void *)
) {
    MalPool *pool = mal_runtime_environment_allocate(
        context,
        object_size,
        destroy
    );
    pool->backing = NULL;
    pool->payload = NULL;
    pool->metadata_owner = NULL;
    pool->capacity = 0;
    pool->stride = stride;
    pool->metadata_size = metadata_size;
    pool->inline_metadata = 0;

    if (metadata_size <= sizeof pool->inline_metadata) {
        pool->metadata = (unsigned char *)&pool->inline_metadata;
    } else {
        pool->metadata_owner = mal_runtime_allocate(context, metadata_size);
        pool->metadata = pool->metadata_owner;
    }
    if (metadata_size != 0) {
        memcpy(pool->metadata, metadata, metadata_size);
    }
    return pool;
}

void *mal_runtime_pool_make(
    MalContext *context,
    const void *metadata,
    size_t metadata_size,
    size_t stride
) {
    return mal_pool_make(
        context,
        metadata,
        metadata_size,
        stride,
        sizeof(MalPool),
        mal_pool_destroy
    );
}

void *mal_runtime_pool_make_managed(
    MalContext *context,
    const void *metadata,
    size_t metadata_size,
    size_t stride,
    MalRuntimeRetain metadata_retain,
    MalRuntimeRelease metadata_release,
    MalRuntimeRetain element_retain,
    MalRuntimeRelease element_release
) {
    if ((metadata_retain == NULL) != (metadata_release == NULL)
        || (element_retain == NULL) != (element_release == NULL)) {
        mal_trap(context, "pool lifecycle callback mismatch");
    }
    MalManagedPool *managed = (MalManagedPool *)mal_pool_make(
        context,
        metadata,
        metadata_size,
        stride,
        sizeof(MalManagedPool),
        mal_managed_pool_destroy
    );
    managed->metadata_retain = metadata_retain;
    managed->metadata_release = metadata_release;
    managed->element_retain = element_retain;
    managed->element_release = element_release;
    return managed;
}

size_t mal_runtime_pool_capacity(const void *opaque_pool) {
    const MalPool *pool = opaque_pool;
    return pool->capacity;
}

void mal_runtime_pool_grow(
    MalContext *context,
    void *opaque_pool,
    size_t count
) {
    MalPool *pool = opaque_pool;
    if (count > SIZE_MAX - pool->capacity) {
        mal_trap(context, "pool capacity overflow");
    }
    size_t next_capacity = pool->capacity + count;
    if (next_capacity == pool->capacity) {
        return;
    }

    size_t payload_offset = mal_pool_payload_offset(context, next_capacity);
    size_t payload_size = mal_pool_payload_size(context, next_capacity, pool->stride);
    if (payload_size > SIZE_MAX - payload_offset) {
        mal_trap(context, "pool allocation size overflow");
    }
    size_t allocation_size = payload_offset + payload_size;
    unsigned char *next_backing = mal_runtime_allocate(context, allocation_size);
    memset(next_backing, 0, allocation_size);
    unsigned char *next_payload = next_backing + payload_offset;

    if (pool->capacity != 0) {
        memcpy(next_backing, pool->backing, mal_pool_bitmap_size(pool->capacity));
        if (pool->stride != 0) {
            for (size_t index = 0; index < pool->capacity; ++index) {
                if (mal_pool_is_live(pool, index)) {
                    memcpy(
                        next_payload + index * pool->stride,
                        pool->payload + index * pool->stride,
                        pool->stride
                    );
                }
            }
        }
    }

    unsigned char *old_backing = pool->backing;
    pool->backing = next_backing;
    pool->payload = next_payload;
    pool->capacity = next_capacity;
    free(old_backing);
}

uint8_t mal_runtime_pool_peek(
    const void *opaque_pool,
    size_t index,
    void *result
) {
    const MalPool *pool = opaque_pool;
    if (!mal_pool_is_live(pool, index)) {
        return 0;
    }
    if (pool->stride != 0) {
        memcpy(result, pool->payload + index * pool->stride, pool->stride);
    }
    return 1;
}

uint8_t mal_runtime_pool_peek_managed(
    MalContext *context,
    const void *opaque_pool,
    size_t index,
    void *result
) {
    const MalManagedPool *managed = opaque_pool;
    uint8_t live = mal_runtime_pool_peek(opaque_pool, index, result);
    if (live != 0 && managed->element_retain != NULL) {
        managed->element_retain(context, result);
    }
    return live;
}

uint8_t mal_runtime_pool_swap(
    void *opaque_pool,
    size_t index,
    uint8_t next_live,
    const void *next,
    void *old
) {
    MalPool *pool = opaque_pool;
    int old_live = mal_pool_is_live(pool, index);
    if (pool->stride != 0) {
        unsigned char *slot = pool->payload + index * pool->stride;
        if (old_live) {
            memcpy(old, slot, pool->stride);
        }
        if (next_live) {
            memcpy(slot, next, pool->stride);
        }
    }
    mal_pool_set_live(pool, index, next_live != 0);
    return (uint8_t)old_live;
}

void mal_runtime_pool_meta(const void *opaque_pool, void *result) {
    const MalPool *pool = opaque_pool;
    if (pool->metadata_size != 0) {
        memcpy(result, pool->metadata, pool->metadata_size);
    }
}

void mal_runtime_pool_meta_managed(
    MalContext *context,
    const void *opaque_pool,
    void *result
) {
    const MalManagedPool *managed = opaque_pool;
    mal_runtime_pool_meta(opaque_pool, result);
    if (managed->metadata_retain != NULL) {
        managed->metadata_retain(context, result);
    }
}

void mal_runtime_pool_swap_meta(
    void *opaque_pool,
    const void *next,
    void *old
) {
    MalPool *pool = opaque_pool;
    if (pool->metadata_size != 0) {
        memcpy(old, pool->metadata, pool->metadata_size);
        memcpy(pool->metadata, next, pool->metadata_size);
    }
}
