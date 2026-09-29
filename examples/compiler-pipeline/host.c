#include "host.mal.h"

#include <errno.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

static uint8_t transfer_buffer[4096];

static uint32_t io_error(void) {
    return errno == 0 ? (uint32_t)EIO : (uint32_t)errno;
}

MAL_DEFINE_transferBuffer(call) {
    return mal_TransferBuffer_return(
        call,
        (mal_TransferBuffer_t){
            .field_0 = transfer_buffer,
            .field_1 = sizeof(transfer_buffer),
        }
    );
}

MAL_DEFINE_readSource(call, path) {
    if (path.field_0 != transfer_buffer
        || path.field_1 >= sizeof(transfer_buffer)
        || memchr(path.field_0, '\0', path.field_1) != NULL) {
        return mal_ReadResult_return_1(call, (uint32_t)EINVAL);
    }

    char terminated[sizeof(transfer_buffer)];
    if (path.field_1 > 0) {
        memcpy(terminated, path.field_0, path.field_1);
    }
    terminated[path.field_1] = '\0';

    errno = 0;
    FILE *file = fopen(terminated, "rb");
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

    return mal_ReadResult_return_0(
        call,
        (mal_SourceBytes_t){
            .field_0 = mal_SourceAllocation_from_bits((uintptr_t)data),
            .field_1 = data,
            .field_2 = length,
        }
    );
}

MAL_DEFINE_releaseSource(call, allocation) {
    free((void *)mal_SourceAllocation_to_bits(allocation));
    return mal_Unit_return(call);
}

MAL_DEFINE_writeChunk(call, value) {
    FILE *stream = value.field_0 == 1 ? stdout : value.field_0 == 2 ? stderr : NULL;
    if (stream == NULL || value.field_1 != transfer_buffer
        || value.field_2 > sizeof(transfer_buffer)) {
        return mal_WriteStatus_return_1(call, (uint32_t)EINVAL);
    }
    errno = 0;
    if ((value.field_2 > 0
            && fwrite(value.field_1, 1, value.field_2, stream) != value.field_2)
        || fflush(stream) != 0) {
        return mal_WriteStatus_return_1(call, io_error());
    }
    return mal_WriteStatus_return_0(call);
}
