#ifndef MAL_RUNTIME_H
#define MAL_RUNTIME_H

#include <stddef.h>

typedef struct {
    unsigned char *storage;
    size_t capacity;
} MalControlArena;

typedef struct MalContext {
    MalControlArena control;
} MalContext;

void mal_control_destroy(MalContext *context);
void *mal_control_reserve_bytes(MalContext *context, size_t required_bytes);
void *mal_control_reserve_frame(
    MalContext *context,
    size_t current_bytes,
    size_t frame_size
);
void *mal_control_reserve_slots(
    MalContext *context,
    size_t required_slots,
    size_t slot_size
);
void *mal_control_storage(MalContext *context);

#endif
