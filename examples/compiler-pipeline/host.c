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
        return mal_ReadResult_return_1(call, (uint32_t)EINVAL);
    }
    char *terminated = malloc(path.length + 1);
    if (terminated == NULL) {
        return mal_ReadResult_return_1(call, (uint32_t)ENOMEM);
    }
    memcpy(terminated, path.data, path.length);
    terminated[path.length] = '\0';
    errno = 0;
    FILE *file = fopen(terminated, "rb");
    free(terminated);
    if (file == NULL) {
        return mal_ReadResult_return_1(call, io_error());
    }
    size_t length = 0;
    size_t capacity = 4096;
    uint8_t *data = malloc(capacity);
    if (data == NULL) {
        fclose(file);
        return mal_ReadResult_return_1(call, (uint32_t)ENOMEM);
    }
    for (;;) {
        if (length == capacity) {
            if (capacity > SIZE_MAX / 2) {
                free(data);
                fclose(file);
                return mal_ReadResult_return_1(call, (uint32_t)EFBIG);
            }
            capacity *= 2;
            uint8_t *resized = realloc(data, capacity);
            if (resized == NULL) {
                free(data);
                fclose(file);
                return mal_ReadResult_return_1(call, (uint32_t)ENOMEM);
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
                return mal_ReadResult_return_1(call, error);
            }
            break;
        }
    }
    errno = 0;
    if (fclose(file) != 0) {
        uint32_t error = io_error();
        free(data);
        return mal_ReadResult_return_1(call, error);
    }
    mal_Buffer_t result = mal_Buffer_make(call, sizeof(uint8_t), length);
    for (size_t index = 0; index < length; ++index) {
        mal_Buffer_new(call, result, &data[index], sizeof(uint8_t));
    }
    free(data);
    return mal_ReadResult_return_0(call, result);
}

MAL_DEFINE_writeChunk(call, value) {
    FILE *stream = value.field_0 == 1 ? stdout : value.field_0 == 2 ? stderr : NULL;
    if (stream == NULL) {
        return mal_WriteStatus_return_1(call, (uint32_t)EINVAL);
    }
    errno = 0;
    if ((value.field_1.length > 0
            && fwrite(value.field_1.data, 1, value.field_1.length, stream)
                != value.field_1.length)
        || fflush(stream) != 0) {
        return mal_WriteStatus_return_1(call, io_error());
    }
    return mal_WriteStatus_return_0(call);
}
