#include "program.mal.h"

#include <errno.h>
#include <inttypes.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

static FILE *file_handle(mal_type(File) file) {
    return (FILE *)mal_to_bits(file);
}

static uint32_t io_error(void) {
    return errno == 0 ? (uint32_t)EIO : (uint32_t)errno;
}

MAL_DEFINE_openReadOnly(call, path) {
    if (path.length == SIZE_MAX
        || (path.length > 0 && memchr(path.data, '\0', path.length) != NULL)) {
        return (mal_type(OpenResult)){ .tag = 1, .payload.variant_1 = (uint32_t)EINVAL };
    }
    char *terminated = malloc(path.length + 1);
    if (terminated == NULL) {
        mal_trap(call, "file path allocation failed");
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
        return (mal_type(OpenResult)){ .tag = 1, .payload.variant_1 = error };
    }
    return (mal_type(OpenResult)){
        .tag = 0,
        .payload.variant_0 = mal_from_bits(mal_type(File), (uintptr_t)file),
    };
}

MAL_DEFINE_readFile(call, file) {
    uint8_t bytes[4096];
    errno = 0;
    size_t length = fread(bytes, 1, sizeof(bytes), file_handle(file));
    if (ferror(file_handle(file))) {
        return (mal_type(ReadResult)){ .tag = 1, .payload.variant_1 = io_error() };
    }
    mal_type(Buffer) result = mal_buffer(call, mal_storageof(mal_type(UInt8)), length);
    if (length > 0) {
        mal_append(call, result, bytes, length);
    }
    return (mal_type(ReadResult)){ .tag = 0, .payload.variant_0 = mal_move(result) };
}

MAL_DEFINE_closeFile(call, file) {
    errno = 0;
    if (fclose(file_handle(file)) != 0) {
        return (mal_type(CloseResult)){ .tag = 1, .payload.variant_1 = io_error() };
    }
    return (mal_type(CloseResult)){ .tag = 0, .payload.variant_0 = mal_unit };
}

MAL_DEFINE_writeBytes(call, value) {
    if (fwrite(value.data, 1, value.length, stdout) != value.length) {
        mal_trap(call, "cannot write stdout");
    }
    return mal_unit;
}

MAL_DEFINE_writeError(call, error) {
    if (fprintf(stderr, "file error: %" PRIu32 "\n", error) < 0) {
        mal_trap(call, "cannot write stderr");
    }
    return mal_unit;
}
