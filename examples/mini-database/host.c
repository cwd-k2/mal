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
    if (value.field_1 == 0 || value.field_1 > SIZE_MAX) {
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
    return mal_Buffer_return(
        call,
        (mal_Buffer_t){
            .field_0 = memory,
            .field_1 = value.field_1,
            .field_2 = UINT64_C(0),
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
    mal_span_t bytes = mal_Symbol_to_bytes(call, path);
    uint64_t length = bytes.length;
    if (length > SIZE_MAX - 1) {
        mal_call_trap(call, "file path is too long");
    }
    if (length > 0 && memchr(bytes.data, '\0', (size_t)length) != NULL) {
        mal_call_trap(call, "file path contains a null byte");
    }
    char *terminated = malloc((size_t)length + 1);
    if (terminated == NULL) {
        mal_call_trap(call, "file path allocation failed");
    }
    if (length > 0) {
        memcpy(terminated, bytes.data, (size_t)length);
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
    void *memory = value.field_1.field_0;
    uint64_t capacity = value.field_1.field_1;
    if (capacity > SIZE_MAX) {
        mal_call_trap(call, "file read capacity is too large");
    }
    size_t length = fread(memory, 1, (size_t)capacity, handle);
    if (ferror(handle)) {
        mal_call_trap(call, "cannot read file");
    }
    return mal_UInt64_return(call, (uint64_t)length);
}

MAL_DEFINE_writeFile(call, value) {
    FILE *handle = file_handle(value.field_0);
    void *memory = value.field_1.field_0;
    uint64_t length = value.field_1.field_1;
    if (length > SIZE_MAX) {
        mal_call_trap(call, "file write length is too large");
    }
    size_t written = fwrite(memory, 1, (size_t)length, handle);
    if (ferror(handle)) {
        mal_call_trap(call, "cannot write file");
    }
    return mal_UInt64_return(call, (uint64_t)written);
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

MAL_DEFINE_writeSymbol(call, value) {
    mal_span_t bytes = mal_Symbol_to_bytes(call, value);
    uint64_t length = bytes.length;
    if (length > SIZE_MAX
        || fwrite(bytes.data, 1, (size_t)length, stdout) != (size_t)length) {
        mal_call_trap(call, "cannot write stdout");
    }
    return mal_Unit_return(call);
}

MAL_DEFINE_writeBytes(call, value) {
    if (value.field_1 > SIZE_MAX
        || fwrite(value.field_0, 1, (size_t)value.field_1, stdout) != (size_t)value.field_1) {
        mal_call_trap(call, "cannot write stdout");
    }
    return mal_Unit_return(call);
}

MAL_DEFINE_fail(call, message) {
    mal_span_t bytes = mal_Symbol_to_bytes(call, message);
    if (bytes.length > 0) {
        fwrite(bytes.data, 1, (size_t)bytes.length, stderr);
        fputc('\n', stderr);
    }
    mal_call_trap(call, "host rejected the database");
}
