#include "program.mal.h"

#include <inttypes.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

static const uint8_t expected[8] = {
    'm', 'a', 'l', 0xe3, 0x81, 0x82, 0x00, 0xff,
};

MAL_DEFINE_receive(context) {
    uint8_t *scratch = (uint8_t *)malloc(sizeof(expected));
    if (scratch == NULL) {
        mal_trap(context, "host allocation failed");
    }
    memcpy(scratch, expected, sizeof(expected));
    MalEngram result = mal_engram_copy(context, scratch, UINT64_C(8));
    memset(scratch, 0, sizeof(expected));
    free(scratch);
    return result;
}

MAL_DEFINE_send(context, value) {
    if (value.length != UINT64_C(8)
        || memcmp(value.data, expected, sizeof(expected)) != 0) {
        mal_trap(context, "unexpected Engram bytes");
    }
    printf("%" PRIu64 " bytes\n", value.length);
}
