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

MAL_DEFINE_receive(call) {
    return mal_Symbol_return(
        call,
        mal_Symbol_from_bytes(
            (mal_span_t){ .data = received, .length = sizeof(received) }
        )
    );
}

MAL_DEFINE_send(call, value) {
    mal_span_t bytes = mal_Symbol_to_bytes(call, value);
    if (bytes.length != UINT64_C(9)
        || memcmp(bytes.data, expected, sizeof(expected)) != 0) {
        mal_call_trap(call, "unexpected Symbol bytes");
    }
    printf("%" PRIu64 " bytes\n", bytes.length);
    return mal_Unit_return(call);
}
