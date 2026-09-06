#include "program.mal.h"

#include <errno.h>
#include <inttypes.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

typedef struct {
    uint8_t *memory;
} AllocationHandle;

static FILE *file_handle(MalType_File file) {
    return (FILE *)mal_File_bits(file);
}

static uint32_t io_error(void) {
    return errno == 0 ? (uint32_t)EIO : (uint32_t)errno;
}

MAL_DEFINE_allocateBuffer(context, size) {
    if (size == 0 || size > SIZE_MAX) {
        mal_trap(context, "invalid allocation size");
    }
    AllocationHandle *allocation = malloc(sizeof(*allocation));
    uint8_t *memory = malloc((size_t)size);
    if (allocation == NULL || memory == NULL) {
        free(allocation);
        free(memory);
        mal_trap(context, "allocation failed");
    }
    allocation->memory = memory;
    MalType_Allocation handle = mal_Allocation_from_bits((uintptr_t)allocation);
    MalType_Buffer buffer = mal_Buffer_make(mal_Ptr_from_address(memory), size, UINT64_C(0));
    return mal_OwnedBuffer_make(handle, buffer);
}

MAL_DEFINE_releaseBuffer(context, allocation) {
    AllocationHandle *handle = (AllocationHandle *)mal_Allocation_bits(allocation);
    free(handle->memory);
    free(handle);
}

MAL_DEFINE_openReadOnly(context, path) {
    uint64_t length = mal_Symbol_length(path);
    if (length > SIZE_MAX - 1
        || (length > 0 && memchr(mal_Symbol_data(path), '\0', (size_t)length) != NULL)) {
        return mal_OpenResult_make_1((uint32_t)EINVAL);
    }
    char *terminated = malloc((size_t)length + 1);
    if (terminated == NULL) {
        mal_trap(context, "file path allocation failed");
    }
    if (length > 0) {
        memcpy(terminated, mal_Symbol_data(path), (size_t)length);
    }
    terminated[length] = '\0';
    FILE *file = fopen(terminated, "rb");
    uint32_t error = io_error();
    free(terminated);
    if (file == NULL) {
        return mal_OpenResult_make_1(error);
    }
    return mal_OpenResult_make_0(mal_File_from_bits((uintptr_t)file));
}

MAL_DEFINE_readFile(context, file, region) {
    MalType_Ptr memory = mal_Region_get_0(region);
    uint64_t capacity = mal_Region_get_1(region);
    if (capacity > SIZE_MAX) {
        return mal_ReadResult_make_1((uint32_t)EINVAL);
    }
    FILE *handle = file_handle(file);
    errno = 0;
    size_t length = fread(mal_Ptr_address(memory), 1, (size_t)capacity, handle);
    if (ferror(handle)) {
        return mal_ReadResult_make_1(io_error());
    }
    return mal_ReadResult_make_0((uint64_t)length);
}

MAL_DEFINE_closeFile(context, file) {
    errno = 0;
    if (fclose(file_handle(file)) != 0) {
        return mal_CloseResult_make_1(io_error());
    }
    return mal_CloseResult_make_0();
}

MAL_DEFINE_writeBytes(context, memory, length) {
    if (length > SIZE_MAX
        || fwrite(mal_Ptr_address(memory), 1, (size_t)length, stdout) != (size_t)length) {
        mal_trap(context, "cannot write stdout");
    }
}

MAL_DEFINE_writeSymbol(context, value) {
    uint64_t length = mal_Symbol_length(value);
    if (length > SIZE_MAX
        || fwrite(mal_Symbol_data(value), 1, (size_t)length, stdout) != (size_t)length) {
        mal_trap(context, "cannot write stdout");
    }
}

MAL_DEFINE_writeError(context, error) {
    if (fprintf(stderr, "file error: %" PRIu32 "\n", error) < 0) {
        mal_trap(context, "cannot write stderr");
    }
}
