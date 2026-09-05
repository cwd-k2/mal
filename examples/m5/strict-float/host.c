#include "program.mal.h"

#include <string.h>

int32_t mal_ext_inspect(
    MalContext *context,
    float halfway,
    float minimum_subnormal,
    double negative_zero,
    int64_t converted
) {
    (void)context;
    uint32_t halfway_bits;
    uint32_t subnormal_bits;
    uint64_t zero_bits;
    memcpy(&halfway_bits, &halfway, sizeof(halfway_bits));
    memcpy(&subnormal_bits, &minimum_subnormal, sizeof(subnormal_bits));
    memcpy(&zero_bits, &negative_zero, sizeof(zero_bits));

    return halfway_bits == UINT32_C(0x3f800000) &&
           subnormal_bits == UINT32_C(0x00000001) &&
           zero_bits == UINT64_C(0x8000000000000000) &&
           converted == INT64_C(-42)
        ? INT32_C(0)
        : INT32_C(1);
}
