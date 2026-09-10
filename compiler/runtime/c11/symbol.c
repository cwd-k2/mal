#include "runtime.h"

#include <stdint.h>
#include <stdlib.h>
#include <string.h>

typedef struct {
    uint64_t references;
    uint64_t length;
    unsigned char bytes[];
} MalSymbol;

_Noreturn void mal_trap(MalContext *context, const char *message);

static MalSymbol *mal_symbol_allocate(MalContext *context, uint64_t length) {
    if (length > SIZE_MAX - sizeof(MalSymbol)) {
        mal_trap(context, "symbol allocation size overflow");
    }
    MalSymbol *symbol = malloc(sizeof(MalSymbol) + (size_t)length);
    if (symbol == NULL) {
        mal_trap(context, "symbol allocation failed");
    }
    symbol->references = 1;
    symbol->length = length;
    return symbol;
}

uint64_t mal_runtime_symbol_length(const void *value) {
    return value == NULL ? 0 : ((const MalSymbol *)value)->length;
}

uint8_t mal_runtime_symbol_at(const void *value, uint64_t index) {
    return ((const MalSymbol *)value)->bytes[index];
}

void *mal_runtime_symbol_retain(MalContext *context, const void *value) {
    if (value == NULL) {
        return NULL;
    }
    MalSymbol *symbol = (MalSymbol *)value;
    if (symbol->references == UINT64_MAX) {
        return symbol;
    }
    if (symbol->references == UINT64_MAX - 1) {
        mal_trap(context, "symbol reference count overflow");
    }
    ++symbol->references;
    return symbol;
}

void mal_runtime_symbol_release(const void *value) {
    if (value == NULL) {
        return;
    }
    MalSymbol *symbol = (MalSymbol *)value;
    if (symbol->references == UINT64_MAX) {
        return;
    }
    --symbol->references;
    if (symbol->references == 0) {
        free(symbol);
    }
}

void *mal_runtime_symbol_concatenate(MalContext *context, const void *left, const void *right) {
    uint64_t left_length = mal_runtime_symbol_length(left);
    uint64_t right_length = mal_runtime_symbol_length(right);
    if (left_length > UINT64_MAX - right_length) {
        mal_trap(context, "symbol length overflow");
    }
    uint64_t length = left_length + right_length;
    if (length == 0) {
        return NULL;
    }
    MalSymbol *result = mal_symbol_allocate(context, length);
    if (left_length != 0) {
        memcpy(result->bytes, ((const MalSymbol *)left)->bytes, (size_t)left_length);
    }
    if (right_length != 0) {
        memcpy(result->bytes + left_length, ((const MalSymbol *)right)->bytes, (size_t)right_length);
    }
    return result;
}

uint8_t mal_runtime_symbol_equal(const void *left, const void *right) {
    uint64_t length = mal_runtime_symbol_length(left);
    if (length != mal_runtime_symbol_length(right)) {
        return 0;
    }
    if (length == 0) {
        return 1;
    }
    return (uint8_t)(memcmp(
        ((const MalSymbol *)left)->bytes,
        ((const MalSymbol *)right)->bytes,
        (size_t)length
    ) == 0);
}

void *mal_runtime_symbol_read(MalContext *context, const void *source, uint64_t length) {
    if (length == 0) {
        return NULL;
    }
    MalSymbol *result = mal_symbol_allocate(context, length);
    memcpy(result->bytes, source, (size_t)length);
    return result;
}

void mal_runtime_symbol_write(void *destination, const void *value) {
    uint64_t length = mal_runtime_symbol_length(value);
    if (length != 0) {
        memcpy(destination, ((const MalSymbol *)value)->bytes, (size_t)length);
    }
}
