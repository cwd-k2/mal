#include "program.mal.h"

#include <errno.h>
#include <inttypes.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

static FILE *file_handle(mal_File_t file) {
    return (FILE *)mal_File_to_bits(file);
}

static uint32_t io_error(void) {
    return errno == 0 ? (uint32_t)EIO : (uint32_t)errno;
}

MAL_DEFINE_allocateBuffer(call, size) {
    if (size == 0) {
        mal_call_trap(call, "invalid allocation size");
    }
    uint8_t *memory = malloc(size);
    if (memory == NULL) {
        mal_call_trap(call, "allocation failed");
    }
    return mal_OwnedBuffer_return(
        call,
        (mal_OwnedBuffer_t){
            .field_0 = mal_Allocation_from_bits((uintptr_t)memory),
            .field_1 = memory,
            .field_2 = size,
        }
    );
}

MAL_DEFINE_releaseBuffer(call, allocation) {
    free((void *)mal_Allocation_to_bits(allocation));
    return mal_Unit_return(call);
}

MAL_DEFINE_openReadOnly(call, path) {
    if (path.field_1 == SIZE_MAX
        || (path.field_1 > 0 && memchr(path.field_0, '\0', path.field_1) != NULL)) {
        return mal_OpenResult_return_1(call, (uint32_t)EINVAL);
    }
    char *terminated = malloc(path.field_1 + 1);
    if (terminated == NULL) {
        mal_call_trap(call, "file path allocation failed");
    }
    if (path.field_1 > 0) {
        memcpy(terminated, path.field_0, path.field_1);
    }
    terminated[path.field_1] = '\0';
    errno = 0;
    FILE *file = fopen(terminated, "rb");
    uint32_t error = io_error();
    free(terminated);
    if (file == NULL) {
        return mal_OpenResult_return_1(call, error);
    }
    return mal_OpenResult_return_0(call, mal_File_from_bits((uintptr_t)file));
}

MAL_DEFINE_readFile(call, value) {
    errno = 0;
    size_t length = fread(value.field_1, 1, value.field_2, file_handle(value.field_0));
    if (ferror(file_handle(value.field_0))) {
        return mal_ReadResult_return_1(call, io_error());
    }
    return mal_ReadResult_return_0(call, length);
}

MAL_DEFINE_closeFile(call, file) {
    errno = 0;
    if (fclose(file_handle(file)) != 0) {
        return mal_CloseResult_return_1(call, io_error());
    }
    return mal_CloseResult_return_0(call);
}

MAL_DEFINE_writeBytes(call, value) {
    if (fwrite(value.field_0, 1, value.field_1, stdout) != value.field_1) {
        mal_call_trap(call, "cannot write stdout");
    }
    return mal_Unit_return(call);
}

MAL_DEFINE_writeError(call, error) {
    if (fprintf(stderr, "file error: %" PRIu32 "\n", error) < 0) {
        mal_call_trap(call, "cannot write stderr");
    }
    return mal_Unit_return(call);
}
