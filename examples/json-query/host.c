#include "program.mal.h"

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

    MalType_Symbol result = mal_Symbol_return(
        call,
        mal_Symbol_from_bytes((mal_span_t){ .data = data, .length = length })
    );
    free(data);
    return result;
}

MAL_DEFINE_symbolFromByte(call, value) {
    uint8_t byte = value;
    return mal_Symbol_return(
        call,
        mal_Symbol_from_bytes((mal_span_t){ .data = &byte, .length = 1 })
    );
}

MAL_DEFINE_writeStdout(call, value) {
    mal_span_t bytes = mal_Symbol_to_bytes(call, value);
    if (bytes.length > 0
        && fwrite(bytes.data, 1, (size_t)bytes.length, stdout) != (size_t)bytes.length) {
        mal_call_trap(call, "cannot write stdout");
    }
    return mal_Unit_return(call);
}
