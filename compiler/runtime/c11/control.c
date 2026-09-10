#include "runtime.h"

#include <stdint.h>
#include <stdlib.h>

void mal_control_destroy(MalControlArena *arena) {
    free(arena->storage);
    arena->storage = NULL;
    arena->capacity = 0;
}

void *mal_control_reserve_slots(
    MalControlArena *arena,
    size_t required_slots,
    size_t slot_size
) {
    if (slot_size != 0 && required_slots > SIZE_MAX / slot_size) {
        abort();
    }
    size_t required = required_slots * slot_size;
    if (required <= arena->capacity) {
        return arena->storage;
    }
    size_t capacity = arena->capacity == 0 ? 64 : arena->capacity;
    while (capacity < required) {
        if (capacity > SIZE_MAX / 2) {
            capacity = required;
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

void *mal_control_storage(MalControlArena *arena) {
    return arena->storage;
}
