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
typedef struct { const uint8_t *data; uint64_t length; } MalString;
typedef struct { uint8_t *address; } MalPtr;

_Noreturn void mal_trap(MalContext *context, const char *message);
MalString mal_string_copy(MalContext *context, const uint8_t *data, uint64_t length);

static inline MalPtr mal_ptr_from_address(uint8_t *address) {
    return (MalPtr){ .address = address };
}

static inline uint8_t *mal_ptr_address(MalPtr value) {
    return value.address;
}

/* External operations */

int32_t mal_ext_inspect(
    MalContext *context,
    float argument_0,
    float argument_1,
    double argument_2,
    int64_t argument_3
);

/* External definition helpers */

#define MAL_HAS_EXTERN_inspect 1
#define MAL_DEFINE_inspect(context, argument_0, argument_1, argument_2, argument_3) \
    int32_t mal_ext_inspect( \
        MalContext *context MAL_MAYBE_UNUSED, \
        float argument_0, \
        float argument_1, \
        double argument_2, \
        int64_t argument_3 \
    )

#endif
