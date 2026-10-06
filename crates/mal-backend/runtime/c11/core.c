#include "runtime.h"

#include <stdio.h>
#include <stdlib.h>

typedef struct {
    size_t references;
    void (*destroy)(void *);
} MalOwnerHeader;

_Static_assert(
    sizeof(MalOwnerHeader) % _Alignof(max_align_t) == 0,
    "managed owner payload alignment is insufficient"
);

void *mal_allocate(MalContext *context, size_t size) {
    if (size == 0) {
        return NULL;
    }
    void *allocation = malloc(size);
    if (allocation == NULL) {
        mal_trap(context, "allocation failure");
    }
    return allocation;
}

void mal_deallocate(MalContext *context, void *allocation) {
    (void)context;
    free(allocation);
}

void *mal_runtime_owner_allocate(
    MalContext *context,
    size_t size,
    void (*destroy)(void *)
) {
    if (destroy == NULL) {
        mal_trap(context, "managed owner destructor missing");
    }
    if (size > SIZE_MAX - sizeof(MalOwnerHeader)) {
        mal_trap(context, "managed owner size overflow");
    }
    MalOwnerHeader *header = mal_allocate(
        context,
        sizeof(MalOwnerHeader) + size
    );
    header->references = 1;
    header->destroy = destroy;
    return header + 1;
}

__attribute__((always_inline))
void *mal_runtime_owner_retain(void *owner) {
    if (owner == NULL || ((uintptr_t)owner & 1) != 0) {
        return owner;
    }
    MalOwnerHeader *header = (MalOwnerHeader *)owner - 1;
    /* A live owner is referenced at least once, and every reference occupies an addressable slot, so the
     * count cannot reach SIZE_MAX. Stating both facts lets the optimizer cancel a retain against a later release
     * and remove the zero test of the release. */
    __builtin_assume(header->references >= 1);
    __builtin_assume(header->references < SIZE_MAX);
    ++header->references;
    return owner;
}

__attribute__((always_inline))
void mal_runtime_owner_release(void *owner) {
    if (owner == NULL || ((uintptr_t)owner & 1) != 0) {
        return;
    }
    MalOwnerHeader *header = (MalOwnerHeader *)owner - 1;
    --header->references;
    if (header->references == 0) {
        header->destroy(owner);
        free(header);
    }
}

__attribute__((always_inline))
uint8_t mal_runtime_owner_is_unique(const void *owner) {
    if (owner == NULL || ((uintptr_t)owner & 1) != 0) {
        return 0;
    }
    const MalOwnerHeader *header = (const MalOwnerHeader *)owner - 1;
    return header->references == 1;
}

_Noreturn void mal_trap(MalContext *context, const char *message) {
    (void)context;
    fputs("mal trap: ", stderr);
    fputs(message, stderr);
    fputc('\n', stderr);
    abort();
}
