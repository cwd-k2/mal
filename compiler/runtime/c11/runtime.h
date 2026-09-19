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

typedef struct {
    void *owner;
    const unsigned char *data;
    size_t length;
} MalBytesView;

_Noreturn void mal_trap(MalContext *context, const char *message);
void *mal_runtime_allocate(MalContext *context, size_t size);
void mal_runtime_deallocate(void *allocation);
void *mal_runtime_environment_allocate(
    MalContext *context,
    size_t size,
    void (*destroy)(void *)
);
void *mal_runtime_scoped_environment_allocate(MalContext *context, size_t size);
void mal_runtime_scoped_environment_deallocate(void *environment);
void *mal_runtime_environment_retain(MalContext *context, void *environment);
void mal_runtime_environment_release(void *environment);
void mal_control_destroy(MalContext *context);
void *mal_control_reserve_frame(
    MalContext *context,
    size_t current_bytes,
    size_t frame_size
);
void *mal_control_storage(MalContext *context);

const uint8_t *mal_runtime_bytes_data(const void *owner);
void *mal_runtime_bytes_read(MalContext *context, const void *source, size_t length);
void *mal_runtime_bytes_retain(MalContext *context, const void *owner);
void mal_runtime_bytes_release(const void *owner);
uint8_t mal_runtime_symbol_at(const void *data, size_t index);
void mal_runtime_symbol_concatenate(
    MalContext *context,
    MalBytesView *result,
    const void *left_owner,
    const void *left_data,
    size_t left_length,
    const void *right_owner,
    const void *right_data,
    size_t right_length
);
void mal_runtime_symbol_concatenate_consuming_left(
    MalContext *context,
    MalBytesView *result,
    void *left_owner,
    const void *left_data,
    size_t left_length,
    const void *right_owner,
    const void *right_data,
    size_t right_length
);
void mal_runtime_symbol_concatenate_consuming_right(
    MalContext *context,
    MalBytesView *result,
    const void *left_owner,
    const void *left_data,
    size_t left_length,
    void *right_owner,
    const void *right_data,
    size_t right_length
);
uint8_t mal_runtime_symbol_equal(
    const void *left_data,
    size_t left_length,
    const void *right_data,
    size_t right_length
);
void mal_runtime_bytes_write(
    void *destination,
    const void *owner,
    size_t offset,
    size_t length
);
void *mal_runtime_packed_builder_start(MalContext *context, size_t stride);
void *mal_runtime_packed_builder_edit(
    MalContext *context,
    const void *owner,
    const void *data,
    size_t count,
    size_t stride
);
void *mal_runtime_packed_builder_prepare_edit(MalContext *context, void *builder);
size_t mal_runtime_packed_builder_new(
    MalContext *context,
    void *builder,
    const void *value
);
void *const *mal_runtime_packed_builder_data_slot(const void *builder);
void mal_runtime_packed_builder_finish(MalBytesView *result, void *builder);

#endif
