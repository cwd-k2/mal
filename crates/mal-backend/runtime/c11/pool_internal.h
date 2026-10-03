#ifndef MAL_POOL_INTERNAL_H
#define MAL_POOL_INTERNAL_H

#include "runtime.h"

#include <stdint.h>

// The stable object stores metadata in its trailing allocation and owns one replaceable allocation containing the
// occupancy bitmap followed by payload storage. Growth moves only the replaceable backing.
typedef struct {
    unsigned char *backing;
    unsigned char *payload;
    unsigned char *metadata;
    size_t capacity;
    size_t physical_capacity;
    size_t stride;
    size_t metadata_size;
} MalPool;

// Callback pairs are NULL for a trivial metadata or element carrier. The base stays first so operations that only move
// carriers use the same object prefix for trivial and managed Pools.
typedef struct {
    MalPool pool;
    MalRuntimeRetain metadata_retain;
    MalRuntimeRelease metadata_release;
    MalRuntimeRetain element_retain;
    MalRuntimeRelease element_release;
} MalManagedPool;

#endif
