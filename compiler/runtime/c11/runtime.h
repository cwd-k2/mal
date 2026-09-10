#ifndef MAL_RUNTIME_H
#define MAL_RUNTIME_H

#include <stddef.h>

typedef struct {
    unsigned char *storage;
    size_t capacity;
} MalControlArena;

void mal_control_destroy(MalControlArena *arena);
void *mal_control_reserve_bytes(MalControlArena *arena, size_t required_bytes);
void *mal_control_reserve_frame(
    MalControlArena *arena,
    size_t current_bytes,
    size_t frame_size
);
void *mal_control_reserve_slots(
    MalControlArena *arena,
    size_t required_slots,
    size_t slot_size
);
void *mal_control_storage(MalControlArena *arena);

#endif
