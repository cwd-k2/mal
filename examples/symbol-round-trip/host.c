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

static uint8_t transfer_buffer[16];

MAL_DEFINE_transferBuffer(call) {
    return mal_Address_return(call, transfer_buffer);
}

MAL_DEFINE_receive(call, value) {
    if (value.field_0 != transfer_buffer || value.field_1 < sizeof(received)) {
        mal_call_trap(call, "invalid receive buffer");
    }
    memcpy(value.field_0, received, sizeof(received));
    return mal_USize_return(call, sizeof(received));
}

MAL_DEFINE_send(call, value) {
    if (value.field_0 != transfer_buffer
        || value.field_1 != sizeof(expected)
        || memcmp(value.field_0, expected, sizeof(expected)) != 0) {
        mal_call_trap(call, "unexpected Symbol bytes");
    }
    printf("%zu bytes\n", value.field_1);
    return mal_Unit_return(call);
}
