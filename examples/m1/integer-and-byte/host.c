#include "program.mal.h"

#include <inttypes.h>
#include <stdio.h>

void mal_ext_printUInt64(MalContext *context, uint64_t value) {
    (void)context;
    printf("%" PRIu64 "\n", value);
}
