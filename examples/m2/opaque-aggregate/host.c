#include "program.mal.h"

#include <inttypes.h>
#include <stdio.h>

MalOpaque_Mem mal_ext_allocate(MalContext *context, uint64_t value) {
    (void)context;
    return (MalOpaque_Mem){ .bits = (uintptr_t)value };
}

MalSum_1 mal_ext_resize(
    MalContext *context,
    MalOpaque_Mem memory,
    uint64_t amount
) {
    (void)context;
    memory.bits += (uintptr_t)amount;
    printf("%" PRIuPTR "\n", memory.bits);
    return (MalSum_1){
        .tag = UINT32_C(1),
        .payload.variant_1 = {
            .field_0 = memory,
            .field_1 = amount,
        },
    };
}

uint64_t mal_ext_handleBits(MalContext *context, MalOpaque_Mem memory) {
    (void)context;
    return (uint64_t)memory.bits;
}
