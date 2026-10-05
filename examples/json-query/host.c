#include "host.mal.h"

#include <stdio.h>
#include <stdlib.h>

MAL_DEFINE_readStdin(call) {
    uint8_t *data = NULL;
    size_t length = 0;
    size_t capacity = 0;
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
    mal_type(Buffer) result = mal_buffer(call, mal_type(UInt8), length);
    if (length > 0) {
        mal_append(call, result, data, length);
    }
    free(data);
    return mal_move(result);
}

MAL_DEFINE_writeBytes(call, value) {
    if (value.length > 0
        && fwrite(value.data, 1, value.length, stdout) != value.length) {
        mal_call_trap(call, "cannot write stdout");
    }
    return mal_unit;
}
