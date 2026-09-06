typedef struct MalAllocation {
    struct MalAllocation *next;
} MalAllocation;

struct MalContext {
    MalAllocation *allocations;
};

_Noreturn void mal_trap(MalContext *context, const char *message) {
    (void)context;
    fputs("mal trap: ", stderr);
    fputs(message, stderr);
    fputc('\n', stderr);
    abort();
}

static void mal_context_destroy(MalContext *context) {
    MalAllocation *allocation = context->allocations;
    while (allocation != NULL) {
        MalAllocation *next = allocation->next;
        free(allocation);
        allocation = next;
    }
}

static void *mal_allocate(MalContext *context, size_t size) {
    if (size > SIZE_MAX - sizeof(MalAllocation)) {
        mal_trap(context, "allocation size overflow");
    }
#ifdef MAL_TEST_FORCE_ALLOCATION_FAILURE
    MalAllocation *allocation = NULL;
#else
    MalAllocation *allocation = malloc(sizeof(MalAllocation) + size);
#endif
    if (allocation == NULL) {
        mal_trap(context, "allocation failed");
    }
    allocation->next = context->allocations;
    context->allocations = allocation;
    return allocation + 1;
}

MalType_Engram mal_Engram_copy_from_bytes(MalContext *context, const uint8_t *data, uint64_t length) {
    if (length == UINT64_C(0)) {
        return (MalType_Engram){ NULL, UINT64_C(0) };
    }
    if (data == NULL) {
        mal_trap(context, "null Engram data");
    }
    size_t size = (size_t)length;
    if ((uint64_t)size != length) {
        mal_trap(context, "allocation size overflow");
    }
    uint8_t *copy = (uint8_t *)mal_allocate(context, size);
    memcpy(copy, data, size);
    return (MalType_Engram){ copy, length };
}
