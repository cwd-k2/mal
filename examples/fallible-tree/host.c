#include "program.mal.h"

#include <stdint.h>
#include <stdlib.h>

typedef struct NodeAllocation NodeAllocation;

struct NodeAllocation {
    uint8_t *memory;
    NodeAllocation *next;
};

typedef struct {
    uint64_t limit;
    uint64_t live;
    NodeAllocation *first;
} AllocatorHandle;

static AllocatorHandle *allocator_handle(MalType_Allocator allocator) {
    return (AllocatorHandle *)mal_Allocator_bits(allocator);
}

MAL_DEFINE_createAllocator(context, limit) {
    AllocatorHandle *allocator = malloc(sizeof(*allocator));
    if (allocator == NULL) {
        mal_trap(context, "allocator creation failed");
    }
    allocator->limit = limit;
    allocator->live = UINT64_C(0);
    allocator->first = NULL;
    return mal_Allocator_from_bits((uintptr_t)allocator);
}

MAL_DEFINE_allocateNode(context, allocator, size) {
    AllocatorHandle *handle = allocator_handle(allocator);
    if (handle->live == handle->limit || size == 0 || size > SIZE_MAX) {
        return mal_NodeResult_make_1();
    }
    NodeAllocation *allocation = malloc(sizeof(*allocation));
    uint8_t *memory = malloc((size_t)size);
    if (allocation == NULL || memory == NULL) {
        free(allocation);
        free(memory);
        return mal_NodeResult_make_1();
    }
    allocation->memory = memory;
    allocation->next = handle->first;
    handle->first = allocation;
    ++handle->live;
    return mal_NodeResult_make_0(mal_Ptr_from_address(memory));
}

MAL_DEFINE_releaseNode(context, allocator, pointer) {
    AllocatorHandle *handle = allocator_handle(allocator);
    NodeAllocation **link = &handle->first;
    while (*link != NULL && (*link)->memory != mal_Ptr_address(pointer)) {
        link = &(*link)->next;
    }
    if (*link == NULL) {
        mal_trap(context, "attempted to release an unknown node");
    }
    NodeAllocation *allocation = *link;
    *link = allocation->next;
    free(allocation->memory);
    free(allocation);
    --handle->live;
}

MAL_DEFINE_destroyAllocator(context, allocator) {
    AllocatorHandle *handle = allocator_handle(allocator);
    if (handle->live != 0 || handle->first != NULL) {
        mal_trap(context, "allocator still owns live nodes");
    }
    free(handle);
}
