#include "program.mal.h"

#include <inttypes.h>
#include <stdio.h>

MAL_DEFINE_allocate(call, value) {
    return mal_Allocation_return(call, mal_Allocation_from_bits((uintptr_t)value));
}

MAL_DEFINE_resize(call, value) {
    uintptr_t bits = mal_Allocation_to_bits(value.field_0) + (uintptr_t)value.field_1;
    printf("%" PRIuPTR "\n", bits);
    value.field_0 = mal_Allocation_from_bits(bits);
    return mal_ResizeResult_return_1(call, value);
}

MAL_DEFINE_handleBits(call, memory) {
    return mal_UInt64_return(call, (uint64_t)mal_Allocation_to_bits(memory));
}
