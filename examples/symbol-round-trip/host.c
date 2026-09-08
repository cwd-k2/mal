#include "program.mal.h"

#include <inttypes.h>
#include <stdio.h>
#include <string.h>

static const uint8_t received[8] = {
    'm', 'a', 'l', 0xe3, 0x81, 0x82, 0x00, 0xff,
};

static const uint8_t expected[9] = {
    'm', 'a', 'l', 0xe3, 0x81, 0x82, 0x00, 0xff, '!',
};

MAL_DEFINE_receive(context) {
    MalSymbolAdmission admission = mal_SymbolAdmission_begin(context, sizeof(received));
    memcpy(mal_SymbolAdmission_data(&admission), received, sizeof(received));
    return mal_SymbolAdmission_finish(context, &admission, sizeof(received));
}

MAL_DEFINE_send(context, value) {
    if (mal_Symbol_length(value) != UINT64_C(9)
        || memcmp(mal_Symbol_data(value), expected, sizeof(expected)) != 0) {
        mal_trap(context, "unexpected Symbol bytes");
    }
    printf("%" PRIu64 " bytes\n", mal_Symbol_length(value));
}
