#include "bytes_internal.h"

#include <string.h>

_Noreturn void mal_trap(MalContext *context, const char *message);

static void mal_symbol_set(
    MalBytesView *result,
    MalBytes *owner,
    const unsigned char *data,
    size_t length
) {
    *result = (MalBytesView){owner, data, length};
}

uint8_t mal_runtime_symbol_at(const void *data, size_t index) {
    return ((const unsigned char *)data)[index];
}

static MalBytes *mal_symbol_copy(
    MalContext *context,
    const void *left_data,
    size_t left_length,
    const void *right_data,
    size_t right_length
) {
    if (right_length > SIZE_MAX - left_length) {
        mal_trap(context, "symbol length overflow");
    }
    return mal_bytes_flat_concatenate(
        context,
        left_data,
        left_length,
        right_data,
        right_length,
        "symbol allocation failed"
    );
}

void mal_runtime_symbol_concatenate(
    MalContext *context,
    MalBytesView *result,
    const void *left_owner,
    const void *left_data,
    size_t left_length,
    const void *right_owner,
    const void *right_data,
    size_t right_length
) {
    if (left_length == 0) {
        mal_symbol_set(
            result,
            mal_bytes_retain(context, (MalBytes *)right_owner),
            right_data,
            right_length
        );
        return;
    }
    if (right_length == 0) {
        mal_symbol_set(
            result,
            mal_bytes_retain(context, (MalBytes *)left_owner),
            left_data,
            left_length
        );
        return;
    }
    MalBytes *owner = mal_symbol_copy(
        context,
        left_data,
        left_length,
        right_data,
        right_length
    );
    mal_symbol_set(result, owner, mal_bytes_data(owner), left_length + right_length);
}

void mal_runtime_symbol_concatenate_consuming_left(
    MalContext *context,
    MalBytesView *result,
    void *left_owner,
    const void *left_data,
    size_t left_length,
    const void *right_owner,
    const void *right_data,
    size_t right_length
) {
    if (left_length == 0) {
        mal_bytes_release(left_owner);
        mal_symbol_set(
            result,
            mal_bytes_retain(context, (MalBytes *)right_owner),
            right_data,
            right_length
        );
        return;
    }
    if (right_length == 0) {
        mal_symbol_set(result, left_owner, left_data, left_length);
        return;
    }
    MalBytes *owner = mal_bytes_append(
        context,
        left_owner,
        mal_bytes_offset(left_owner, left_data),
        left_length,
        right_data,
        right_length,
        "symbol allocation failed"
    );
    mal_symbol_set(result, owner, mal_bytes_data(owner), left_length + right_length);
}

void mal_runtime_symbol_concatenate_consuming_right(
    MalContext *context,
    MalBytesView *result,
    const void *left_owner,
    const void *left_data,
    size_t left_length,
    void *right_owner,
    const void *right_data,
    size_t right_length
) {
    if (left_length == 0) {
        mal_symbol_set(result, right_owner, right_data, right_length);
        return;
    }
    if (right_length == 0) {
        mal_bytes_release(right_owner);
        mal_symbol_set(
            result,
            mal_bytes_retain(context, (MalBytes *)left_owner),
            left_data,
            left_length
        );
        return;
    }
    MalBytes *owner = mal_bytes_prepend(
        context,
        right_owner,
        mal_bytes_offset(right_owner, right_data),
        right_length,
        left_data,
        left_length,
        "symbol allocation failed"
    );
    mal_symbol_set(result, owner, mal_bytes_data(owner), left_length + right_length);
}

uint8_t mal_runtime_symbol_equal(
    const void *left_data,
    size_t left_length,
    const void *right_data,
    size_t right_length
) {
    return left_length == right_length
        && (left_length == 0
            || left_data == right_data
            || memcmp(left_data, right_data, left_length) == 0);
}
