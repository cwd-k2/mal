#include "runtime.h"

#include <stdio.h>
#include <stdlib.h>

_Noreturn void mal_trap(MalContext *context, const char *message) {
    (void)context;
    fputs("mal trap: ", stderr);
    fputs(message, stderr);
    fputc('\n', stderr);
    abort();
}
