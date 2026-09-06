#include "program.mal.h"

#include <inttypes.h>
#include <stdio.h>

MAL_DEFINE_allocate(context, value) {
    return mal_Mem_from_bits((uintptr_t)value);
}

MAL_DEFINE_resize(context, memory, amount) {
    uintptr_t bits = mal_Mem_bits(memory) + (uintptr_t)amount;
    printf("%" PRIuPTR "\n", bits);
    return mal_Response_make_1(mal_Mem_from_bits(bits), amount);
}

MAL_DEFINE_handleBits(context, memory) {
    return (uint64_t)mal_Mem_bits(memory);
}
