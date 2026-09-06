#ifndef MAL_PROGRAM_MAL_H
#define MAL_PROGRAM_MAL_H

#include <stdint.h>

#define MAL_C_ABI_VERSION 0x000500u

#if defined(__clang__) || defined(__GNUC__)
#define MAL_MAYBE_UNUSED __attribute__((unused))
#else
#define MAL_MAYBE_UNUSED
#endif

/* Runtime API */

typedef struct MalContext MalContext;
typedef struct { uint8_t unused; } MalUnit;
typedef struct { const uint8_t *data; uint64_t length; } MalEngram;
typedef struct { uint8_t *address; } MalPtr;

_Noreturn void mal_trap(MalContext *context, const char *message);
MalEngram mal_engram_copy(MalContext *context, const uint8_t *data, uint64_t length);

static inline MalPtr mal_ptr_from_address(uint8_t *address) {
    return (MalPtr){ .address = address };
}

static inline uint8_t *mal_ptr_address(MalPtr value) {
    return value.address;
}
#endif
