#include "bytes_internal.h"

#include <string.h>

_Noreturn void mal_trap(MalContext *context, const char *message);

static void mal_symbol_set(
    MalBytesView *result,
    MalBytes *owner,
    size_t offset,
    size_t length
) {
    *result = (MalBytesView){owner, offset, length};
}

uint8_t mal_runtime_symbol_at(const void *owner, size_t offset, size_t index) {
    return mal_bytes_data(owner)[offset + index];
}

static MalBytes *mal_symbol_copy(
    MalContext *context,
    const void *left_owner,
    size_t left_offset,
    size_t left_length,
    const void *right_owner,
    size_t right_offset,
    size_t right_length
) {
    if (right_length > SIZE_MAX - left_length) {
        mal_trap(context, "symbol length overflow");
    }
    return mal_bytes_flat_concatenate(
        context,
        mal_bytes_data(left_owner) + left_offset,
        left_length,
        mal_bytes_data(right_owner) + right_offset,
        right_length,
        "symbol allocation failed"
    );
}

void mal_runtime_symbol_concatenate(
    MalContext *context,
    MalBytesView *result,
    const void *left_owner,
    size_t left_offset,
    size_t left_length,
    const void *right_owner,
    size_t right_offset,
    size_t right_length
) {
    if (left_length == 0) {
        mal_symbol_set(result, mal_bytes_retain(context, (MalBytes *)right_owner), right_offset, right_length);
        return;
    }
    if (right_length == 0) {
        mal_symbol_set(result, mal_bytes_retain(context, (MalBytes *)left_owner), left_offset, left_length);
        return;
    }
    mal_symbol_set(
        result,
        mal_symbol_copy(context, left_owner, left_offset, left_length, right_owner, right_offset, right_length),
        0,
        left_length + right_length
    );
}

void mal_runtime_symbol_concatenate_consuming_left(
    MalContext *context,
    MalBytesView *result,
    void *left_owner,
    size_t left_offset,
    size_t left_length,
    const void *right_owner,
    size_t right_offset,
    size_t right_length
) {
    if (left_length == 0) {
        mal_bytes_release(left_owner);
        mal_symbol_set(result, mal_bytes_retain(context, (MalBytes *)right_owner), right_offset, right_length);
        return;
    }
    if (right_length == 0) {
        mal_symbol_set(result, left_owner, left_offset, left_length);
        return;
    }
    MalBytes *owner = mal_bytes_append(
        context,
        left_owner,
        left_offset,
        left_length,
        mal_bytes_data(right_owner) + right_offset,
        right_length,
        "symbol allocation failed"
    );
    mal_symbol_set(result, owner, 0, left_length + right_length);
}

void mal_runtime_symbol_concatenate_consuming_right(
    MalContext *context,
    MalBytesView *result,
    const void *left_owner,
    size_t left_offset,
    size_t left_length,
    void *right_owner,
    size_t right_offset,
    size_t right_length
) {
    if (left_length == 0) {
        mal_symbol_set(result, right_owner, right_offset, right_length);
        return;
    }
    if (right_length == 0) {
        mal_bytes_release(right_owner);
        mal_symbol_set(result, mal_bytes_retain(context, (MalBytes *)left_owner), left_offset, left_length);
        return;
    }
    MalBytes *owner = mal_bytes_prepend(
        context,
        right_owner,
        right_offset,
        right_length,
        mal_bytes_data(left_owner) + left_offset,
        left_length,
        "symbol allocation failed"
    );
    mal_symbol_set(result, owner, 0, left_length + right_length);
}

uint8_t mal_runtime_symbol_equal(
    const void *left_owner,
    size_t left_offset,
    size_t left_length,
    const void *right_owner,
    size_t right_offset,
    size_t right_length
) {
    return left_length == right_length
        && (left_length == 0
            || (left_owner == right_owner && left_offset == right_offset)
            || memcmp(
                mal_bytes_data(left_owner) + left_offset,
                mal_bytes_data(right_owner) + right_offset,
                left_length
            ) == 0);
}
