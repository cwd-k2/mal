#include "program.mal.h"

#include <stdio.h>

void mal_ext_printInt32(MalContext *context, int32_t value) {
    (void)context;
    printf("%d\n", value);
}
