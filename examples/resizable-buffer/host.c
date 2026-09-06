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
    uint64_t capacity;
    uint64_t limit;
    RetiredStorage *retired;
} AllocationHandle;

static AllocationHandle *allocation_handle(MalType_Allocation allocation) {
    return (AllocationHandle *)mal_Allocation_bits(allocation);
}

static MalType_BufferResult buffer_error(uint32_t error) {
    return mal_BufferResult_make_1(error);
}

static MalType_BufferResult buffer_success(AllocationHandle *handle, uint64_t length) {
    return mal_BufferResult_make_0(
        mal_Allocation_from_bits((uintptr_t)handle),
        mal_Ptr_from_address(handle->memory),
        handle->capacity,
        length
    );
}

static MalType_Bool is_current_buffer(
    MalType_Allocation allocation,
    MalType_Ptr memory,
    uint64_t capacity,
    uint64_t length
) {
    AllocationHandle *handle = allocation_handle(allocation);
    return handle->memory == mal_Ptr_address(memory)
        && handle->capacity == capacity
        && length <= capacity
        ? MAL_TRUE
        : MAL_FALSE;
}

static MalType_Bool is_current_slice(
    MalType_Allocation allocation,
    MalType_Ptr memory,
    uint64_t length
) {
    AllocationHandle *handle = allocation_handle(allocation);
    uintptr_t base = (uintptr_t)handle->memory;
    uintptr_t address = (uintptr_t)mal_Ptr_address(memory);
    if (address < base || length > handle->capacity) {
        return MAL_FALSE;
    }
    return address - base <= handle->capacity - length ? MAL_TRUE : MAL_FALSE;
}

static void write_current_slice(
    MalContext *context,
    MalType_Allocation allocation,
    MalType_Ptr memory,
    uint64_t length
) {
    if (!is_current_slice(allocation, memory, length)) {
        mal_trap(context, "attempted to use a stale slice");
    }
    if (length > SIZE_MAX
        || fwrite(mal_Ptr_address(memory), 1, (size_t)length, stdout) != (size_t)length) {
        mal_trap(context, "cannot write stdout");
    }
}

MAL_DEFINE_allocateBuffer(context, initialCapacity, limit) {
    if (initialCapacity == 0 || initialCapacity > limit || initialCapacity > SIZE_MAX) {
        return buffer_error((uint32_t)EINVAL);
    }
    AllocationHandle *handle = malloc(sizeof(*handle));
    uint8_t *memory = malloc((size_t)initialCapacity);
    if (handle == NULL || memory == NULL) {
        free(handle);
        free(memory);
        return buffer_error((uint32_t)ENOMEM);
    }
    handle->memory = memory;
    handle->capacity = initialCapacity;
    handle->limit = limit;
    handle->retired = NULL;
    return buffer_success(handle, UINT64_C(0));
}

MAL_DEFINE_resizeBuffer(context, buffer, targetCapacity) {
    MalType_Allocation allocation = mal_Buffer_get_0(buffer);
    MalType_Ptr memory = mal_Buffer_get_1(buffer);
    uint64_t capacity = mal_Buffer_get_2(buffer);
    uint64_t length = mal_Buffer_get_3(buffer);
    if (!is_current_buffer(allocation, memory, capacity, length)) {
        return buffer_error((uint32_t)EINVAL);
    }
    AllocationHandle *handle = allocation_handle(allocation);
    if (targetCapacity < length || targetCapacity > handle->limit || targetCapacity > SIZE_MAX) {
        return buffer_error((uint32_t)EINVAL);
    }
    uint8_t *nextMemory = malloc((size_t)targetCapacity);
    RetiredStorage *retired = malloc(sizeof(*retired));
    if (nextMemory == NULL || retired == NULL) {
        free(nextMemory);
        free(retired);
        return buffer_error((uint32_t)ENOMEM);
    }
    if (length > 0) {
        memcpy(nextMemory, handle->memory, (size_t)length);
    }
    retired->memory = handle->memory;
    retired->next = handle->retired;
    handle->retired = retired;
    handle->memory = nextMemory;
    handle->capacity = targetCapacity;
    return buffer_success(handle, length);
}

MAL_DEFINE_releaseBuffer(context, allocation) {
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
}

MAL_DEFINE_isCurrentBuffer(context, allocation, memory, capacity, length) {
    return is_current_buffer(allocation, memory, capacity, length);
}

MAL_DEFINE_isCurrentSlice(context, allocation, memory, length) {
    return is_current_slice(allocation, memory, length);
}

MAL_DEFINE_writeSlice(context, allocation, memory, length) {
    write_current_slice(context, allocation, memory, length);
}

MAL_DEFINE_writeSliceDescriptor(context, allocation, descriptor) {
    uint8_t *address = mal_Ptr_address(descriptor);
    MalType_Ptr memory;
    uint64_t length;
    memcpy(&memory, address, sizeof(memory));
    memcpy(&length, address + sizeof(memory), sizeof(length));
    write_current_slice(context, allocation, memory, length);
}

MAL_DEFINE_writeSymbol(context, value) {
    uint64_t length = mal_Symbol_length(value);
    if (length > SIZE_MAX
        || fwrite(mal_Symbol_data(value), 1, (size_t)length, stdout) != (size_t)length) {
        mal_trap(context, "cannot write stdout");
    }
}
