#ifndef MAL_POOL_INTERNAL_H
#define MAL_POOL_INTERNAL_H

#include "runtime.h"

#include <stdint.h>

// The stable object stores the Header carrier in its trailing allocation and owns one replaceable allocation containing the
// occupancy bitmap followed by payload storage. Growth moves only the replaceable backing.
typedef struct {
    unsigned char *backing;
    unsigned char *payload;
    unsigned char *header;
    size_t capacity;
    size_t physical_capacity;
    size_t stride;
    size_t header_size;
} MalPool;

// Callback pairs are NULL for a trivial Header or element carrier. The base stays first so operations that only move
// carriers use the same object prefix for trivial and managed Pools.
typedef struct {
    MalPool pool;
    MalRuntimeRetain header_retain;
    MalRuntimeRelease header_release;
    MalRuntimeRetain element_retain;
    MalRuntimeRelease element_release;
} MalManagedPool;

#endif
