#include "program.mal.h"

#include <stdio.h>
#include <string.h>

static const uint8_t received[] = {
    'm', 'a', 'l', 0xe3, 0x81, 0x82, 0x00, 0xff,
};

static const uint8_t expected[] = {
    'M', 'a', 'l', 0xe3, 0x81, 0x82, 0x00, 0xff, '!',
};

MAL_DEFINE_receive(call) {
    mal_type(Buffer) result = mal_buffer(call, mal_storage(mal_type(UInt8)), sizeof(received));
    mal_append(call, result, received, sizeof(received));
    return mal_move(result);
}

MAL_DEFINE_send(call, value) {
    if (value.length != sizeof(expected)
        || memcmp(value.data, expected, sizeof(expected)) != 0) {
        mal_call_trap(call, "unexpected output bytes");
    }
    printf("%zu bytes\n", value.length);
    return mal_unit;
}
