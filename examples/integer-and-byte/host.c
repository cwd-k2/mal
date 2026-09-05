#include "program.mal.h"

#include <inttypes.h>
#include <stdio.h>

MAL_DEFINE_printUInt64(context, value) {
    printf("%" PRIu64 "\n", value);
}
