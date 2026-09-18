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
    if (header->references == SIZE_MAX) {
        mal_trap(context, "closure reference count overflow");
    }
    ++header->references;
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

void *mal_runtime_scoped_environment_allocate(MalContext *context, size_t size) {
    return mal_runtime_allocate(context, size);
}

void mal_runtime_scoped_environment_deallocate(void *environment) {
    free(environment);
}

_Noreturn void mal_trap(MalContext *context, const char *message) {
    (void)context;
    fputs("mal trap: ", stderr);
    fputs(message, stderr);
    fputc('\n', stderr);
    abort();
}
