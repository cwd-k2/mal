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
    mal_Buffer_t result = mal_Buffer_make(call, sizeof(uint8_t), sizeof(received));
    for (size_t index = 0; index < sizeof(received); ++index) {
        mal_Buffer_new(call, result, &received[index], sizeof(uint8_t));
    }
    return mal_Buffer_return_move(call, result);
}

MAL_DEFINE_send(call, value) {
    if (value.length != sizeof(expected)
        || memcmp(value.data, expected, sizeof(expected)) != 0) {
        mal_call_trap(call, "unexpected output bytes");
    }
    printf("%zu bytes\n", value.length);
    return mal_Unit_return(call);
}
