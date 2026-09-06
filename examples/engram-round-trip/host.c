#include "program.mal.h"

#include <inttypes.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

static const uint8_t received[8] = {
    'm', 'a', 'l', 0xe3, 0x81, 0x82, 0x00, 0xff,
};

static const uint8_t expected[9] = {
    'm', 'a', 'l', 0xe3, 0x81, 0x82, 0x00, 0xff, '!',
};

MAL_DEFINE_receive(context) {
    uint8_t *scratch = (uint8_t *)malloc(sizeof(received));
    if (scratch == NULL) {
        mal_trap(context, "host allocation failed");
    }
    memcpy(scratch, received, sizeof(received));
    MalType_Engram result = mal_Engram_copy_from_bytes(context, scratch, UINT64_C(8));
    memset(scratch, 0, sizeof(received));
    free(scratch);
    return result;
}

MAL_DEFINE_send(context, value) {
    if (mal_Engram_length(value) != UINT64_C(9)
        || memcmp(mal_Engram_data(value), expected, sizeof(expected)) != 0) {
        mal_trap(context, "unexpected Engram bytes");
    }
    printf("%" PRIu64 " bytes\n", mal_Engram_length(value));
}
