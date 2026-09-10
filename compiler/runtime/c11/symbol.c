#include "runtime.h"

#include <stdint.h>
#include <stdlib.h>
#include <string.h>

typedef enum { MAL_SYMBOL_LEAF, MAL_SYMBOL_ROPE } MalSymbolKind;

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
} MalSymbolLeaf;

typedef struct {
    MalSymbol header;
    MalSymbol *left;
    MalSymbol *right;
    unsigned char *materialized;
} MalSymbolRope;

typedef struct {
    const MalSymbolRope *pending[UINT8_MAX + 1];
    size_t depth;
    const MalSymbolLeaf *leaf;
    uint64_t index;
} MalSymbolCursor;

_Noreturn void mal_trap(MalContext *context, const char *message);

static MalSymbol *mal_symbol_owner_retain(MalContext *context, MalSymbol *symbol);
static void mal_symbol_owner_release(MalSymbol *symbol);

_Static_assert(sizeof(MalSymbol) == 24, "symbol header layout mismatch");
_Static_assert(offsetof(MalSymbol, references) == 0, "symbol reference offset mismatch");
_Static_assert(offsetof(MalSymbol, length) == 8, "symbol length offset mismatch");
_Static_assert(offsetof(MalSymbol, kind) == 16, "symbol kind offset mismatch");
_Static_assert(offsetof(MalSymbol, height) == 17, "symbol height offset mismatch");
_Static_assert(offsetof(MalSymbolLeaf, bytes) == 24, "symbol byte offset mismatch");

static void *mal_symbol_allocate(MalContext *context, size_t size) {
    void *allocation = malloc(size);
    if (allocation == NULL) {
        mal_trap(context, "symbol allocation failed");
    }
    return allocation;
}

static MalSymbolLeaf *mal_symbol_leaf_allocate(MalContext *context, uint64_t length) {
    if (length > SIZE_MAX - sizeof(MalSymbolLeaf)) {
        mal_trap(context, "symbol allocation size overflow");
    }
    MalSymbolLeaf *leaf = mal_symbol_allocate(context, sizeof(MalSymbolLeaf) + (size_t)length);
    leaf->header = (MalSymbol){1, length, MAL_SYMBOL_LEAF, 0, {0}};
    return leaf;
}

static uint8_t mal_symbol_height(const MalSymbol *symbol) {
    return symbol == NULL ? 0 : symbol->height;
}

static MalSymbol *mal_symbol_node(MalContext *context, MalSymbol *left, MalSymbol *right) {
    MalSymbolRope *rope = mal_symbol_allocate(context, sizeof(MalSymbolRope));
    uint8_t left_height = mal_symbol_height(left);
    uint8_t right_height = mal_symbol_height(right);
    rope->header = (MalSymbol){
        1,
        left->length + right->length,
        MAL_SYMBOL_ROPE,
        (uint8_t)((left_height > right_height ? left_height : right_height) + 1),
        {0},
    };
    rope->left = mal_symbol_owner_retain(context, left);
    rope->right = mal_symbol_owner_retain(context, right);
    rope->materialized = NULL;
    return &rope->header;
}

static MalSymbol *mal_symbol_balance(MalContext *context, MalSymbol *left, MalSymbol *right) {
    uint8_t left_height = mal_symbol_height(left);
    uint8_t right_height = mal_symbol_height(right);
    if (left_height > (uint8_t)(right_height + 1)) {
        MalSymbolRope *outer = (MalSymbolRope *)left;
        if (mal_symbol_height(outer->left) >= mal_symbol_height(outer->right)) {
            MalSymbol *inner = mal_symbol_node(context, outer->right, right);
            MalSymbol *result = mal_symbol_node(context, outer->left, inner);
            mal_symbol_owner_release(inner);
            return result;
        }
        MalSymbolRope *middle = (MalSymbolRope *)outer->right;
        MalSymbol *new_left = mal_symbol_node(context, outer->left, middle->left);
        MalSymbol *new_right = mal_symbol_node(context, middle->right, right);
        MalSymbol *result = mal_symbol_node(context, new_left, new_right);
        mal_symbol_owner_release(new_right);
        mal_symbol_owner_release(new_left);
        return result;
    }
    if (right_height > (uint8_t)(left_height + 1)) {
        MalSymbolRope *outer = (MalSymbolRope *)right;
        if (mal_symbol_height(outer->right) >= mal_symbol_height(outer->left)) {
            MalSymbol *inner = mal_symbol_node(context, left, outer->left);
            MalSymbol *result = mal_symbol_node(context, inner, outer->right);
            mal_symbol_owner_release(inner);
            return result;
        }
        MalSymbolRope *middle = (MalSymbolRope *)outer->left;
        MalSymbol *new_left = mal_symbol_node(context, left, middle->left);
        MalSymbol *new_right = mal_symbol_node(context, middle->right, outer->right);
        MalSymbol *result = mal_symbol_node(context, new_left, new_right);
        mal_symbol_owner_release(new_right);
        mal_symbol_owner_release(new_left);
        return result;
    }
    return mal_symbol_node(context, left, right);
}

static MalSymbol *mal_symbol_join(MalContext *context, MalSymbol *left, MalSymbol *right) {
    uint8_t left_height = mal_symbol_height(left);
    uint8_t right_height = mal_symbol_height(right);
    if (left_height > (uint8_t)(right_height + 1)) {
        MalSymbolRope *rope = (MalSymbolRope *)left;
        MalSymbol *joined = mal_symbol_join(context, rope->right, right);
        MalSymbol *result = mal_symbol_balance(context, rope->left, joined);
        mal_symbol_owner_release(joined);
        return result;
    }
    if (right_height > (uint8_t)(left_height + 1)) {
        MalSymbolRope *rope = (MalSymbolRope *)right;
        MalSymbol *joined = mal_symbol_join(context, left, rope->left);
        MalSymbol *result = mal_symbol_balance(context, joined, rope->right);
        mal_symbol_owner_release(joined);
        return result;
    }
    return mal_symbol_node(context, left, right);
}

static MalSymbol *mal_symbol_owner_retain(MalContext *context, MalSymbol *symbol) {
    if (symbol == NULL || symbol->references == UINT64_MAX) {
        return symbol;
    }
    if (symbol->references == UINT64_MAX - 1) {
        mal_trap(context, "symbol reference count overflow");
    }
    ++symbol->references;
    return symbol;
}

static void mal_symbol_owner_release(MalSymbol *symbol) {
    if (symbol == NULL || symbol->references == UINT64_MAX) {
        return;
    }
    --symbol->references;
    if (symbol->references != 0) {
        return;
    }
    if (symbol->kind == MAL_SYMBOL_ROPE) {
        MalSymbolRope *rope = (MalSymbolRope *)symbol;
        mal_symbol_owner_release(rope->right);
        mal_symbol_owner_release(rope->left);
        free(rope->materialized);
    }
    free(symbol);
}

static void mal_symbol_cursor_descend(MalSymbolCursor *cursor, const MalSymbol *symbol) {
    while (symbol->kind == MAL_SYMBOL_ROPE) {
        const MalSymbolRope *rope = (const MalSymbolRope *)symbol;
        cursor->pending[cursor->depth++] = rope;
        symbol = rope->left;
    }
    cursor->leaf = (const MalSymbolLeaf *)symbol;
    cursor->index = 0;
}

static void mal_symbol_cursor_advance(MalSymbolCursor *cursor, uint64_t count) {
    cursor->index += count;
    if (cursor->index == cursor->leaf->header.length && cursor->depth != 0) {
        const MalSymbolRope *rope = cursor->pending[--cursor->depth];
        mal_symbol_cursor_descend(cursor, rope->right);
    }
}

uint64_t mal_runtime_symbol_length(const void *value) {
    return value == NULL ? 0 : ((const MalSymbol *)value)->length;
}

const uint8_t *mal_runtime_symbol_data(MalContext *context, void *value) {
    if (value == NULL) {
        return NULL;
    }
    MalSymbol *symbol = value;
    if (symbol->kind == MAL_SYMBOL_LEAF) {
        return ((MalSymbolLeaf *)symbol)->bytes;
    }
    MalSymbolRope *rope = (MalSymbolRope *)symbol;
    if (rope->materialized == NULL) {
        if (symbol->length > SIZE_MAX) {
            mal_trap(context, "symbol materialization size overflow");
        }
        rope->materialized = mal_symbol_allocate(context, (size_t)symbol->length);
        mal_runtime_symbol_write(rope->materialized, symbol);
    }
    return rope->materialized;
}

uint8_t mal_runtime_symbol_at(const void *value, uint64_t index) {
    const MalSymbol *symbol = value;
    while (symbol->kind == MAL_SYMBOL_ROPE) {
        const MalSymbolRope *rope = (const MalSymbolRope *)symbol;
        if (index < rope->left->length) {
            symbol = rope->left;
        } else {
            index -= rope->left->length;
            symbol = rope->right;
        }
    }
    return ((const MalSymbolLeaf *)symbol)->bytes[index];
}

void *mal_runtime_symbol_retain(MalContext *context, const void *value) {
    return mal_symbol_owner_retain(context, (MalSymbol *)value);
}

void mal_runtime_symbol_release(const void *value) {
    mal_symbol_owner_release((MalSymbol *)value);
}

void *mal_runtime_symbol_concatenate(MalContext *context, const void *left, const void *right) {
    uint64_t left_length = mal_runtime_symbol_length(left);
    uint64_t right_length = mal_runtime_symbol_length(right);
    if (left_length > UINT64_MAX - right_length) {
        mal_trap(context, "symbol length overflow");
    }
    if (left_length == 0) {
        return mal_symbol_owner_retain(context, (MalSymbol *)right);
    }
    if (right_length == 0) {
        return mal_symbol_owner_retain(context, (MalSymbol *)left);
    }
    return mal_symbol_join(context, (MalSymbol *)left, (MalSymbol *)right);
}

uint8_t mal_runtime_symbol_equal(const void *left, const void *right) {
    uint64_t length = mal_runtime_symbol_length(left);
    if (length != mal_runtime_symbol_length(right)) {
        return 0;
    }
    if (length == 0 || left == right) {
        return 1;
    }
    MalSymbolCursor left_cursor = {.depth = 0};
    MalSymbolCursor right_cursor = {.depth = 0};
    mal_symbol_cursor_descend(&left_cursor, left);
    mal_symbol_cursor_descend(&right_cursor, right);
    uint64_t compared = 0;
    while (compared < length) {
        uint64_t left_remaining = left_cursor.leaf->header.length - left_cursor.index;
        uint64_t right_remaining = right_cursor.leaf->header.length - right_cursor.index;
        uint64_t count = left_remaining < right_remaining ? left_remaining : right_remaining;
        if (memcmp(
                left_cursor.leaf->bytes + left_cursor.index,
                right_cursor.leaf->bytes + right_cursor.index,
                (size_t)count
            ) != 0) {
            return 0;
        }
        compared += count;
        mal_symbol_cursor_advance(&left_cursor, count);
        mal_symbol_cursor_advance(&right_cursor, count);
    }
    return 1;
}

void *mal_runtime_symbol_read(MalContext *context, const void *source, uint64_t length) {
    if (length == 0) {
        return NULL;
    }
    MalSymbolLeaf *result = mal_symbol_leaf_allocate(context, length);
    memcpy(result->bytes, source, (size_t)length);
    return &result->header;
}

void mal_runtime_symbol_write(void *destination, const void *value) {
    uint64_t length = mal_runtime_symbol_length(value);
    if (length == 0) {
        return;
    }
    MalSymbolCursor cursor = {.depth = 0};
    mal_symbol_cursor_descend(&cursor, value);
    uint64_t written = 0;
    while (written < length) {
        uint64_t count = cursor.leaf->header.length - cursor.index;
        memcpy(
            (unsigned char *)destination + written,
            cursor.leaf->bytes + cursor.index,
            (size_t)count
        );
        written += count;
        mal_symbol_cursor_advance(&cursor, count);
    }
}
