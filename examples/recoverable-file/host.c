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

static FILE *file_handle(mal_File_t file) {
    return (FILE *)mal_File_to_bits(file);
}

static uint32_t io_error(void) {
    return errno == 0 ? (uint32_t)EIO : (uint32_t)errno;
}

MAL_DEFINE_allocateBuffer(call, size) {
    if (size == 0 || size > SIZE_MAX) {
        mal_call_trap(call, "invalid allocation size");
    }
    AllocationHandle *allocation = malloc(sizeof(*allocation));
    uint8_t *memory = malloc((size_t)size);
    if (allocation == NULL || memory == NULL) {
        free(allocation);
        free(memory);
        mal_call_trap(call, "allocation failed");
    }
    allocation->memory = memory;
    mal_Allocation_t handle = mal_Allocation_from_bits((uintptr_t)allocation);
    mal_Buffer_t buffer = {
        .field_0 = memory,
        .field_1 = size,
        .field_2 = UINT64_C(0),
    };
    return mal_OwnedBuffer_return(
        call,
        (mal_OwnedBuffer_t){ .field_0 = handle, .field_1 = buffer }
    );
}

MAL_DEFINE_releaseBuffer(call, allocation) {
    AllocationHandle *handle = (AllocationHandle *)mal_Allocation_to_bits(allocation);
    free(handle->memory);
    free(handle);
    return mal_Unit_return(call);
}

MAL_DEFINE_openReadOnly(call, path) {
    mal_span_t bytes = mal_Symbol_to_bytes(call, path);
    uint64_t length = bytes.length;
    if (length > SIZE_MAX - 1
        || (length > 0 && memchr(bytes.data, '\0', (size_t)length) != NULL)) {
        return mal_OpenResult_return_1(call, (uint32_t)EINVAL);
    }
    char *terminated = malloc((size_t)length + 1);
    if (terminated == NULL) {
        mal_call_trap(call, "file path allocation failed");
    }
    if (length > 0) {
        memcpy(terminated, bytes.data, (size_t)length);
    }
    terminated[length] = '\0';
    FILE *file = fopen(terminated, "rb");
    uint32_t error = io_error();
    free(terminated);
    if (file == NULL) {
        return mal_OpenResult_return_1(call, error);
    }
    return mal_OpenResult_return_0(call, mal_File_from_bits((uintptr_t)file));
}

MAL_DEFINE_readFile(call, value) {
    FILE *handle = file_handle(value.field_0);
    void *memory = value.field_1.field_0;
    uint64_t capacity = value.field_1.field_1;
    if (capacity > SIZE_MAX) {
        return mal_ReadResult_return_1(call, (uint32_t)EINVAL);
    }
    errno = 0;
    size_t length = fread(memory, 1, (size_t)capacity, handle);
    if (ferror(handle)) {
        return mal_ReadResult_return_1(call, io_error());
    }
    return mal_ReadResult_return_0(call, (uint64_t)length);
}

MAL_DEFINE_closeFile(call, file) {
    errno = 0;
    if (fclose(file_handle(file)) != 0) {
        return mal_CloseResult_return_1(call, io_error());
    }
    return mal_CloseResult_return_0(call);
}

MAL_DEFINE_writeBytes(call, value) {
    if (value.field_1 > SIZE_MAX
        || fwrite(value.field_0, 1, (size_t)value.field_1, stdout) != (size_t)value.field_1) {
        mal_call_trap(call, "cannot write stdout");
    }
    return mal_Unit_return(call);
}

MAL_DEFINE_writeSymbol(call, value) {
    mal_span_t bytes = mal_Symbol_to_bytes(call, value);
    uint64_t length = bytes.length;
    if (length > SIZE_MAX
        || fwrite(bytes.data, 1, (size_t)length, stdout) != (size_t)length) {
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
