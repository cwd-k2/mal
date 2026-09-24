#include "runtime.h"

#include <stdint.h>
#include <stdlib.h>

void mal_control_destroy(MalContext *context) {
    MalControlArena *arena = &context->control;
    free(arena->storage);
    arena->storage = NULL;
    arena->capacity = 0;
}

static void *mal_control_grow(MalContext *context, size_t required_bytes) {
    MalControlArena *arena = &context->control;
    size_t capacity = arena->capacity == 0 ? 64 : arena->capacity;
    while (capacity < required_bytes) {
        if (capacity > SIZE_MAX / 2) {
            capacity = required_bytes;
            break;
        }
        capacity *= 2;
    }
    void *storage = realloc(arena->storage, capacity);
    if (storage == NULL) {
        mal_trap(context, "control storage allocation failed");
    }
    arena->storage = storage;
    arena->capacity = capacity;
    return storage;
}

__attribute__((always_inline))
void *mal_control_reserve_frame(
    MalContext *context,
    size_t current_bytes,
    size_t frame_size
) {
    if (current_bytes > SIZE_MAX - frame_size) {
        mal_trap(context, "control storage size overflow");
    }
    size_t required_bytes = current_bytes + frame_size;
    if (required_bytes <= context->control.capacity) {
        return context->control.storage;
    }
    return mal_control_grow(context, required_bytes);
}

__attribute__((always_inline))
void *mal_control_storage(MalContext *context) {
    return context->control.storage;
}

__attribute__((always_inline))
size_t mal_control_capacity(MalContext *context) {
    return context->control.capacity;
}
