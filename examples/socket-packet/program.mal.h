#ifndef MAL_PROGRAM_MAL_H
#define MAL_PROGRAM_MAL_H

#include <stddef.h>
#include <stdint.h>
#include <limits.h>

#define MAL_C_ABI_VERSION 0x000800u

#if defined(__clang__)
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
typedef size_t MalType_ByteSize;
typedef size_t MalType_USize;
typedef void *MalType_Address;
_Static_assert((sizeof((size_t)0) * CHAR_BIT) == 64, "size_t does not match the mal target pointer index width");
typedef MalType_Unit mal_Unit_t;
typedef MalType_Bool mal_Bool_t;
typedef MalType_Int8 mal_Int8_t;
typedef MalType_Int16 mal_Int16_t;
typedef MalType_Int32 mal_Int32_t;
typedef MalType_Int64 mal_Int64_t;
typedef MalType_UInt8 mal_UInt8_t;
typedef MalType_UInt16 mal_UInt16_t;
typedef MalType_UInt32 mal_UInt32_t;
typedef MalType_UInt64 mal_UInt64_t;
typedef MalType_Float32 mal_Float32_t;
typedef MalType_Float64 mal_Float64_t;
typedef MalType_Address mal_Address_t;
typedef MalType_ByteSize mal_ByteSize_t;
typedef MalType_USize mal_USize_t;
typedef struct { MalContext *mal_detail_context; } mal_call_t;

#define mal_false (mal_Bool_t)UINT8_C(0)
#define mal_true (mal_Bool_t)UINT8_C(1)

_Noreturn void mal_trap(MalContext *context, const char *message);
static inline _Noreturn void mal_call_trap(mal_call_t *call, const char *message) {
    mal_trap(call->mal_detail_context, message);
}
static inline MalType_Unit mal_Unit_return(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED) {
    return (MalType_Unit){ .unused = UINT8_C(0) };
}
static inline MalType_Int8 mal_Int8_return(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, mal_Int8_t value) {
    return value;
}
static inline MalType_Int16 mal_Int16_return(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, mal_Int16_t value) {
    return value;
}
static inline MalType_Int32 mal_Int32_return(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, mal_Int32_t value) {
    return value;
}
static inline MalType_Int64 mal_Int64_return(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, mal_Int64_t value) {
    return value;
}
static inline MalType_UInt8 mal_UInt8_return(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, mal_UInt8_t value) {
    return value;
}
static inline MalType_UInt16 mal_UInt16_return(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, mal_UInt16_t value) {
    return value;
}
static inline MalType_UInt32 mal_UInt32_return(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, mal_UInt32_t value) {
    return value;
}
static inline MalType_UInt64 mal_UInt64_return(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, mal_UInt64_t value) {
    return value;
}
static inline MalType_Float32 mal_Float32_return(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, mal_Float32_t value) {
    return value;
}
static inline MalType_Float64 mal_Float64_return(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, mal_Float64_t value) {
    return value;
}
static inline MalType_ByteSize mal_ByteSize_return(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, mal_ByteSize_t value) {
    return value;
}
static inline MalType_USize mal_USize_return(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, mal_USize_t value) {
    return value;
}
static inline MalType_Address mal_Address_return(mal_call_t *call, mal_Address_t value) {
    if (value == 0) {
        mal_call_trap(call, "invalid Address result");
    }
    return value;
}
static inline MalType_Bool mal_Bool_return(mal_call_t *call, mal_Bool_t value) {
    if ((value != mal_false) && (value != mal_true)) {
        mal_call_trap(call, "invalid Bool result");
    }
    return value;
}

/* Host-visible types */

typedef struct { uintptr_t bits; } MalType_Socket;

typedef struct MalRepr_Product_0 MalRepr_Product_0;
typedef struct MalRepr_Product_1 MalRepr_Product_1;
typedef struct MalRepr_Product_2 MalRepr_Product_2;
typedef struct MalRepr_Sum_3 MalRepr_Sum_3;
typedef struct MalRepr_Product_4 MalRepr_Product_4;
typedef struct MalRepr_Product_5 MalRepr_Product_5;
typedef struct MalRepr_Sum_6 MalRepr_Sum_6;

struct MalRepr_Product_0 {
    MalType_Socket field_0;
    MalType_Socket field_1;
};

struct MalRepr_Product_1 {
    MalType_Address field_0;
    MalType_USize field_1;
};

struct MalRepr_Product_2 {
    MalType_Socket field_0;
    MalType_UInt64 field_1;
    MalType_Address field_2;
    MalType_USize field_3;
};

struct MalRepr_Sum_3 {
    uint32_t tag;
    union {
        MalType_Unit variant_0;
        MalType_UInt32 variant_1;
    } payload;
};

struct MalRepr_Product_4 {
    MalType_Socket field_0;
    MalType_Address field_1;
    MalType_USize field_2;
};

struct MalRepr_Product_5 {
    MalType_UInt64 field_0;
    MalType_USize field_1;
};

struct MalRepr_Sum_6 {
    uint32_t tag;
    union {
        MalRepr_Product_5 variant_0;
        MalType_UInt32 variant_1;
    } payload;
};

typedef MalRepr_Product_0 MalType_SocketPair;
typedef MalRepr_Sum_3 MalType_SocketStatus;
typedef MalRepr_Product_5 MalType_ReceivedPacket;
typedef MalRepr_Sum_6 MalType_ReceiveResult;
typedef MalRepr_Product_1 MalType_PacketBuffer;

typedef struct { uintptr_t mal_detail_bits; } mal_Socket_t;
typedef struct mal_detail_repr_product_0 mal_repr_product_0_t;
typedef struct mal_detail_repr_product_1 mal_repr_product_1_t;
typedef struct mal_detail_repr_product_2 mal_repr_product_2_t;
typedef struct mal_detail_repr_sum_3 mal_repr_sum_3_t;
typedef struct mal_detail_repr_product_4 mal_repr_product_4_t;
typedef struct mal_detail_repr_product_5 mal_repr_product_5_t;
typedef struct mal_detail_repr_sum_6 mal_repr_sum_6_t;
typedef mal_repr_product_0_t mal_SocketPair_t;
typedef mal_repr_sum_3_t mal_SocketStatus_t;
typedef mal_repr_product_5_t mal_ReceivedPacket_t;
typedef mal_repr_sum_6_t mal_ReceiveResult_t;
typedef mal_repr_product_1_t mal_PacketBuffer_t;

struct mal_detail_repr_product_0 {
    mal_Socket_t field_0;
    mal_Socket_t field_1;
};

struct mal_detail_repr_product_1 {
    mal_Address_t field_0;
    mal_USize_t field_1;
};

struct mal_detail_repr_product_2 {
    mal_Socket_t field_0;
    mal_UInt64_t field_1;
    mal_Address_t field_2;
    mal_USize_t field_3;
};

struct mal_detail_repr_sum_3 {
    uint32_t tag;
    union {
        mal_Unit_t variant_0;
        mal_UInt32_t variant_1;
    } payload;
};

struct mal_detail_repr_product_4 {
    mal_Socket_t field_0;
    mal_Address_t field_1;
    mal_USize_t field_2;
};

struct mal_detail_repr_product_5 {
    mal_UInt64_t field_0;
    mal_USize_t field_1;
};

struct mal_detail_repr_sum_6 {
    uint32_t tag;
    union {
        mal_repr_product_5_t variant_0;
        mal_UInt32_t variant_1;
    } payload;
};

/* Type helpers */

static inline MalRepr_Product_0 mal_repr_product_0_return(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, mal_repr_product_0_t value) {
    return (MalRepr_Product_0){ .field_0 = (MalType_Socket){ .bits = value.field_0.mal_detail_bits }, .field_1 = (MalType_Socket){ .bits = value.field_1.mal_detail_bits } };
}

static inline MalRepr_Product_1 mal_repr_product_1_return(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, mal_repr_product_1_t value) {
    return (MalRepr_Product_1){ .field_0 = mal_Address_return(call, value.field_0), .field_1 = value.field_1 };
}

static inline MalRepr_Product_2 mal_repr_product_2_return(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, mal_repr_product_2_t value) {
    return (MalRepr_Product_2){ .field_0 = (MalType_Socket){ .bits = value.field_0.mal_detail_bits }, .field_1 = value.field_1, .field_2 = mal_Address_return(call, value.field_2), .field_3 = value.field_3 };
}

static inline mal_repr_sum_3_t mal_detail_to_host_3(mal_call_t *call, MalRepr_Sum_3 value) {
    switch (value.tag) {
        case UINT32_C(0): {
            return (mal_repr_sum_3_t){ .tag = UINT32_C(0), .payload.variant_0 = value.payload.variant_0 };
        }
        case UINT32_C(1): {
            return (mal_repr_sum_3_t){ .tag = UINT32_C(1), .payload.variant_1 = value.payload.variant_1 };
        }
        default: {
            mal_call_trap(call, "invalid sum tag");
        }
    }
}

static inline MalRepr_Sum_3 mal_detail_to_raw_3(mal_call_t *call, mal_repr_sum_3_t value) {
    switch (value.tag) {
        case UINT32_C(0): {
            return (MalRepr_Sum_3){ .tag = UINT32_C(0), .payload.variant_0 = (MalType_Unit){ 0 } };
        }
        case UINT32_C(1): {
            return (MalRepr_Sum_3){ .tag = UINT32_C(1), .payload.variant_1 = value.payload.variant_1 };
        }
        default: {
            mal_call_trap(call, "invalid sum tag");
        }
    }
}

#define mal_repr_sum_3_tag_0 UINT32_C(0)
static inline mal_repr_sum_3_t mal_repr_sum_3_make_0(void) {
    return (mal_repr_sum_3_t){ .tag = mal_repr_sum_3_tag_0, .payload.variant_0 = (mal_Unit_t){ 0 } };
}

static inline MalRepr_Sum_3 mal_repr_sum_3_return_0(mal_call_t *call) {
    return mal_detail_to_raw_3(call, (mal_repr_sum_3_t){ .tag = mal_repr_sum_3_tag_0, .payload.variant_0 = (mal_Unit_t){ 0 } });
}

#define mal_repr_sum_3_tag_1 UINT32_C(1)
static inline mal_repr_sum_3_t mal_repr_sum_3_make_1(mal_UInt32_t value) {
    return (mal_repr_sum_3_t){ .tag = mal_repr_sum_3_tag_1, .payload.variant_1 = value };
}

static inline MalRepr_Sum_3 mal_repr_sum_3_return_1(mal_call_t *call, mal_UInt32_t value) {
    return mal_detail_to_raw_3(call, (mal_repr_sum_3_t){ .tag = mal_repr_sum_3_tag_1, .payload.variant_1 = value });
}

static inline MalRepr_Product_4 mal_repr_product_4_return(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, mal_repr_product_4_t value) {
    return (MalRepr_Product_4){ .field_0 = (MalType_Socket){ .bits = value.field_0.mal_detail_bits }, .field_1 = mal_Address_return(call, value.field_1), .field_2 = value.field_2 };
}

static inline MalRepr_Product_5 mal_repr_product_5_return(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, mal_repr_product_5_t value) {
    return (MalRepr_Product_5){ .field_0 = value.field_0, .field_1 = value.field_1 };
}

static inline mal_repr_sum_6_t mal_detail_to_host_6(mal_call_t *call, MalRepr_Sum_6 value) {
    switch (value.tag) {
        case UINT32_C(0): {
            return (mal_repr_sum_6_t){ .tag = UINT32_C(0), .payload.variant_0 = (mal_repr_product_5_t){ .field_0 = value.payload.variant_0.field_0, .field_1 = value.payload.variant_0.field_1 } };
        }
        case UINT32_C(1): {
            return (mal_repr_sum_6_t){ .tag = UINT32_C(1), .payload.variant_1 = value.payload.variant_1 };
        }
        default: {
            mal_call_trap(call, "invalid sum tag");
        }
    }
}

static inline MalRepr_Sum_6 mal_detail_to_raw_6(mal_call_t *call, mal_repr_sum_6_t value) {
    switch (value.tag) {
        case UINT32_C(0): {
            return (MalRepr_Sum_6){ .tag = UINT32_C(0), .payload.variant_0 = (MalRepr_Product_5){ .field_0 = value.payload.variant_0.field_0, .field_1 = value.payload.variant_0.field_1 } };
        }
        case UINT32_C(1): {
            return (MalRepr_Sum_6){ .tag = UINT32_C(1), .payload.variant_1 = value.payload.variant_1 };
        }
        default: {
            mal_call_trap(call, "invalid sum tag");
        }
    }
}

#define mal_repr_sum_6_tag_0 UINT32_C(0)
static inline mal_repr_sum_6_t mal_repr_sum_6_make_0(mal_repr_product_5_t value) {
    return (mal_repr_sum_6_t){ .tag = mal_repr_sum_6_tag_0, .payload.variant_0 = value };
}

static inline MalRepr_Sum_6 mal_repr_sum_6_return_0(mal_call_t *call, mal_repr_product_5_t value) {
    return mal_detail_to_raw_6(call, (mal_repr_sum_6_t){ .tag = mal_repr_sum_6_tag_0, .payload.variant_0 = value });
}

#define mal_repr_sum_6_tag_1 UINT32_C(1)
static inline mal_repr_sum_6_t mal_repr_sum_6_make_1(mal_UInt32_t value) {
    return (mal_repr_sum_6_t){ .tag = mal_repr_sum_6_tag_1, .payload.variant_1 = value };
}

static inline MalRepr_Sum_6 mal_repr_sum_6_return_1(mal_call_t *call, mal_UInt32_t value) {
    return mal_detail_to_raw_6(call, (mal_repr_sum_6_t){ .tag = mal_repr_sum_6_tag_1, .payload.variant_1 = value });
}

static inline mal_Socket_t mal_Socket_from_bits(uintptr_t bits) {
    return (mal_Socket_t){ .mal_detail_bits = bits };
}

static inline uintptr_t mal_Socket_to_bits(mal_Socket_t value) {
    return value.mal_detail_bits;
}

static inline MalType_Socket mal_Socket_return(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, mal_Socket_t value) {
    return (MalType_Socket){ .bits = value.mal_detail_bits };
}

static inline MalType_SocketPair mal_SocketPair_return(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, mal_SocketPair_t value) {
    return (MalRepr_Product_0){ .field_0 = (MalType_Socket){ .bits = value.field_0.mal_detail_bits }, .field_1 = (MalType_Socket){ .bits = value.field_1.mal_detail_bits } };
}

#define mal_SocketStatus_tag_0 UINT32_C(0)
static inline mal_SocketStatus_t mal_SocketStatus_make_0(void) {
    return (mal_SocketStatus_t){ .tag = mal_SocketStatus_tag_0, .payload.variant_0 = (mal_Unit_t){ 0 } };
}

static inline MalType_SocketStatus mal_SocketStatus_return_0(mal_call_t *call) {
    return mal_detail_to_raw_3(call, (mal_SocketStatus_t){ .tag = mal_SocketStatus_tag_0, .payload.variant_0 = (mal_Unit_t){ 0 } });
}

#define mal_SocketStatus_tag_1 UINT32_C(1)
static inline mal_SocketStatus_t mal_SocketStatus_make_1(mal_UInt32_t value) {
    return (mal_SocketStatus_t){ .tag = mal_SocketStatus_tag_1, .payload.variant_1 = value };
}

static inline MalType_SocketStatus mal_SocketStatus_return_1(mal_call_t *call, mal_UInt32_t value) {
    return mal_detail_to_raw_3(call, (mal_SocketStatus_t){ .tag = mal_SocketStatus_tag_1, .payload.variant_1 = value });
}

static inline MalType_ReceivedPacket mal_ReceivedPacket_return(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, mal_ReceivedPacket_t value) {
    return (MalRepr_Product_5){ .field_0 = value.field_0, .field_1 = value.field_1 };
}

#define mal_ReceiveResult_tag_0 UINT32_C(0)
static inline mal_ReceiveResult_t mal_ReceiveResult_make_0(mal_ReceivedPacket_t value) {
    return (mal_ReceiveResult_t){ .tag = mal_ReceiveResult_tag_0, .payload.variant_0 = value };
}

static inline MalType_ReceiveResult mal_ReceiveResult_return_0(mal_call_t *call, mal_ReceivedPacket_t value) {
    return mal_detail_to_raw_6(call, (mal_ReceiveResult_t){ .tag = mal_ReceiveResult_tag_0, .payload.variant_0 = value });
}

#define mal_ReceiveResult_tag_1 UINT32_C(1)
static inline mal_ReceiveResult_t mal_ReceiveResult_make_1(mal_UInt32_t value) {
    return (mal_ReceiveResult_t){ .tag = mal_ReceiveResult_tag_1, .payload.variant_1 = value };
}

static inline MalType_ReceiveResult mal_ReceiveResult_return_1(mal_call_t *call, mal_UInt32_t value) {
    return mal_detail_to_raw_6(call, (mal_ReceiveResult_t){ .tag = mal_ReceiveResult_tag_1, .payload.variant_1 = value });
}

static inline MalType_PacketBuffer mal_PacketBuffer_return(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, mal_PacketBuffer_t value) {
    return (MalRepr_Product_1){ .field_0 = mal_Address_return(call, value.field_0), .field_1 = value.field_1 };
}

/* External operations */

MalType_SocketPair mal_ext_createSocketPair(MalContext *context);
MalType_PacketBuffer mal_ext_packetBuffer(MalContext *context);
MalType_SocketStatus mal_ext_sendPacket(MalContext *context, MalType_Socket argument_0, MalType_UInt64 argument_1, MalType_Address argument_2, MalType_USize argument_3);
MalType_ReceiveResult mal_ext_receivePacket(MalContext *context, MalType_Socket argument_0, MalType_Address argument_1, MalType_USize argument_2);
MalType_SocketStatus mal_ext_closeSocket(MalContext *context, MalType_Socket value);
void mal_ext_writeError(MalContext *context, MalType_UInt32 value);

/* External definition helpers */

#define MAL_HAS_EXTERN_createSocketPair 1
#define MAL_DEFINE_createSocketPair(call) \
static MalType_SocketPair mal_detail_createSocketPair(mal_call_t *call); \
MalType_SocketPair mal_ext_createSocketPair(MalContext *context MAL_DETAIL_MAYBE_UNUSED) { \
    mal_call_t call = (mal_call_t){ .mal_detail_context = context }; \
    return mal_detail_createSocketPair(&call); \
} \
static MalType_SocketPair mal_detail_createSocketPair( \
    mal_call_t *call \
)

#define MAL_HAS_EXTERN_packetBuffer 1
#define MAL_DEFINE_packetBuffer(call) \
static MalType_PacketBuffer mal_detail_packetBuffer(mal_call_t *call); \
MalType_PacketBuffer mal_ext_packetBuffer(MalContext *context MAL_DETAIL_MAYBE_UNUSED) { \
    mal_call_t call = (mal_call_t){ .mal_detail_context = context }; \
    return mal_detail_packetBuffer(&call); \
} \
static MalType_PacketBuffer mal_detail_packetBuffer( \
    mal_call_t *call \
)

#define MAL_HAS_EXTERN_sendPacket 1
#define MAL_DEFINE_sendPacket(call, value) \
static MalType_SocketStatus mal_detail_sendPacket(mal_call_t *call, mal_repr_product_2_t value); \
MalType_SocketStatus mal_ext_sendPacket(MalContext *context MAL_DETAIL_MAYBE_UNUSED, MalType_Socket argument_0, MalType_UInt64 argument_1, MalType_Address argument_2, MalType_USize argument_3) { \
    mal_call_t call = (mal_call_t){ .mal_detail_context = context }; \
    return mal_detail_sendPacket(&call, (mal_repr_product_2_t){ .field_0 = (mal_Socket_t){ .mal_detail_bits = ((MalRepr_Product_2){ .field_0 = argument_0, .field_1 = argument_1, .field_2 = argument_2, .field_3 = argument_3 }).field_0.bits }, .field_1 = ((MalRepr_Product_2){ .field_0 = argument_0, .field_1 = argument_1, .field_2 = argument_2, .field_3 = argument_3 }).field_1, .field_2 = ((MalRepr_Product_2){ .field_0 = argument_0, .field_1 = argument_1, .field_2 = argument_2, .field_3 = argument_3 }).field_2, .field_3 = ((MalRepr_Product_2){ .field_0 = argument_0, .field_1 = argument_1, .field_2 = argument_2, .field_3 = argument_3 }).field_3 }); \
} \
static MalType_SocketStatus mal_detail_sendPacket( \
    mal_call_t *call, \
    mal_repr_product_2_t value \
)

#define MAL_HAS_EXTERN_receivePacket 1
#define MAL_DEFINE_receivePacket(call, value) \
static MalType_ReceiveResult mal_detail_receivePacket(mal_call_t *call, mal_repr_product_4_t value); \
MalType_ReceiveResult mal_ext_receivePacket(MalContext *context MAL_DETAIL_MAYBE_UNUSED, MalType_Socket argument_0, MalType_Address argument_1, MalType_USize argument_2) { \
    mal_call_t call = (mal_call_t){ .mal_detail_context = context }; \
    return mal_detail_receivePacket(&call, (mal_repr_product_4_t){ .field_0 = (mal_Socket_t){ .mal_detail_bits = ((MalRepr_Product_4){ .field_0 = argument_0, .field_1 = argument_1, .field_2 = argument_2 }).field_0.bits }, .field_1 = ((MalRepr_Product_4){ .field_0 = argument_0, .field_1 = argument_1, .field_2 = argument_2 }).field_1, .field_2 = ((MalRepr_Product_4){ .field_0 = argument_0, .field_1 = argument_1, .field_2 = argument_2 }).field_2 }); \
} \
static MalType_ReceiveResult mal_detail_receivePacket( \
    mal_call_t *call, \
    mal_repr_product_4_t value \
)

#define MAL_HAS_EXTERN_closeSocket 1
#define MAL_DEFINE_closeSocket(call, value) \
static MalType_SocketStatus mal_detail_closeSocket(mal_call_t *call, mal_Socket_t value); \
MalType_SocketStatus mal_ext_closeSocket(MalContext *context MAL_DETAIL_MAYBE_UNUSED, MalType_Socket value) { \
    mal_call_t call = (mal_call_t){ .mal_detail_context = context }; \
    return mal_detail_closeSocket(&call, (mal_Socket_t){ .mal_detail_bits = value.bits }); \
} \
static MalType_SocketStatus mal_detail_closeSocket( \
    mal_call_t *call, \
    mal_Socket_t value \
)

#define MAL_HAS_EXTERN_writeError 1
#define MAL_DEFINE_writeError(call, value) \
static MalType_Unit mal_detail_writeError(mal_call_t *call, mal_UInt32_t value); \
void mal_ext_writeError(MalContext *context MAL_DETAIL_MAYBE_UNUSED, MalType_UInt32 value) { \
    mal_call_t call = (mal_call_t){ .mal_detail_context = context }; \
    mal_detail_writeError(&call, value); \
} \
static MalType_Unit mal_detail_writeError( \
    mal_call_t *call, \
    mal_UInt32_t value \
)

#endif
