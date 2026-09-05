#include "program.mal.h"

MalPtr mal_ext_memory(MalContext *context) {
    static uint8_t bytes[10];
    (void)context;
    return (MalPtr){ .address = bytes };
}
