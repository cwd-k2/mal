#include "program.mal.h"

#include <stdint.h>
#include <stdio.h>

static uint8_t transfer_buffer[64];

MAL_DEFINE_transferBuffer(call) {
    return mal_Address_return(call, transfer_buffer);
}

MAL_DEFINE_writeBytes(call, value) {
    if (value.field_1 > sizeof(transfer_buffer)
        || fwrite(value.field_0, 1, value.field_1, stdout) != value.field_1) {
        mal_call_trap(call, "cannot write bytes");
    }
    return mal_Unit_return(call);
}
