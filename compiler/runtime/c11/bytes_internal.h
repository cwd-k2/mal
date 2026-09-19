#ifndef MAL_BYTES_INTERNAL_H
#define MAL_BYTES_INTERNAL_H

#include "runtime.h"

#include <stdint.h>

typedef enum { MAL_BYTES_STATIC, MAL_BYTES_FLAT } MalBytesKind;

typedef struct MalBytes {
    uint64_t references;
    uint64_t length;
    uint8_t kind;
    uint8_t reserved[7];
} MalBytes;

typedef struct {
    MalBytes header;
    unsigned char bytes[];
} MalBytesStatic;

typedef struct {
    MalBytes header;
    size_t capacity;
    size_t start;
    unsigned char bytes[];
} MalBytesFlat;

_Static_assert(sizeof(MalBytes) == 24, "byte owner header layout mismatch");
_Static_assert(offsetof(MalBytes, references) == 0, "byte owner reference offset mismatch");
_Static_assert(offsetof(MalBytes, length) == 8, "byte owner length offset mismatch");
_Static_assert(offsetof(MalBytes, kind) == 16, "byte owner kind offset mismatch");
_Static_assert(offsetof(MalBytesStatic, bytes) == 24, "static byte offset mismatch");
_Static_assert(_Alignof(max_align_t) >= 8, "byte owner allocation alignment is insufficient");
_Static_assert(
    offsetof(MalBytesStatic, bytes) % 8 == 0,
    "static byte storage must preserve canonical alignment"
);
_Static_assert(
    offsetof(MalBytesFlat, bytes) % 8 == 0,
    "flat byte storage must preserve canonical alignment"
);

MalBytes *mal_bytes_flat_copy(
    MalContext *context,
    const void *source,
    size_t length,
    const char *allocation_failure
);
MalBytes *mal_bytes_flat_concatenate(
    MalContext *context,
    const unsigned char *left,
    size_t left_length,
    const unsigned char *right,
    size_t right_length,
    const char *allocation_failure
);
MalBytes *mal_bytes_retain(MalContext *context, MalBytes *owner);
void mal_bytes_release(MalBytes *owner);
const unsigned char *mal_bytes_data(const MalBytes *owner);
size_t mal_bytes_offset(const MalBytes *owner, const unsigned char *data);
MalBytes *mal_bytes_append(
    MalContext *context,
    MalBytes *owner,
    size_t offset,
    size_t length,
    const unsigned char *suffix,
    size_t suffix_length,
    const char *allocation_failure
);
MalBytes *mal_bytes_prepend(
    MalContext *context,
    MalBytes *owner,
    size_t offset,
    size_t length,
    const unsigned char *prefix,
    size_t prefix_length,
    const char *allocation_failure
);

#endif
