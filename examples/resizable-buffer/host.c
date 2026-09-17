#include "program.mal.h"

#include <errno.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

typedef struct RetiredStorage RetiredStorage;

struct RetiredStorage {
    uint8_t *memory;
    RetiredStorage *next;
};

typedef struct {
    uint8_t *memory;
    size_t capacity;
    size_t limit;
    RetiredStorage *retired;
} AllocationHandle;

static AllocationHandle *allocation_handle(mal_Allocation_t allocation) {
    return (AllocationHandle *)mal_Allocation_to_bits(allocation);
}

static mal_Buffer_t buffer_value(AllocationHandle *handle, size_t length) {
    return (mal_Buffer_t){
        .field_0 = mal_Allocation_from_bits((uintptr_t)handle),
        .field_1 = handle->memory,
        .field_2 = handle->capacity,
        .field_3 = length,
    };
}

static mal_Bool_t is_current_buffer(mal_Buffer_t buffer) {
    AllocationHandle *handle = allocation_handle(buffer.field_0);
    return handle->memory == buffer.field_1
        && handle->capacity == buffer.field_2
        && buffer.field_3 <= buffer.field_2
        ? mal_true
        : mal_false;
}

static mal_Bool_t is_current_slice(mal_Slice_t slice) {
    AllocationHandle *handle = allocation_handle(slice.field_0);
    uintptr_t base = (uintptr_t)handle->memory;
    uintptr_t address = (uintptr_t)slice.field_1;
    if (address < base || slice.field_2 > handle->capacity) {
        return mal_false;
    }
    return address - base <= handle->capacity - slice.field_2 ? mal_true : mal_false;
}

static void write_current_slice(
    mal_call_t *call,
    mal_Slice_t slice
) {
    if (!is_current_slice(slice)) {
        mal_call_trap(call, "attempted to use a stale slice");
    }
    if (fwrite(slice.field_1, 1, slice.field_2, stdout) != slice.field_2) {
        mal_call_trap(call, "cannot write stdout");
    }
}

MAL_DEFINE_allocateBuffer(call, value) {
    if (value.field_0 == 0 || value.field_0 > value.field_1) {
        return mal_BufferResult_return_1(call, (uint32_t)EINVAL);
    }
    AllocationHandle *handle = malloc(sizeof(*handle));
    uint8_t *memory = malloc(value.field_0);
    if (handle == NULL || memory == NULL) {
        free(handle);
        free(memory);
        return mal_BufferResult_return_1(call, (uint32_t)ENOMEM);
    }
    handle->memory = memory;
    handle->capacity = value.field_0;
    handle->limit = value.field_1;
    handle->retired = NULL;
    return mal_BufferResult_return_0(call, buffer_value(handle, 0));
}

MAL_DEFINE_resizeBuffer(call, value) {
    mal_Buffer_t buffer = value.field_0;
    size_t targetCapacity = value.field_1;
    if (!is_current_buffer(buffer)) {
        return mal_BufferResult_return_1(call, (uint32_t)EINVAL);
    }
    AllocationHandle *handle = allocation_handle(buffer.field_0);
    if (targetCapacity < buffer.field_3 || targetCapacity > handle->limit) {
        return mal_BufferResult_return_1(call, (uint32_t)EINVAL);
    }
    uint8_t *nextMemory = malloc(targetCapacity);
    RetiredStorage *retired = malloc(sizeof(*retired));
    if (nextMemory == NULL || retired == NULL) {
        free(nextMemory);
        free(retired);
        return mal_BufferResult_return_1(call, (uint32_t)ENOMEM);
    }
    if (buffer.field_3 > 0) {
        memcpy(nextMemory, handle->memory, buffer.field_3);
    }
    retired->memory = handle->memory;
    retired->next = handle->retired;
    handle->retired = retired;
    handle->memory = nextMemory;
    handle->capacity = targetCapacity;
    return mal_BufferResult_return_0(call, buffer_value(handle, buffer.field_3));
}

MAL_DEFINE_releaseBuffer(call, allocation) {
    AllocationHandle *handle = allocation_handle(allocation);
    RetiredStorage *retired = handle->retired;
    while (retired != NULL) {
        RetiredStorage *next = retired->next;
        free(retired->memory);
        free(retired);
        retired = next;
    }
    free(handle->memory);
    free(handle);
    return mal_Unit_return(call);
}

MAL_DEFINE_isCurrentBuffer(call, value) {
    return mal_Bool_return(call, is_current_buffer(value));
}

MAL_DEFINE_isCurrentSlice(call, value) {
    return mal_Bool_return(call, is_current_slice(value));
}

MAL_DEFINE_writeSlice(call, value) {
    write_current_slice(call, value);
    return mal_Unit_return(call);
}

MAL_DEFINE_writeSliceDescriptor(call, value) {
    uint8_t *address = value.field_1;
    void *memory;
    size_t length;
    memcpy(&memory, address, sizeof(memory));
    memcpy(&length, address + sizeof(memory), sizeof(length));
    write_current_slice(
        call,
        (mal_Slice_t){ .field_0 = value.field_0, .field_1 = memory, .field_2 = length }
    );
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
