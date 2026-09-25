#include "program.mal.h"

#include <errno.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

static FILE *file_handle(mal_File_t file) {
    return (FILE *)mal_File_to_bits(file);
}

typedef struct Allocation Allocation;

struct Allocation {
    uint8_t *memory;
    Allocation *next;
};

typedef struct {
    Allocation *first;
} AllocatorHandle;

static uint8_t output_buffer[512];

static AllocatorHandle *allocator_handle(mal_Allocator_t allocator) {
    return (AllocatorHandle *)mal_Allocator_to_bits(allocator);
}

MAL_DEFINE_createAllocator(call) {
    AllocatorHandle *allocator = malloc(sizeof(*allocator));
    if (allocator == NULL) {
        mal_call_trap(call, "allocator creation failed");
    }
    allocator->first = NULL;
    return mal_Allocator_return(call, mal_Allocator_from_bits((uintptr_t)allocator));
}

MAL_DEFINE_allocateBuffer(call, value) {
    if (value.field_1 == 0) {
        mal_call_trap(call, "invalid allocation size");
    }
    Allocation *allocation = malloc(sizeof(*allocation));
    uint8_t *memory = malloc((size_t)value.field_1);
    if (allocation == NULL || memory == NULL) {
        free(allocation);
        free(memory);
        mal_call_trap(call, "allocation failed");
    }
    allocation->memory = memory;
    allocation->next = allocator_handle(value.field_0)->first;
    allocator_handle(value.field_0)->first = allocation;
    return mal_AllocatedBytes_return(
        call,
        (mal_AllocatedBytes_t){
            .field_0 = memory,
            .field_1 = value.field_1,
        }
    );
}

MAL_DEFINE_destroyAllocator(call, allocator) {
    AllocatorHandle *handle = allocator_handle(allocator);
    Allocation *allocation = handle->first;
    while (allocation != NULL) {
        Allocation *next = allocation->next;
        free(allocation->memory);
        free(allocation);
        allocation = next;
    }
    free(handle);
    return mal_Unit_return(call);
}

MAL_DEFINE_openReadWriteCreate(call, path) {
    const uint8_t *bytes = path.field_0;
    size_t length = path.field_1;
    if (length == SIZE_MAX) {
        mal_call_trap(call, "file path is too long");
    }
    if (length > 0 && memchr(bytes, '\0', length) != NULL) {
        mal_call_trap(call, "file path contains a null byte");
    }
    char *terminated = malloc((size_t)length + 1);
    if (terminated == NULL) {
        mal_call_trap(call, "file path allocation failed");
    }
    if (length > 0) {
        memcpy(terminated, bytes, length);
    }
    terminated[length] = '\0';
    FILE *file = fopen(terminated, "r+b");
    if (file == NULL && errno == ENOENT) {
        file = fopen(terminated, "w+b");
    }
    free(terminated);
    if (file == NULL) {
        mal_call_trap(call, "cannot open file");
    }
    return mal_File_return(call, mal_File_from_bits((uintptr_t)file));
}

MAL_DEFINE_standardInput(call) {
    return mal_File_return(call, mal_File_from_bits((uintptr_t)stdin));
}

MAL_DEFINE_readFile(call, value) {
    FILE *handle = file_handle(value.field_0);
    uint8_t *memory = value.field_1;
    size_t offset = value.field_2;
    size_t capacity = value.field_3;
    size_t length = fread(memory + offset, 1, capacity, handle);
    if (ferror(handle)) {
        mal_call_trap(call, "cannot read file");
    }
    return mal_USize_return(call, length);
}

MAL_DEFINE_writeFile(call, value) {
    FILE *handle = file_handle(value.field_0);
    const uint8_t *memory = value.field_1;
    size_t offset = value.field_2;
    size_t length = value.field_3;
    size_t written = fwrite(memory + offset, 1, length, handle);
    if (ferror(handle)) {
        mal_call_trap(call, "cannot write file");
    }
    return mal_USize_return(call, written);
}


MAL_DEFINE_rewindFile(call, file) {
    if (fseek(file_handle(file), 0, SEEK_SET) != 0) {
        mal_call_trap(call, "cannot rewind file");
    }
    return mal_Unit_return(call);
}

MAL_DEFINE_flushFile(call, file) {
    if (fflush(file_handle(file)) != 0) {
        mal_call_trap(call, "cannot flush file");
    }
    return mal_Unit_return(call);
}

MAL_DEFINE_closeFile(call, file) {
    if (fclose(file_handle(file)) != 0) {
        mal_call_trap(call, "cannot close file");
    }
    return mal_Unit_return(call);
}

MAL_DEFINE_outputBuffer(call) {
    return mal_OutputBuffer_return(
        call,
        (mal_OutputBuffer_t){
            .field_0 = output_buffer,
            .field_1 = sizeof(output_buffer),
        }
    );
}

MAL_DEFINE_writeStdout(call, value) {
    if (value.field_0 != output_buffer || value.field_1 > sizeof(output_buffer)
        || fwrite(value.field_0, 1, value.field_1, stdout) != value.field_1) {
        mal_call_trap(call, "cannot write stdout");
    }
    return mal_Unit_return(call);
}

MAL_DEFINE_writeStderr(call, value) {
    if (value.field_0 != output_buffer || value.field_1 > sizeof(output_buffer)
        || fwrite(value.field_0, 1, value.field_1, stderr) != value.field_1) {
        mal_call_trap(call, "cannot write stderr");
    }
    return mal_Unit_return(call);
}

MAL_DEFINE_failNow(call) {
    mal_call_trap(call, "host rejected the database");
}
