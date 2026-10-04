#include "program.mal.h"

#include <stdio.h>
#include <string.h>

MAL_DEFINE_printInt32(call, value) {
    printf("%d\n", value);
    return mal_unit;
}

MAL_DEFINE_inspectNumeric(call, value) {
    uint32_t halfway_bits;
    uint32_t subnormal_bits;
    uint64_t zero_bits;
    memcpy(&halfway_bits, &value.field_3, sizeof(halfway_bits));
    memcpy(&subnormal_bits, &value.field_4, sizeof(subnormal_bits));
    memcpy(&zero_bits, &value.field_5, sizeof(zero_bits));

    return value.field_0 == UINT64_C(255) &&
        value.field_1 == UINT64_C(0) &&
        value.field_2 == UINT64_MAX &&
        halfway_bits == UINT32_C(0x3f800000) &&
        subnormal_bits == UINT32_C(0x00000001) &&
        zero_bits == UINT64_C(0x8000000000000000) &&
        value.field_6 == INT64_C(-42)
        ? INT32_C(0)
        : INT32_C(1);
}
