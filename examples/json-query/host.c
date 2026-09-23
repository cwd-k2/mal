#include "program.mal.h"

#include <stdio.h>
#include <stdlib.h>
#include <string.h>

/* The mal writer chunks output to this nonempty process-lifetime buffer. */
static uint8_t output_buffer[32];

MAL_DEFINE_readStdin(call) {
    uint8_t *data = NULL;
    size_t length = 0;
    size_t capacity = 0;

    /* The host retains this buffer until mal finishes its borrowed parse and releases the handle. */
    for (;;) {
        if (length == capacity) {
            size_t next_capacity = capacity == 0 ? 4096 : capacity * 2;
            if (next_capacity < capacity) {
                free(data);
                mal_call_trap(call, "stdin is too large");
            }
            uint8_t *next = realloc(data, next_capacity);
            if (next == NULL) {
                free(data);
                mal_call_trap(call, "cannot allocate stdin buffer");
            }
            data = next;
            capacity = next_capacity;
        }

        size_t transferred = fread(data + length, 1, capacity - length, stdin);
        length += transferred;
        if (transferred == 0) {
            if (ferror(stdin)) {
                free(data);
                mal_call_trap(call, "cannot read stdin");
            }
            break;
        }
    }

    return mal_StdinBytes_return(
        call,
        (mal_StdinBytes_t){
            .field_0 = mal_InputAllocation_from_bits((uintptr_t)data),
            .field_1 = data,
            .field_2 = length,
        }
    );
}

MAL_DEFINE_releaseInput(call, input) {
    free((void *)mal_InputAllocation_to_bits(input));
    return mal_Unit_return(call);
}

MAL_DEFINE_outputBuffer(call) {
    return mal_OutputBuffer_return(
        call,
        (mal_OutputBuffer_t){
            .field_0 = output_buffer,
            .field_1 = sizeof(output_buffer),
        }
    );
}

MAL_DEFINE_writeBytes(call, value) {
    if (value.field_0 != output_buffer || value.field_1 > sizeof(output_buffer)
        || (value.field_1 > 0
            && fwrite(value.field_0, 1, value.field_1, stdout) != value.field_1)) {
        mal_call_trap(call, "cannot write stdout");
    }
    return mal_Unit_return(call);
}

MAL_DEFINE_argumentLength(call, address) {
    return mal_USize_return(call, strlen((const char *)address));
}
