#include "program.mal.h"

#include <stdint.h>
#include <stdlib.h>

MAL_DEFINE_allocate(call, size) {
    uint8_t *memory = malloc(size);
    if (memory == NULL)
        mal_call_trap(call, "allocation failed");
    return mal_Address_return(call, memory);
}

MAL_DEFINE_release(call, pointer) {
    free(pointer);
    return mal_Unit_return(call);
}
