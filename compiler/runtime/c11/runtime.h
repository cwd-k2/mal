#ifndef MAL_RUNTIME_H
#define MAL_RUNTIME_H

#include <stddef.h>
#include <stdint.h>

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

uint64_t mal_runtime_symbol_length(const void *symbol);
const uint8_t *mal_runtime_symbol_data(const void *symbol);
uint8_t mal_runtime_symbol_at(const void *symbol, uint64_t index);
void *mal_runtime_symbol_retain(MalContext *context, const void *symbol);
void mal_runtime_symbol_release(const void *symbol);
void *mal_runtime_symbol_concatenate(MalContext *context, const void *left, const void *right);
uint8_t mal_runtime_symbol_equal(const void *left, const void *right);
void *mal_runtime_symbol_read(MalContext *context, const void *source, uint64_t length);
void mal_runtime_symbol_write(void *destination, const void *symbol);

#endif
