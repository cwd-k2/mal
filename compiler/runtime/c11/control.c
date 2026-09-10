#include "runtime.h"

#include <stdint.h>
#include <stdlib.h>

void mal_control_destroy(MalContext *context) {
    MalControlArena *arena = &context->control;
    free(arena->storage);
    arena->storage = NULL;
    arena->capacity = 0;
}

void *mal_control_reserve_bytes(MalContext *context, size_t required_bytes) {
    MalControlArena *arena = &context->control;
    if (required_bytes <= arena->capacity) {
        return arena->storage;
    }
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
        abort();
    }
    arena->storage = storage;
    arena->capacity = capacity;
    return storage;
}

void *mal_control_reserve_slots(
    MalContext *context,
    size_t required_slots,
    size_t slot_size
) {
    if (slot_size != 0 && required_slots > SIZE_MAX / slot_size) {
        abort();
    }
    return mal_control_reserve_bytes(context, required_slots * slot_size);
}

void *mal_control_reserve_frame(
    MalContext *context,
    size_t current_bytes,
    size_t frame_size
) {
    if (current_bytes > SIZE_MAX - frame_size) {
        abort();
    }
    return mal_control_reserve_bytes(context, current_bytes + frame_size);
}

void *mal_control_storage(MalContext *context) {
    return context->control.storage;
}
