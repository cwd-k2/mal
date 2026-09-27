#include "runtime.h"

#include <stdio.h>
#include <stdlib.h>

typedef struct {
    size_t references;
    void (*destroy)(void *);
} MalEnvironmentHeader;

void *mal_runtime_allocate(MalContext *context, size_t size) {
    if (size == 0) {
        return NULL;
    }
    void *allocation = malloc(size);
    if (allocation == NULL) {
        mal_trap(context, "allocation failure");
    }
    return allocation;
}

void mal_runtime_deallocate(void *allocation) {
    free(allocation);
}

void *mal_runtime_environment_allocate(
    MalContext *context,
    size_t size,
    void (*destroy)(void *)
) {
    if (destroy == NULL) {
        mal_trap(context, "closure environment destructor missing");
    }
    if (size > SIZE_MAX - sizeof(MalEnvironmentHeader)) {
        mal_trap(context, "closure environment size overflow");
    }
    MalEnvironmentHeader *header = mal_runtime_allocate(
        context,
        sizeof(MalEnvironmentHeader) + size
    );
    header->references = 1;
    header->destroy = destroy;
    return header + 1;
}

__attribute__((always_inline))
void *mal_runtime_environment_retain(MalContext *context, void *environment) {
    if (environment == NULL || ((uintptr_t)environment & 1) != 0) {
        return environment;
    }
    MalEnvironmentHeader *header = (MalEnvironmentHeader *)environment - 1;
    /* A live environment is referenced at least once, and every reference occupies an addressable slot, so the
     * count cannot reach SIZE_MAX. Stating both facts lets the optimizer cancel a retain against a later release
     * instead of keeping the trap and the zero test of the release. */
    __builtin_assume(header->references >= 1);
    __builtin_assume(header->references < SIZE_MAX);
    ++header->references;
    (void)context;
    return environment;
}

__attribute__((always_inline))
void mal_runtime_environment_release(void *environment) {
    if (environment == NULL || ((uintptr_t)environment & 1) != 0) {
        return;
    }
    MalEnvironmentHeader *header = (MalEnvironmentHeader *)environment - 1;
    --header->references;
    if (header->references == 0) {
        header->destroy(environment);
        free(header);
    }
}

__attribute__((always_inline))
uint8_t mal_runtime_environment_is_unique(const void *environment) {
    if (environment == NULL || ((uintptr_t)environment & 1) != 0) {
        return 0;
    }
    const MalEnvironmentHeader *header = (const MalEnvironmentHeader *)environment - 1;
    return header->references == 1;
}

_Noreturn void mal_trap(MalContext *context, const char *message) {
    (void)context;
    fputs("mal trap: ", stderr);
    fputs(message, stderr);
    fputc('\n', stderr);
    abort();
}
