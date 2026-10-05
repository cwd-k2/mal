#include "host.mal.h"

#include <errno.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

static uint32_t io_error(void) {
    return errno == 0 ? (uint32_t)EIO : (uint32_t)errno;
}

MAL_DEFINE_readSource(call, path) {
    if (path.length == SIZE_MAX || memchr(path.data, '\0', path.length) != NULL) {
        return (mal_type(ReadResult)){ .tag = 1, .payload.variant_1 = (uint32_t)EINVAL };
    }
    char *terminated = malloc(path.length + 1);
    if (terminated == NULL) {
        return (mal_type(ReadResult)){ .tag = 1, .payload.variant_1 = (uint32_t)ENOMEM };
    }
    memcpy(terminated, path.data, path.length);
    terminated[path.length] = '\0';
    errno = 0;
    FILE *file = fopen(terminated, "rb");
    free(terminated);
    if (file == NULL) {
        return (mal_type(ReadResult)){ .tag = 1, .payload.variant_1 = io_error() };
    }
    size_t length = 0;
    size_t capacity = 4096;
    uint8_t *data = malloc(capacity);
    if (data == NULL) {
        fclose(file);
        return (mal_type(ReadResult)){ .tag = 1, .payload.variant_1 = (uint32_t)ENOMEM };
    }
    for (;;) {
        if (length == capacity) {
            if (capacity > SIZE_MAX / 2) {
                free(data);
                fclose(file);
                return (mal_type(ReadResult)){ .tag = 1, .payload.variant_1 = (uint32_t)EFBIG };
            }
            capacity *= 2;
            uint8_t *resized = realloc(data, capacity);
            if (resized == NULL) {
                free(data);
                fclose(file);
                return (mal_type(ReadResult)){ .tag = 1, .payload.variant_1 = (uint32_t)ENOMEM };
            }
            data = resized;
        }
        errno = 0;
        size_t transferred = fread(data + length, 1, capacity - length, file);
        length += transferred;
        if (transferred == 0) {
            if (ferror(file)) {
                uint32_t error = io_error();
                free(data);
                fclose(file);
                return (mal_type(ReadResult)){ .tag = 1, .payload.variant_1 = error };
            }
            break;
        }
    }
    errno = 0;
    if (fclose(file) != 0) {
        uint32_t error = io_error();
        free(data);
        return (mal_type(ReadResult)){ .tag = 1, .payload.variant_1 = error };
    }
    mal_type(Buffer) result = mal_buffer(call, mal_type(UInt8), length);
    if (length > 0) {
        mal_append(call, result, data, length);
    }
    free(data);
    return (mal_type(ReadResult)){ .tag = 0, .payload.variant_0 = mal_move(result) };
}

MAL_DEFINE_writeChunk(call, value) {
    FILE *stream = value.field_0 == 1 ? stdout : value.field_0 == 2 ? stderr : NULL;
    if (stream == NULL) {
        return (mal_type(WriteStatus)){ .tag = 1, .payload.variant_1 = (uint32_t)EINVAL };
    }
    errno = 0;
    if ((value.field_1.length > 0
            && fwrite(value.field_1.data, 1, value.field_1.length, stream)
                != value.field_1.length)
        || fflush(stream) != 0) {
        return (mal_type(WriteStatus)){ .tag = 1, .payload.variant_1 = io_error() };
    }
    return (mal_type(WriteStatus)){ .tag = 0, .payload.variant_0 = mal_unit };
}
