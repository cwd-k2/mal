#include "runtime.h"

#include <pthread.h>
#include <stdint.h>
#include <stdlib.h>

/* The generated umbrella header is force-included before this translation unit, so glibc has already selected its
 * feature surface before control.c could define _GNU_SOURCE. The supported runtime is pinned glibc. */
extern int pthread_getattr_np(pthread_t thread, pthread_attr_t *attributes);
extern int pthread_attr_getstack(
    const pthread_attr_t *restrict attributes,
    void **restrict stack_address,
    size_t *restrict stack_size
);

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

/* Native recursion may use this much stack below the program entry. The equal reserve keeps the fallback call and its
 * fixed native work away from the mapped stack boundary. */
#define MAL_NATIVE_STACK_BUDGET (64u * 1024u)
#define MAL_NATIVE_STACK_RESERVE MAL_NATIVE_STACK_BUDGET

__attribute__((noinline))
void mal_native_stack_begin(MalContext *context) {
    uintptr_t current = (uintptr_t)__builtin_frame_address(0);
    context->native_stack_limit = UINTPTR_MAX;

    pthread_attr_t attributes;
    if (pthread_getattr_np(pthread_self(), &attributes) != 0) {
        return;
    }
    void *stack_address = NULL;
    size_t stack_size = 0;
    int found = pthread_attr_getstack(&attributes, &stack_address, &stack_size);
    pthread_attr_destroy(&attributes);
    if (found != 0) {
        return;
    }

    uintptr_t low = (uintptr_t)stack_address;
    if (stack_size > UINTPTR_MAX - low) {
        return;
    }
    uintptr_t high = low + stack_size;
    if (current < low || current > high || current < MAL_NATIVE_STACK_BUDGET) {
        return;
    }
    uintptr_t budget_limit = current - MAL_NATIVE_STACK_BUDGET;
    if (low > UINTPTR_MAX - MAL_NATIVE_STACK_RESERVE) {
        return;
    }
    uintptr_t boundary_limit = low + MAL_NATIVE_STACK_RESERVE;
    if (current <= boundary_limit) {
        return;
    }
    context->native_stack_limit = budget_limit > boundary_limit ? budget_limit : boundary_limit;
}

__attribute__((always_inline))
uint8_t mal_native_stack_is_deep(MalContext *context, void *stack_address) {
    return (uintptr_t)stack_address <= context->native_stack_limit;
}
