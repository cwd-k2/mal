#ifndef MAL_SYMBOL_INTERNAL_H
#define MAL_SYMBOL_INTERNAL_H

#include "runtime.h"

#include <stdint.h>

typedef enum { MAL_SYMBOL_STATIC, MAL_SYMBOL_FLAT, MAL_SYMBOL_ROPE } MalSymbolKind;

enum { MAL_SYMBOL_FLAT_CONCATENATION_LIMIT = 256 };

typedef struct MalSymbol {
    uint64_t references;
    uint64_t length;
    uint8_t kind;
    uint8_t height;
    uint8_t reserved[6];
} MalSymbol;

typedef struct {
    MalSymbol header;
    unsigned char bytes[];
} MalSymbolStatic;

typedef struct {
    MalSymbol header;
    size_t capacity;
    size_t start;
    unsigned char bytes[];
} MalSymbolFlat;

typedef struct {
    MalSymbol header;
    MalSymbol *left;
    MalSymbol *right;
    unsigned char *materialized;
} MalSymbolRope;

typedef struct {
    const MalSymbolRope *pending[UINT8_MAX + 1];
    size_t depth;
    const MalSymbol *leaf;
    uint64_t index;
} MalSymbolCursor;

_Static_assert(sizeof(MalSymbol) == 24, "symbol header layout mismatch");
_Static_assert(offsetof(MalSymbol, references) == 0, "symbol reference offset mismatch");
_Static_assert(offsetof(MalSymbol, length) == 8, "symbol length offset mismatch");
_Static_assert(offsetof(MalSymbol, kind) == 16, "symbol kind offset mismatch");
_Static_assert(offsetof(MalSymbol, height) == 17, "symbol height offset mismatch");
_Static_assert(offsetof(MalSymbolStatic, bytes) == 24, "symbol byte offset mismatch");

#endif
