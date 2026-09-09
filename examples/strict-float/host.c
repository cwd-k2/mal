#include "program.mal.h"

#include <string.h>

MAL_DEFINE_inspect(call, value) {
    uint32_t halfway_bits;
    uint32_t subnormal_bits;
    uint64_t zero_bits;
    memcpy(&halfway_bits, &value.field_0, sizeof(halfway_bits));
    memcpy(&subnormal_bits, &value.field_1, sizeof(subnormal_bits));
    memcpy(&zero_bits, &value.field_2, sizeof(zero_bits));

    return mal_Int32_return(
        call,
        halfway_bits == UINT32_C(0x3f800000) &&
        subnormal_bits == UINT32_C(0x00000001) &&
        zero_bits == UINT64_C(0x8000000000000000) &&
        value.field_3 == INT64_C(-42)
            ? INT32_C(0)
            : INT32_C(1)
    );
}
