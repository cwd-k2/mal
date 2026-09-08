#ifndef MAL_PROGRAM_MAL_H
#define MAL_PROGRAM_MAL_H

#include <stdint.h>

#define MAL_C_ABI_VERSION 0x000500u
#define MAL_TYPE(name) MalType_##name
#define MAL_OPERATION(type, operation) mal_##type##_##operation
#define MAL_TAG(type, variant) MAL_##type##_TAG_##variant
#define MAL_EXTERN(name) mal_ext_##name
#define MAL_CLONE(owner) mal_##owner##_clone
#define MAL_MOVE(owner) mal_##owner##_take
#define MAL_DROP(owner) mal_##owner##_drop

#if defined(__clang__) || defined(__GNUC__)
#define MAL_DETAIL_MAYBE_UNUSED __attribute__((unused))
#else
#define MAL_DETAIL_MAYBE_UNUSED
#endif

/* Runtime API */

typedef struct MalContext MalContext;
typedef struct { uint8_t unused; } MalType_Unit;
typedef uint8_t MalType_Bool;
typedef int8_t MalType_Int8;
typedef int16_t MalType_Int16;
typedef int32_t MalType_Int32;
typedef int64_t MalType_Int64;
typedef uint8_t MalType_UInt8;
typedef uint16_t MalType_UInt16;
typedef uint32_t MalType_UInt32;
typedef uint64_t MalType_UInt64;
typedef float MalType_Float32;
typedef double MalType_Float64;
typedef struct { const uint8_t *data; uint64_t length; void *ownership; } MalType_Symbol;
typedef struct { uint8_t *address; } MalType_Ptr;
typedef struct { void *state; } MalSymbolAdmission;

#define MAL_FALSE (MalType_Bool)UINT8_C(0)
#define MAL_TRUE (MalType_Bool)UINT8_C(1)

_Noreturn void mal_trap(MalContext *context, const char *message);
MalSymbolAdmission mal_SymbolAdmission_begin(MalContext *context, uint64_t minimum_capacity);
uint64_t mal_SymbolAdmission_capacity(const MalSymbolAdmission *admission);
uint8_t *mal_SymbolAdmission_data(MalSymbolAdmission *admission);
void mal_SymbolAdmission_reserve(MalContext *context, MalSymbolAdmission *admission, uint64_t minimum_capacity);
MalType_Symbol mal_SymbolAdmission_finish(MalContext *context, MalSymbolAdmission *admission, uint64_t length);
void mal_SymbolAdmission_drop(MalContext *context, MalSymbolAdmission *admission);
MalType_Symbol mal_Symbol_clone(MalContext *context, MalType_Symbol value);
MalType_Symbol mal_Symbol_take(MalType_Symbol *value);
void mal_Symbol_drop(MalContext *context, MalType_Symbol *value);

static inline const uint8_t *mal_Symbol_data(MalType_Symbol value) {
    return value.data;
}

static inline uint64_t mal_Symbol_length(MalType_Symbol value) {
    return value.length;
}

static inline MalType_Ptr mal_Ptr_from_address(uint8_t *address) {
    return (MalType_Ptr){ .address = address };
}

static inline uint8_t *mal_Ptr_address(MalType_Ptr value) {
    return value.address;
}

/* Host-visible types */

typedef struct { uintptr_t bits; } MalType_Socket;

typedef struct MalRepr_Product_0 MalRepr_Product_0;
typedef struct MalRepr_Product_1 MalRepr_Product_1;
typedef struct MalRepr_Product_2 MalRepr_Product_2;
typedef struct MalRepr_Sum_3 MalRepr_Sum_3;
typedef struct MalRepr_Sum_4 MalRepr_Sum_4;

struct MalRepr_Product_0 {
    MalType_Socket field_0;
    MalType_Socket field_1;
};

struct MalRepr_Product_1 {
    MalType_UInt64 field_0;
    MalType_Symbol field_1;
};

struct MalRepr_Product_2 {
    MalType_Socket field_0;
    MalRepr_Product_1 field_1;
};

struct MalRepr_Sum_3 {
    uint32_t tag;
    union {
        MalType_Unit variant_0;
        MalType_UInt32 variant_1;
    } payload;
};

struct MalRepr_Sum_4 {
    uint32_t tag;
    union {
        MalRepr_Product_1 variant_0;
        MalType_UInt32 variant_1;
    } payload;
};

typedef MalRepr_Product_1 MalType_Packet;
typedef MalRepr_Product_0 MalType_SocketPair;
typedef MalRepr_Sum_3 MalType_Status;
typedef MalRepr_Sum_4 MalType_ReceiveResult;

/* Type helpers */

static inline MalType_Socket mal_Socket_from_bits(uintptr_t bits) {
    return (MalType_Socket){ .bits = bits };
}

static inline uintptr_t mal_Socket_bits(MalType_Socket value) {
    return value.bits;
}

static inline MalRepr_Product_1 mal_Repr_Product_1_clone(MalContext *context, MalRepr_Product_1 value) {
    value.field_1 = mal_Symbol_clone(context, value.field_1);
    return value;
}

static inline MalRepr_Product_1 mal_Repr_Product_1_take(MalRepr_Product_1 *value) {
    MalRepr_Product_1 result = *value;
    *value = (MalRepr_Product_1){ 0 };
    return result;
}

static inline void mal_Repr_Product_1_drop(MalContext *context, MalRepr_Product_1 *value) {
    mal_Symbol_drop(context, &value->field_1);
    *value = (MalRepr_Product_1){ 0 };
}

static inline MalRepr_Product_2 mal_Repr_Product_2_clone(MalContext *context, MalRepr_Product_2 value) {
    value.field_1 = mal_Repr_Product_1_clone(context, value.field_1);
    return value;
}

static inline MalRepr_Product_2 mal_Repr_Product_2_take(MalRepr_Product_2 *value) {
    MalRepr_Product_2 result = *value;
    *value = (MalRepr_Product_2){ 0 };
    return result;
}

static inline void mal_Repr_Product_2_drop(MalContext *context, MalRepr_Product_2 *value) {
    mal_Repr_Product_1_drop(context, &value->field_1);
    *value = (MalRepr_Product_2){ 0 };
}

static inline MalRepr_Sum_4 mal_Repr_Sum_4_clone(MalContext *context, MalRepr_Sum_4 value) {
    switch (value.tag) {
        case UINT32_C(0): {
            value.payload.variant_0 = mal_Repr_Product_1_clone(context, value.payload.variant_0);
            break;
        }
        case UINT32_C(1): {
            break;
        }
        default: {
            mal_trap(context, "invalid sum tag");
        }
    }
    return value;
}

static inline MalRepr_Sum_4 mal_Repr_Sum_4_take(MalRepr_Sum_4 *value) {
    MalRepr_Sum_4 result = *value;
    *value = (MalRepr_Sum_4){ 0 };
    return result;
}

static inline void mal_Repr_Sum_4_drop(MalContext *context, MalRepr_Sum_4 *value) {
    switch (value->tag) {
        case UINT32_C(0): {
            mal_Repr_Product_1_drop(context, &value->payload.variant_0);
            break;
        }
        case UINT32_C(1): {
            break;
        }
        default: {
            mal_trap(context, "invalid sum tag");
        }
    }
    *value = (MalRepr_Sum_4){ 0 };
}

static inline MalType_Packet mal_Packet_clone(MalContext *context, MalType_Packet value) {
    return mal_Repr_Product_1_clone(context, value);
}

static inline MalType_Packet mal_Packet_take(MalType_Packet *value) {
    return mal_Repr_Product_1_take(value);
}

static inline void mal_Packet_drop(MalContext *context, MalType_Packet *value) {
    mal_Repr_Product_1_drop(context, value);
}

static inline MalType_ReceiveResult mal_ReceiveResult_clone(MalContext *context, MalType_ReceiveResult value) {
    return mal_Repr_Sum_4_clone(context, value);
}

static inline MalType_ReceiveResult mal_ReceiveResult_take(MalType_ReceiveResult *value) {
    return mal_Repr_Sum_4_take(value);
}

static inline void mal_ReceiveResult_drop(MalContext *context, MalType_ReceiveResult *value) {
    mal_Repr_Sum_4_drop(context, value);
}

static inline MalType_Packet mal_Packet_make(MalType_UInt64 value_0, MalType_Symbol value_1) {
    return (MalType_Packet){ .field_0 = value_0, .field_1 = value_1 };
}

static inline MalType_UInt64 mal_Packet_get_0(MalType_Packet value) {
    return value.field_0;
}

static inline MalType_Symbol mal_Packet_get_1(MalType_Packet value) {
    return value.field_1;
}

static inline MalType_SocketPair mal_SocketPair_make(MalType_Socket value_0, MalType_Socket value_1) {
    return (MalType_SocketPair){ .field_0 = value_0, .field_1 = value_1 };
}

static inline MalType_Socket mal_SocketPair_get_0(MalType_SocketPair value) {
    return value.field_0;
}

static inline MalType_Socket mal_SocketPair_get_1(MalType_SocketPair value) {
    return value.field_1;
}

#define MAL_Status_TAG_0 UINT32_C(0)
#define MAL_Status_TAG_1 UINT32_C(1)
static inline uint32_t mal_Status_tag(MalType_Status value) {
    return value.tag;
}

static inline MalType_Bool mal_Status_is_0(MalType_Status value) {
    return value.tag == MAL_Status_TAG_0;
}

static inline MalType_Status mal_Status_make_0(void) {
    return (MalType_Status){ .tag = MAL_Status_TAG_0, .payload.variant_0 = (MalType_Unit){ .unused = UINT8_C(0) } };
}

static inline MalType_Bool mal_Status_is_1(MalType_Status value) {
    return value.tag == MAL_Status_TAG_1;
}

static inline MalType_Status mal_Status_make_1(MalType_UInt32 value) {
    return (MalType_Status){ .tag = MAL_Status_TAG_1, .payload.variant_1 = value };
}

static inline MalType_UInt32 mal_Status_expect_1(MalContext *context, MalType_Status value) {
    if (!mal_Status_is_1(value)) {
        mal_trap(context, "expected Status variant 1");
    }
    return value.payload.variant_1;
}

#define MAL_ReceiveResult_TAG_0 UINT32_C(0)
#define MAL_ReceiveResult_TAG_1 UINT32_C(1)
static inline uint32_t mal_ReceiveResult_tag(MalType_ReceiveResult value) {
    return value.tag;
}

static inline MalType_Bool mal_ReceiveResult_is_0(MalType_ReceiveResult value) {
    return value.tag == MAL_ReceiveResult_TAG_0;
}

static inline MalType_ReceiveResult mal_ReceiveResult_make_0(MalType_UInt64 value_0, MalType_Symbol value_1) {
    return (MalType_ReceiveResult){ .tag = MAL_ReceiveResult_TAG_0, .payload.variant_0 = (MalRepr_Product_1){ .field_0 = value_0, .field_1 = value_1 } };
}

static inline MalType_UInt64 mal_ReceiveResult_expect_0_0(MalContext *context, MalType_ReceiveResult value) {
    if (!mal_ReceiveResult_is_0(value)) {
        mal_trap(context, "expected ReceiveResult variant 0");
    }
    return value.payload.variant_0.field_0;
}

static inline MalType_Symbol mal_ReceiveResult_expect_0_1(MalContext *context, MalType_ReceiveResult value) {
    if (!mal_ReceiveResult_is_0(value)) {
        mal_trap(context, "expected ReceiveResult variant 0");
    }
    return value.payload.variant_0.field_1;
}

static inline MalType_Bool mal_ReceiveResult_is_1(MalType_ReceiveResult value) {
    return value.tag == MAL_ReceiveResult_TAG_1;
}

static inline MalType_ReceiveResult mal_ReceiveResult_make_1(MalType_UInt32 value) {
    return (MalType_ReceiveResult){ .tag = MAL_ReceiveResult_TAG_1, .payload.variant_1 = value };
}

static inline MalType_UInt32 mal_ReceiveResult_expect_1(MalContext *context, MalType_ReceiveResult value) {
    if (!mal_ReceiveResult_is_1(value)) {
        mal_trap(context, "expected ReceiveResult variant 1");
    }
    return value.payload.variant_1;
}

/* External operations */

MalType_SocketPair mal_ext_createSocketPair(MalContext *context);
MalType_Status mal_ext_sendPacket(MalContext *context, MalType_Socket argument_0, MalType_Packet argument_1);
MalType_ReceiveResult mal_ext_receivePacket(MalContext *context, MalType_Socket value);
MalType_Status mal_ext_closeSocket(MalContext *context, MalType_Socket value);
void mal_ext_writeError(MalContext *context, MalType_UInt32 value);

/* External definition helpers */

#define MAL_HAS_EXTERN_createSocketPair 1
#define MAL_DEFINE_createSocketPair(context) \
MalType_SocketPair mal_ext_createSocketPair( \
    MalContext *context MAL_DETAIL_MAYBE_UNUSED \
)

#define MAL_HAS_EXTERN_sendPacket 1
#define MAL_DEFINE_sendPacket(context, argument_0, argument_1) \
MalType_Status mal_ext_sendPacket( \
    MalContext *context MAL_DETAIL_MAYBE_UNUSED, \
    MalType_Socket argument_0, \
    MalType_Packet argument_1 \
)

#define MAL_HAS_EXTERN_receivePacket 1
#define MAL_DEFINE_receivePacket(context, value) \
MalType_ReceiveResult mal_ext_receivePacket( \
    MalContext *context MAL_DETAIL_MAYBE_UNUSED, \
    MalType_Socket value \
)

#define MAL_HAS_EXTERN_closeSocket 1
#define MAL_DEFINE_closeSocket(context, value) \
MalType_Status mal_ext_closeSocket( \
    MalContext *context MAL_DETAIL_MAYBE_UNUSED, \
    MalType_Socket value \
)

#define MAL_HAS_EXTERN_writeError 1
#define MAL_DEFINE_writeError(context, value) \
void mal_ext_writeError( \
    MalContext *context MAL_DETAIL_MAYBE_UNUSED, \
    MalType_UInt32 value \
)

#endif
