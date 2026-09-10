#include "runtime.h"

#include <stdio.h>
#include <stdlib.h>

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

_Noreturn void mal_trap(MalContext *context, const char *message) {
    (void)context;
    fputs("mal trap: ", stderr);
    fputs(message, stderr);
    fputc('\n', stderr);
    abort();
}
