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

MAL_DEFINE_openReadOnly(call, path) {
    if (path.length == SIZE_MAX
        || (path.length > 0 && memchr(path.data, '\0', path.length) != NULL)) {
        return mal_OpenResult_return_1(call, (uint32_t)EINVAL);
    }
    char *terminated = malloc(path.length + 1);
    if (terminated == NULL) {
        mal_call_trap(call, "file path allocation failed");
    }
    if (path.length > 0) {
        memcpy(terminated, path.data, path.length);
    }
    terminated[path.length] = '\0';
    errno = 0;
    FILE *file = fopen(terminated, "rb");
    uint32_t error = io_error();
    free(terminated);
    if (file == NULL) {
        return mal_OpenResult_return_1(call, error);
    }
    return mal_OpenResult_return_0(call, mal_File_from_bits((uintptr_t)file));
}

MAL_DEFINE_readFile(call, file) {
    uint8_t bytes[4096];
    errno = 0;
    size_t length = fread(bytes, 1, sizeof(bytes), file_handle(file));
    if (ferror(file_handle(file))) {
        return mal_ReadResult_return_1(call, io_error());
    }
    mal_Buffer_t result = mal_Buffer_make(call, sizeof(uint8_t), length);
    for (size_t index = 0; index < length; ++index) {
        mal_Buffer_new(call, result, &bytes[index], sizeof(uint8_t));
    }
    return mal_ReadResult_return_0(call, result);
}

MAL_DEFINE_closeFile(call, file) {
    errno = 0;
    if (fclose(file_handle(file)) != 0) {
        return mal_CloseResult_return_1(call, io_error());
    }
    return mal_CloseResult_return_0(call);
}

MAL_DEFINE_writeBytes(call, value) {
    if (fwrite(value.data, 1, value.length, stdout) != value.length) {
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
