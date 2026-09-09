#include "program.mal.h"

#include <inttypes.h>
#include <stdio.h>

MAL_DEFINE_printUInt64(call, value) {
    printf("%" PRIu64 "\n", value);
    return mal_Unit_return(call);
}
