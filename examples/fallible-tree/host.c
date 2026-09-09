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

static AllocatorHandle *allocator_handle(mal_Allocator_t allocator) {
    return (AllocatorHandle *)mal_Allocator_to_bits(allocator);
}

MAL_DEFINE_createAllocator(call, limit) {
    AllocatorHandle *allocator = malloc(sizeof(*allocator));
    if (allocator == NULL) {
        mal_call_trap(call, "allocator creation failed");
    }
    allocator->limit = limit;
    allocator->live = UINT64_C(0);
    allocator->first = NULL;
    return mal_Allocator_return(call, mal_Allocator_from_bits((uintptr_t)allocator));
}

MAL_DEFINE_allocateNode(call, value) {
    AllocatorHandle *handle = allocator_handle(value.field_0);
    if (handle->live == handle->limit || value.field_1 == 0 || value.field_1 > SIZE_MAX) {
        return mal_NodeResult_return_1(call);
    }
    NodeAllocation *allocation = malloc(sizeof(*allocation));
    uint8_t *memory = malloc((size_t)value.field_1);
    if (allocation == NULL || memory == NULL) {
        free(allocation);
        free(memory);
        return mal_NodeResult_return_1(call);
    }
    allocation->memory = memory;
    allocation->next = handle->first;
    handle->first = allocation;
    ++handle->live;
    return mal_NodeResult_return_0(call, memory);
}

MAL_DEFINE_releaseNode(call, value) {
    AllocatorHandle *handle = allocator_handle(value.field_0);
    NodeAllocation **link = &handle->first;
    while (*link != NULL && (*link)->memory != value.field_1) {
        link = &(*link)->next;
    }
    if (*link == NULL) {
        mal_call_trap(call, "attempted to release an unknown node");
    }
    NodeAllocation *allocation = *link;
    *link = allocation->next;
    free(allocation->memory);
    free(allocation);
    --handle->live;
    return mal_Unit_return(call);
}

MAL_DEFINE_destroyAllocator(call, allocator) {
    AllocatorHandle *handle = allocator_handle(allocator);
    if (handle->live != 0 || handle->first != NULL) {
        mal_call_trap(call, "allocator still owns live nodes");
    }
    free(handle);
    return mal_Unit_return(call);
}
