#include "program.mal.h"

#include <stdint.h>
#include <stdlib.h>

MAL_DEFINE_allocate(context, size) {
    if (size > SIZE_MAX)
        mal_trap(context, "allocation size overflow");
    uint8_t *memory = malloc((size_t)size);
    if (memory == NULL)
        mal_trap(context, "allocation failed");
    return mal_Ptr_from_address(memory);
}

MAL_DEFINE_release(context, pointer) {
    free(mal_Ptr_address(pointer));
}
