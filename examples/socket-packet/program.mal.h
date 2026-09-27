#ifndef MAL_BUILD_UMBRELLA
#include <mal.h>
#include <string.h>

_Static_assert(MAL_C_ABI_VERSION == 0x000900u, "generated header requires mal C ABI 0x000900");
_Static_assert((sizeof((size_t)0) * CHAR_BIT) == 64, "size_t does not match the mal target pointer index width");

#ifndef MAL_GENERATED_INTERFACE_D29A56066CEDE3E1_H
#define MAL_GENERATED_INTERFACE_D29A56066CEDE3E1_H
/* Host-visible types */

typedef struct { uintptr_t bits; } MalType_Socket;

#ifndef MAL_DETAIL_RAW_REPR_8ae85a9b6e39c816_DECLARED
#define MAL_DETAIL_RAW_REPR_8ae85a9b6e39c816_DECLARED
typedef struct MalRepr_Product_8ae85a9b6e39c816 MalRepr_Product_8ae85a9b6e39c816;
#endif
#ifndef MAL_DETAIL_RAW_REPR_1e5c20e9f358150e_DECLARED
#define MAL_DETAIL_RAW_REPR_1e5c20e9f358150e_DECLARED
typedef struct MalRepr_Product_1e5c20e9f358150e MalRepr_Product_1e5c20e9f358150e;
#endif
#ifndef MAL_DETAIL_RAW_REPR_f9c88caf7e8dddf4_DECLARED
#define MAL_DETAIL_RAW_REPR_f9c88caf7e8dddf4_DECLARED
typedef struct MalRepr_Product_f9c88caf7e8dddf4 MalRepr_Product_f9c88caf7e8dddf4;
#endif
#ifndef MAL_DETAIL_RAW_REPR_4647c725c84fdeda_DECLARED
#define MAL_DETAIL_RAW_REPR_4647c725c84fdeda_DECLARED
typedef struct MalRepr_Sum_4647c725c84fdeda MalRepr_Sum_4647c725c84fdeda;
#endif
#ifndef MAL_DETAIL_RAW_REPR_cb68c98eb0579ca1_DECLARED
#define MAL_DETAIL_RAW_REPR_cb68c98eb0579ca1_DECLARED
typedef struct MalRepr_Product_cb68c98eb0579ca1 MalRepr_Product_cb68c98eb0579ca1;
#endif
#ifndef MAL_DETAIL_RAW_REPR_1e5f80e9f35aee05_DECLARED
#define MAL_DETAIL_RAW_REPR_1e5f80e9f35aee05_DECLARED
typedef struct MalRepr_Product_1e5f80e9f35aee05 MalRepr_Product_1e5f80e9f35aee05;
#endif
#ifndef MAL_DETAIL_RAW_REPR_df02f3d72e6af636_DECLARED
#define MAL_DETAIL_RAW_REPR_df02f3d72e6af636_DECLARED
typedef struct MalRepr_Sum_df02f3d72e6af636 MalRepr_Sum_df02f3d72e6af636;
#endif

#ifndef MAL_DETAIL_RAW_REPR_8ae85a9b6e39c816_DEFINED
#define MAL_DETAIL_RAW_REPR_8ae85a9b6e39c816_DEFINED
struct MalRepr_Product_8ae85a9b6e39c816 {
    MalType_Socket field_0;
    MalType_Socket field_1;
};

#endif

#ifndef MAL_DETAIL_RAW_REPR_1e5c20e9f358150e_DEFINED
#define MAL_DETAIL_RAW_REPR_1e5c20e9f358150e_DEFINED
struct MalRepr_Product_1e5c20e9f358150e {
    MalType_Address field_0;
    MalType_USize field_1;
};

#endif

#ifndef MAL_DETAIL_RAW_REPR_f9c88caf7e8dddf4_DEFINED
#define MAL_DETAIL_RAW_REPR_f9c88caf7e8dddf4_DEFINED
struct MalRepr_Product_f9c88caf7e8dddf4 {
    MalType_Socket field_0;
    MalType_UInt64 field_1;
    MalType_Address field_2;
    MalType_USize field_3;
};

#endif

#ifndef MAL_DETAIL_RAW_REPR_4647c725c84fdeda_DEFINED
#define MAL_DETAIL_RAW_REPR_4647c725c84fdeda_DEFINED
struct MalRepr_Sum_4647c725c84fdeda {
    uint32_t tag;
    union {
        MalType_Unit variant_0;
        MalType_UInt32 variant_1;
    } payload;
};

#endif

#ifndef MAL_DETAIL_RAW_REPR_cb68c98eb0579ca1_DEFINED
#define MAL_DETAIL_RAW_REPR_cb68c98eb0579ca1_DEFINED
struct MalRepr_Product_cb68c98eb0579ca1 {
    MalType_Socket field_0;
    MalType_Address field_1;
    MalType_USize field_2;
};

#endif

#ifndef MAL_DETAIL_RAW_REPR_1e5f80e9f35aee05_DEFINED
#define MAL_DETAIL_RAW_REPR_1e5f80e9f35aee05_DEFINED
struct MalRepr_Product_1e5f80e9f35aee05 {
    MalType_UInt64 field_0;
    MalType_USize field_1;
};

#endif

#ifndef MAL_DETAIL_RAW_REPR_df02f3d72e6af636_DEFINED
#define MAL_DETAIL_RAW_REPR_df02f3d72e6af636_DEFINED
struct MalRepr_Sum_df02f3d72e6af636 {
    uint32_t tag;
    union {
        MalRepr_Product_1e5f80e9f35aee05 variant_0;
        MalType_UInt32 variant_1;
    } payload;
};

#endif

typedef MalRepr_Product_8ae85a9b6e39c816 MalType_SocketPair;
typedef MalRepr_Sum_4647c725c84fdeda MalType_SocketStatus;
typedef MalRepr_Product_1e5f80e9f35aee05 MalType_ReceivedPacket;
typedef MalRepr_Sum_df02f3d72e6af636 MalType_ReceiveResult;
typedef MalRepr_Product_1e5c20e9f358150e MalType_PacketBuffer;

typedef struct { uintptr_t mal_detail_bits; } mal_Socket_t;
#ifndef MAL_DETAIL_HOST_REPR_8ae85a9b6e39c816_DECLARED
#define MAL_DETAIL_HOST_REPR_8ae85a9b6e39c816_DECLARED
typedef struct mal_detail_repr_product_8ae85a9b6e39c816 mal_repr_product_8ae85a9b6e39c816_t;
#endif
#ifndef MAL_DETAIL_HOST_REPR_1e5c20e9f358150e_DECLARED
#define MAL_DETAIL_HOST_REPR_1e5c20e9f358150e_DECLARED
typedef struct mal_detail_repr_product_1e5c20e9f358150e mal_repr_product_1e5c20e9f358150e_t;
#endif
#ifndef MAL_DETAIL_HOST_REPR_f9c88caf7e8dddf4_DECLARED
#define MAL_DETAIL_HOST_REPR_f9c88caf7e8dddf4_DECLARED
typedef struct mal_detail_repr_product_f9c88caf7e8dddf4 mal_repr_product_f9c88caf7e8dddf4_t;
#endif
#ifndef MAL_DETAIL_HOST_REPR_4647c725c84fdeda_DECLARED
#define MAL_DETAIL_HOST_REPR_4647c725c84fdeda_DECLARED
typedef struct mal_detail_repr_sum_4647c725c84fdeda mal_repr_sum_4647c725c84fdeda_t;
#endif
#ifndef MAL_DETAIL_HOST_REPR_cb68c98eb0579ca1_DECLARED
#define MAL_DETAIL_HOST_REPR_cb68c98eb0579ca1_DECLARED
typedef struct mal_detail_repr_product_cb68c98eb0579ca1 mal_repr_product_cb68c98eb0579ca1_t;
#endif
#ifndef MAL_DETAIL_HOST_REPR_1e5f80e9f35aee05_DECLARED
#define MAL_DETAIL_HOST_REPR_1e5f80e9f35aee05_DECLARED
typedef struct mal_detail_repr_product_1e5f80e9f35aee05 mal_repr_product_1e5f80e9f35aee05_t;
#endif
#ifndef MAL_DETAIL_HOST_REPR_df02f3d72e6af636_DECLARED
#define MAL_DETAIL_HOST_REPR_df02f3d72e6af636_DECLARED
typedef struct mal_detail_repr_sum_df02f3d72e6af636 mal_repr_sum_df02f3d72e6af636_t;
#endif
typedef mal_repr_product_8ae85a9b6e39c816_t mal_SocketPair_t;
typedef mal_repr_sum_4647c725c84fdeda_t mal_SocketStatus_t;
typedef mal_repr_product_1e5f80e9f35aee05_t mal_ReceivedPacket_t;
typedef mal_repr_sum_df02f3d72e6af636_t mal_ReceiveResult_t;
typedef mal_repr_product_1e5c20e9f358150e_t mal_PacketBuffer_t;

#ifndef MAL_DETAIL_HOST_REPR_8ae85a9b6e39c816_DEFINED
#define MAL_DETAIL_HOST_REPR_8ae85a9b6e39c816_DEFINED
struct mal_detail_repr_product_8ae85a9b6e39c816 {
    mal_Socket_t field_0;
    mal_Socket_t field_1;
};
#endif

#ifndef MAL_DETAIL_HOST_REPR_1e5c20e9f358150e_DEFINED
#define MAL_DETAIL_HOST_REPR_1e5c20e9f358150e_DEFINED
struct mal_detail_repr_product_1e5c20e9f358150e {
    mal_Address_t field_0;
    mal_USize_t field_1;
};
#endif

#ifndef MAL_DETAIL_HOST_REPR_f9c88caf7e8dddf4_DEFINED
#define MAL_DETAIL_HOST_REPR_f9c88caf7e8dddf4_DEFINED
struct mal_detail_repr_product_f9c88caf7e8dddf4 {
    mal_Socket_t field_0;
    mal_UInt64_t field_1;
    mal_Address_t field_2;
    mal_USize_t field_3;
};
#endif

#ifndef MAL_DETAIL_HOST_REPR_4647c725c84fdeda_DEFINED
#define MAL_DETAIL_HOST_REPR_4647c725c84fdeda_DEFINED
struct mal_detail_repr_sum_4647c725c84fdeda {
    uint32_t tag;
    union {
        mal_Unit_t variant_0;
        mal_UInt32_t variant_1;
    } payload;
};
#endif

#ifndef MAL_DETAIL_HOST_REPR_cb68c98eb0579ca1_DEFINED
#define MAL_DETAIL_HOST_REPR_cb68c98eb0579ca1_DEFINED
struct mal_detail_repr_product_cb68c98eb0579ca1 {
    mal_Socket_t field_0;
    mal_Address_t field_1;
    mal_USize_t field_2;
};
#endif

#ifndef MAL_DETAIL_HOST_REPR_1e5f80e9f35aee05_DEFINED
#define MAL_DETAIL_HOST_REPR_1e5f80e9f35aee05_DEFINED
struct mal_detail_repr_product_1e5f80e9f35aee05 {
    mal_UInt64_t field_0;
    mal_USize_t field_1;
};
#endif

#ifndef MAL_DETAIL_HOST_REPR_df02f3d72e6af636_DEFINED
#define MAL_DETAIL_HOST_REPR_df02f3d72e6af636_DEFINED
struct mal_detail_repr_sum_df02f3d72e6af636 {
    uint32_t tag;
    union {
        mal_repr_product_1e5f80e9f35aee05_t variant_0;
        mal_UInt32_t variant_1;
    } payload;
};
#endif

/* Type helpers */

#ifndef MAL_DETAIL_HOST_REPR_8ae85a9b6e39c816_HELPERS
#define MAL_DETAIL_HOST_REPR_8ae85a9b6e39c816_HELPERS
static inline MalRepr_Product_8ae85a9b6e39c816 mal_repr_product_8ae85a9b6e39c816_return(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, mal_repr_product_8ae85a9b6e39c816_t value) {
    return (MalRepr_Product_8ae85a9b6e39c816){ .field_0 = (MalType_Socket){ .bits = value.field_0.mal_detail_bits }, .field_1 = (MalType_Socket){ .bits = value.field_1.mal_detail_bits } };
}

#endif

#ifndef MAL_DETAIL_HOST_REPR_1e5c20e9f358150e_HELPERS
#define MAL_DETAIL_HOST_REPR_1e5c20e9f358150e_HELPERS
static inline MalRepr_Product_1e5c20e9f358150e mal_repr_product_1e5c20e9f358150e_return(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, mal_repr_product_1e5c20e9f358150e_t value) {
    return (MalRepr_Product_1e5c20e9f358150e){ .field_0 = mal_Address_return(call, value.field_0), .field_1 = value.field_1 };
}

#endif

#ifndef MAL_DETAIL_HOST_REPR_f9c88caf7e8dddf4_HELPERS
#define MAL_DETAIL_HOST_REPR_f9c88caf7e8dddf4_HELPERS
static inline MalRepr_Product_f9c88caf7e8dddf4 mal_repr_product_f9c88caf7e8dddf4_return(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, mal_repr_product_f9c88caf7e8dddf4_t value) {
    return (MalRepr_Product_f9c88caf7e8dddf4){ .field_0 = (MalType_Socket){ .bits = value.field_0.mal_detail_bits }, .field_1 = value.field_1, .field_2 = mal_Address_return(call, value.field_2), .field_3 = value.field_3 };
}

#endif

#ifndef MAL_DETAIL_HOST_REPR_4647c725c84fdeda_HELPERS
#define MAL_DETAIL_HOST_REPR_4647c725c84fdeda_HELPERS
static inline mal_repr_sum_4647c725c84fdeda_t mal_detail_to_host_4647c725c84fdeda(mal_call_t *call, MalRepr_Sum_4647c725c84fdeda value) {
    switch (value.tag) {
        case UINT32_C(0): {
            return (mal_repr_sum_4647c725c84fdeda_t){ .tag = UINT32_C(0), .payload.variant_0 = value.payload.variant_0 };
        }
        case UINT32_C(1): {
            return (mal_repr_sum_4647c725c84fdeda_t){ .tag = UINT32_C(1), .payload.variant_1 = value.payload.variant_1 };
        }
        default: {
            mal_call_trap(call, "invalid sum tag");
        }
    }
}

static inline MalRepr_Sum_4647c725c84fdeda mal_detail_to_raw_4647c725c84fdeda(mal_call_t *call, mal_repr_sum_4647c725c84fdeda_t value) {
    switch (value.tag) {
        case UINT32_C(0): {
            return (MalRepr_Sum_4647c725c84fdeda){ .tag = UINT32_C(0), .payload.variant_0 = (MalType_Unit){ 0 } };
        }
        case UINT32_C(1): {
            return (MalRepr_Sum_4647c725c84fdeda){ .tag = UINT32_C(1), .payload.variant_1 = value.payload.variant_1 };
        }
        default: {
            mal_call_trap(call, "invalid sum tag");
        }
    }
}

#define mal_repr_sum_4647c725c84fdeda_tag_0 UINT32_C(0)
static inline mal_repr_sum_4647c725c84fdeda_t mal_repr_sum_4647c725c84fdeda_make_0(void) {
    return (mal_repr_sum_4647c725c84fdeda_t){ .tag = mal_repr_sum_4647c725c84fdeda_tag_0, .payload.variant_0 = (mal_Unit_t){ 0 } };
}

static inline MalRepr_Sum_4647c725c84fdeda mal_repr_sum_4647c725c84fdeda_return_0(mal_call_t *call) {
    return mal_detail_to_raw_4647c725c84fdeda(call, (mal_repr_sum_4647c725c84fdeda_t){ .tag = mal_repr_sum_4647c725c84fdeda_tag_0, .payload.variant_0 = (mal_Unit_t){ 0 } });
}

#define mal_repr_sum_4647c725c84fdeda_tag_1 UINT32_C(1)
static inline mal_repr_sum_4647c725c84fdeda_t mal_repr_sum_4647c725c84fdeda_make_1(mal_UInt32_t value) {
    return (mal_repr_sum_4647c725c84fdeda_t){ .tag = mal_repr_sum_4647c725c84fdeda_tag_1, .payload.variant_1 = value };
}

static inline MalRepr_Sum_4647c725c84fdeda mal_repr_sum_4647c725c84fdeda_return_1(mal_call_t *call, mal_UInt32_t value) {
    return mal_detail_to_raw_4647c725c84fdeda(call, (mal_repr_sum_4647c725c84fdeda_t){ .tag = mal_repr_sum_4647c725c84fdeda_tag_1, .payload.variant_1 = value });
}

#endif

#ifndef MAL_DETAIL_HOST_REPR_cb68c98eb0579ca1_HELPERS
#define MAL_DETAIL_HOST_REPR_cb68c98eb0579ca1_HELPERS
static inline MalRepr_Product_cb68c98eb0579ca1 mal_repr_product_cb68c98eb0579ca1_return(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, mal_repr_product_cb68c98eb0579ca1_t value) {
    return (MalRepr_Product_cb68c98eb0579ca1){ .field_0 = (MalType_Socket){ .bits = value.field_0.mal_detail_bits }, .field_1 = mal_Address_return(call, value.field_1), .field_2 = value.field_2 };
}

#endif

#ifndef MAL_DETAIL_HOST_REPR_1e5f80e9f35aee05_HELPERS
#define MAL_DETAIL_HOST_REPR_1e5f80e9f35aee05_HELPERS
static inline MalRepr_Product_1e5f80e9f35aee05 mal_repr_product_1e5f80e9f35aee05_return(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, mal_repr_product_1e5f80e9f35aee05_t value) {
    return (MalRepr_Product_1e5f80e9f35aee05){ .field_0 = value.field_0, .field_1 = value.field_1 };
}

#endif

#ifndef MAL_DETAIL_HOST_REPR_df02f3d72e6af636_HELPERS
#define MAL_DETAIL_HOST_REPR_df02f3d72e6af636_HELPERS
static inline mal_repr_sum_df02f3d72e6af636_t mal_detail_to_host_df02f3d72e6af636(mal_call_t *call, MalRepr_Sum_df02f3d72e6af636 value) {
    switch (value.tag) {
        case UINT32_C(0): {
            return (mal_repr_sum_df02f3d72e6af636_t){ .tag = UINT32_C(0), .payload.variant_0 = (mal_repr_product_1e5f80e9f35aee05_t){ .field_0 = value.payload.variant_0.field_0, .field_1 = value.payload.variant_0.field_1 } };
        }
        case UINT32_C(1): {
            return (mal_repr_sum_df02f3d72e6af636_t){ .tag = UINT32_C(1), .payload.variant_1 = value.payload.variant_1 };
        }
        default: {
            mal_call_trap(call, "invalid sum tag");
        }
    }
}

static inline MalRepr_Sum_df02f3d72e6af636 mal_detail_to_raw_df02f3d72e6af636(mal_call_t *call, mal_repr_sum_df02f3d72e6af636_t value) {
    switch (value.tag) {
        case UINT32_C(0): {
            return (MalRepr_Sum_df02f3d72e6af636){ .tag = UINT32_C(0), .payload.variant_0 = (MalRepr_Product_1e5f80e9f35aee05){ .field_0 = value.payload.variant_0.field_0, .field_1 = value.payload.variant_0.field_1 } };
        }
        case UINT32_C(1): {
            return (MalRepr_Sum_df02f3d72e6af636){ .tag = UINT32_C(1), .payload.variant_1 = value.payload.variant_1 };
        }
        default: {
            mal_call_trap(call, "invalid sum tag");
        }
    }
}

#define mal_repr_sum_df02f3d72e6af636_tag_0 UINT32_C(0)
static inline mal_repr_sum_df02f3d72e6af636_t mal_repr_sum_df02f3d72e6af636_make_0(mal_repr_product_1e5f80e9f35aee05_t value) {
    return (mal_repr_sum_df02f3d72e6af636_t){ .tag = mal_repr_sum_df02f3d72e6af636_tag_0, .payload.variant_0 = value };
}

static inline MalRepr_Sum_df02f3d72e6af636 mal_repr_sum_df02f3d72e6af636_return_0(mal_call_t *call, mal_repr_product_1e5f80e9f35aee05_t value) {
    return mal_detail_to_raw_df02f3d72e6af636(call, (mal_repr_sum_df02f3d72e6af636_t){ .tag = mal_repr_sum_df02f3d72e6af636_tag_0, .payload.variant_0 = value });
}

#define mal_repr_sum_df02f3d72e6af636_tag_1 UINT32_C(1)
static inline mal_repr_sum_df02f3d72e6af636_t mal_repr_sum_df02f3d72e6af636_make_1(mal_UInt32_t value) {
    return (mal_repr_sum_df02f3d72e6af636_t){ .tag = mal_repr_sum_df02f3d72e6af636_tag_1, .payload.variant_1 = value };
}

static inline MalRepr_Sum_df02f3d72e6af636 mal_repr_sum_df02f3d72e6af636_return_1(mal_call_t *call, mal_UInt32_t value) {
    return mal_detail_to_raw_df02f3d72e6af636(call, (mal_repr_sum_df02f3d72e6af636_t){ .tag = mal_repr_sum_df02f3d72e6af636_tag_1, .payload.variant_1 = value });
}

#endif

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
    return (MalRepr_Product_8ae85a9b6e39c816){ .field_0 = (MalType_Socket){ .bits = value.field_0.mal_detail_bits }, .field_1 = (MalType_Socket){ .bits = value.field_1.mal_detail_bits } };
}

#define mal_SocketStatus_tag_0 UINT32_C(0)
static inline mal_SocketStatus_t mal_SocketStatus_make_0(void) {
    return (mal_SocketStatus_t){ .tag = mal_SocketStatus_tag_0, .payload.variant_0 = (mal_Unit_t){ 0 } };
}

static inline MalType_SocketStatus mal_SocketStatus_return_0(mal_call_t *call) {
    return mal_detail_to_raw_4647c725c84fdeda(call, (mal_SocketStatus_t){ .tag = mal_SocketStatus_tag_0, .payload.variant_0 = (mal_Unit_t){ 0 } });
}

#define mal_SocketStatus_tag_1 UINT32_C(1)
static inline mal_SocketStatus_t mal_SocketStatus_make_1(mal_UInt32_t value) {
    return (mal_SocketStatus_t){ .tag = mal_SocketStatus_tag_1, .payload.variant_1 = value };
}

static inline MalType_SocketStatus mal_SocketStatus_return_1(mal_call_t *call, mal_UInt32_t value) {
    return mal_detail_to_raw_4647c725c84fdeda(call, (mal_SocketStatus_t){ .tag = mal_SocketStatus_tag_1, .payload.variant_1 = value });
}

static inline MalType_ReceivedPacket mal_ReceivedPacket_return(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, mal_ReceivedPacket_t value) {
    return (MalRepr_Product_1e5f80e9f35aee05){ .field_0 = value.field_0, .field_1 = value.field_1 };
}

#define mal_ReceiveResult_tag_0 UINT32_C(0)
static inline mal_ReceiveResult_t mal_ReceiveResult_make_0(mal_ReceivedPacket_t value) {
    return (mal_ReceiveResult_t){ .tag = mal_ReceiveResult_tag_0, .payload.variant_0 = value };
}

static inline MalType_ReceiveResult mal_ReceiveResult_return_0(mal_call_t *call, mal_ReceivedPacket_t value) {
    return mal_detail_to_raw_df02f3d72e6af636(call, (mal_ReceiveResult_t){ .tag = mal_ReceiveResult_tag_0, .payload.variant_0 = value });
}

#define mal_ReceiveResult_tag_1 UINT32_C(1)
static inline mal_ReceiveResult_t mal_ReceiveResult_make_1(mal_UInt32_t value) {
    return (mal_ReceiveResult_t){ .tag = mal_ReceiveResult_tag_1, .payload.variant_1 = value };
}

static inline MalType_ReceiveResult mal_ReceiveResult_return_1(mal_call_t *call, mal_UInt32_t value) {
    return mal_detail_to_raw_df02f3d72e6af636(call, (mal_ReceiveResult_t){ .tag = mal_ReceiveResult_tag_1, .payload.variant_1 = value });
}

static inline MalType_PacketBuffer mal_PacketBuffer_return(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, mal_PacketBuffer_t value) {
    return (MalRepr_Product_1e5c20e9f358150e){ .field_0 = mal_Address_return(call, value.field_0), .field_1 = value.field_1 };
}

/* Canonical memory access */

#ifndef MAL_DETAIL_MEMORY_UInt8_HELPERS
#define MAL_DETAIL_MEMORY_UInt8_HELPERS
static inline mal_UInt8_t mal_detail_memory_read_UInt8(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, const uint8_t *source) {
    mal_UInt8_t value;
    memcpy(&value, source, sizeof(value));
    return value;
}

static inline void mal_detail_memory_write_UInt8(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, uint8_t *destination, mal_UInt8_t value) {
    memcpy(destination, &value, sizeof(value));
}

#endif

#ifndef MAL_DETAIL_MEMORY_UInt32_HELPERS
#define MAL_DETAIL_MEMORY_UInt32_HELPERS
static inline mal_UInt32_t mal_detail_memory_read_UInt32(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, const uint8_t *source) {
    mal_UInt32_t value;
    memcpy(&value, source, sizeof(value));
    return value;
}

static inline void mal_detail_memory_write_UInt32(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, uint8_t *destination, mal_UInt32_t value) {
    memcpy(destination, &value, sizeof(value));
}

#endif

#ifndef MAL_DETAIL_MEMORY_UInt64_HELPERS
#define MAL_DETAIL_MEMORY_UInt64_HELPERS
static inline mal_UInt64_t mal_detail_memory_read_UInt64(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, const uint8_t *source) {
    mal_UInt64_t value;
    memcpy(&value, source, sizeof(value));
    return value;
}

static inline void mal_detail_memory_write_UInt64(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, uint8_t *destination, mal_UInt64_t value) {
    memcpy(destination, &value, sizeof(value));
}

#endif

#ifndef MAL_DETAIL_MEMORY_Address_HELPERS
#define MAL_DETAIL_MEMORY_Address_HELPERS
static inline mal_Address_t mal_detail_memory_read_Address(mal_call_t *call, const uint8_t *source) {
    mal_Address_t value;
    memcpy(&value, source, sizeof(value));
    return mal_Address_return(call, value);
}

static inline void mal_detail_memory_write_Address(mal_call_t *call, uint8_t *destination, mal_Address_t value) {
    mal_Address_return(call, value);
    memcpy(destination, &value, sizeof(value));
}

#endif

#ifndef MAL_DETAIL_MEMORY_USize_HELPERS
#define MAL_DETAIL_MEMORY_USize_HELPERS
static inline mal_USize_t mal_detail_memory_read_USize(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, const uint8_t *source) {
    mal_USize_t value;
    memcpy(&value, source, sizeof(value));
    return value;
}

static inline void mal_detail_memory_write_USize(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, uint8_t *destination, mal_USize_t value) {
    memcpy(destination, &value, sizeof(value));
}

#endif

#ifndef MAL_DETAIL_MEMORY_REPR_1e5c20e9f358150e_HELPERS
#define MAL_DETAIL_MEMORY_REPR_1e5c20e9f358150e_HELPERS
static inline mal_repr_product_1e5c20e9f358150e_t mal_detail_memory_read_1e5c20e9f358150e(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, const uint8_t *source MAL_DETAIL_MAYBE_UNUSED) {
    mal_repr_product_1e5c20e9f358150e_t value;
    value.field_0 = mal_detail_memory_read_Address(call, source + 0);
    value.field_1 = mal_detail_memory_read_USize(call, source + 8);
    return value;
}

static inline void mal_detail_memory_write_1e5c20e9f358150e(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, uint8_t *destination MAL_DETAIL_MAYBE_UNUSED, mal_repr_product_1e5c20e9f358150e_t value) {
    mal_detail_memory_write_Address(call, destination + 0, value.field_0);
    mal_detail_memory_write_USize(call, destination + 8, value.field_1);
}

#endif

#ifndef MAL_DETAIL_MEMORY_REPR_4647c725c84fdeda_HELPERS
#define MAL_DETAIL_MEMORY_REPR_4647c725c84fdeda_HELPERS
static inline mal_repr_sum_4647c725c84fdeda_t mal_detail_memory_read_4647c725c84fdeda(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, const uint8_t *source MAL_DETAIL_MAYBE_UNUSED) {
    switch (mal_detail_memory_read_UInt8(call, source)) {
        case 0: {
            return (mal_repr_sum_4647c725c84fdeda_t){ .tag = UINT32_C(0), .payload.variant_0 = (mal_Unit_t){ 0 } };
        }
        case 1: {
            return (mal_repr_sum_4647c725c84fdeda_t){ .tag = UINT32_C(1), .payload.variant_1 = mal_detail_memory_read_UInt32(call, source + 4) };
        }
        default: {
            mal_call_trap(call, "invalid canonical sum tag");
        }
    }
}

static inline void mal_detail_memory_write_4647c725c84fdeda(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, uint8_t *destination MAL_DETAIL_MAYBE_UNUSED, mal_repr_sum_4647c725c84fdeda_t value) {
    switch (value.tag) {
        case UINT32_C(0): {
            mal_detail_memory_write_UInt8(call, destination, (mal_UInt8_t)value.tag);
            (void)value.payload.variant_0;
            return;
        }
        case UINT32_C(1): {
            mal_detail_memory_write_UInt8(call, destination, (mal_UInt8_t)value.tag);
            mal_detail_memory_write_UInt32(call, destination + 4, value.payload.variant_1);
            return;
        }
        default: {
            mal_call_trap(call, "invalid sum tag");
        }
    }
}

#endif

#ifndef MAL_DETAIL_MEMORY_REPR_1e5f80e9f35aee05_HELPERS
#define MAL_DETAIL_MEMORY_REPR_1e5f80e9f35aee05_HELPERS
static inline mal_repr_product_1e5f80e9f35aee05_t mal_detail_memory_read_1e5f80e9f35aee05(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, const uint8_t *source MAL_DETAIL_MAYBE_UNUSED) {
    mal_repr_product_1e5f80e9f35aee05_t value;
    value.field_0 = mal_detail_memory_read_UInt64(call, source + 0);
    value.field_1 = mal_detail_memory_read_USize(call, source + 8);
    return value;
}

static inline void mal_detail_memory_write_1e5f80e9f35aee05(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, uint8_t *destination MAL_DETAIL_MAYBE_UNUSED, mal_repr_product_1e5f80e9f35aee05_t value) {
    mal_detail_memory_write_UInt64(call, destination + 0, value.field_0);
    mal_detail_memory_write_USize(call, destination + 8, value.field_1);
}

#endif

#ifndef MAL_DETAIL_MEMORY_REPR_df02f3d72e6af636_HELPERS
#define MAL_DETAIL_MEMORY_REPR_df02f3d72e6af636_HELPERS
static inline mal_repr_sum_df02f3d72e6af636_t mal_detail_memory_read_df02f3d72e6af636(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, const uint8_t *source MAL_DETAIL_MAYBE_UNUSED) {
    switch (mal_detail_memory_read_UInt8(call, source)) {
        case 0: {
            return (mal_repr_sum_df02f3d72e6af636_t){ .tag = UINT32_C(0), .payload.variant_0 = mal_detail_memory_read_1e5f80e9f35aee05(call, source + 8) };
        }
        case 1: {
            return (mal_repr_sum_df02f3d72e6af636_t){ .tag = UINT32_C(1), .payload.variant_1 = mal_detail_memory_read_UInt32(call, source + 8) };
        }
        default: {
            mal_call_trap(call, "invalid canonical sum tag");
        }
    }
}

static inline void mal_detail_memory_write_df02f3d72e6af636(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, uint8_t *destination MAL_DETAIL_MAYBE_UNUSED, mal_repr_sum_df02f3d72e6af636_t value) {
    switch (value.tag) {
        case UINT32_C(0): {
            mal_detail_memory_write_UInt8(call, destination, (mal_UInt8_t)value.tag);
            mal_detail_memory_write_1e5f80e9f35aee05(call, destination + 8, value.payload.variant_0);
            return;
        }
        case UINT32_C(1): {
            mal_detail_memory_write_UInt8(call, destination, (mal_UInt8_t)value.tag);
            mal_detail_memory_write_UInt32(call, destination + 8, value.payload.variant_1);
            return;
        }
        default: {
            mal_call_trap(call, "invalid sum tag");
        }
    }
}

#endif

static inline mal_SocketStatus_t mal_SocketStatus_read(mal_call_t *call, mal_Address_t address, mal_USize_t index) {
    mal_Address_return(call, address);
    return mal_detail_memory_read_4647c725c84fdeda(call, (const uint8_t *)address + (index * 8));
}

static inline void mal_SocketStatus_write(mal_call_t *call, mal_Address_t address, mal_USize_t index, mal_SocketStatus_t value) {
    mal_Address_return(call, address);
    mal_detail_memory_write_4647c725c84fdeda(call, (uint8_t *)address + (index * 8), value);
}

static inline mal_ReceivedPacket_t mal_ReceivedPacket_read(mal_call_t *call, mal_Address_t address, mal_USize_t index) {
    mal_Address_return(call, address);
    return mal_detail_memory_read_1e5f80e9f35aee05(call, (const uint8_t *)address + (index * 16));
}

static inline void mal_ReceivedPacket_write(mal_call_t *call, mal_Address_t address, mal_USize_t index, mal_ReceivedPacket_t value) {
    mal_Address_return(call, address);
    mal_detail_memory_write_1e5f80e9f35aee05(call, (uint8_t *)address + (index * 16), value);
}

static inline mal_ReceiveResult_t mal_ReceiveResult_read(mal_call_t *call, mal_Address_t address, mal_USize_t index) {
    mal_Address_return(call, address);
    return mal_detail_memory_read_df02f3d72e6af636(call, (const uint8_t *)address + (index * 24));
}

static inline void mal_ReceiveResult_write(mal_call_t *call, mal_Address_t address, mal_USize_t index, mal_ReceiveResult_t value) {
    mal_Address_return(call, address);
    mal_detail_memory_write_df02f3d72e6af636(call, (uint8_t *)address + (index * 24), value);
}

static inline mal_PacketBuffer_t mal_PacketBuffer_read(mal_call_t *call, mal_Address_t address, mal_USize_t index) {
    mal_Address_return(call, address);
    return mal_detail_memory_read_1e5c20e9f358150e(call, (const uint8_t *)address + (index * 16));
}

static inline void mal_PacketBuffer_write(mal_call_t *call, mal_Address_t address, mal_USize_t index, mal_PacketBuffer_t value) {
    mal_Address_return(call, address);
    mal_detail_memory_write_1e5c20e9f358150e(call, (uint8_t *)address + (index * 16), value);
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
static MalType_SocketStatus mal_detail_sendPacket(mal_call_t *call, mal_repr_product_f9c88caf7e8dddf4_t value); \
MalType_SocketStatus mal_ext_sendPacket(MalContext *context MAL_DETAIL_MAYBE_UNUSED, MalType_Socket argument_0, MalType_UInt64 argument_1, MalType_Address argument_2, MalType_USize argument_3) { \
    mal_call_t call = (mal_call_t){ .mal_detail_context = context }; \
    return mal_detail_sendPacket(&call, (mal_repr_product_f9c88caf7e8dddf4_t){ .field_0 = (mal_Socket_t){ .mal_detail_bits = ((MalRepr_Product_f9c88caf7e8dddf4){ .field_0 = argument_0, .field_1 = argument_1, .field_2 = argument_2, .field_3 = argument_3 }).field_0.bits }, .field_1 = ((MalRepr_Product_f9c88caf7e8dddf4){ .field_0 = argument_0, .field_1 = argument_1, .field_2 = argument_2, .field_3 = argument_3 }).field_1, .field_2 = ((MalRepr_Product_f9c88caf7e8dddf4){ .field_0 = argument_0, .field_1 = argument_1, .field_2 = argument_2, .field_3 = argument_3 }).field_2, .field_3 = ((MalRepr_Product_f9c88caf7e8dddf4){ .field_0 = argument_0, .field_1 = argument_1, .field_2 = argument_2, .field_3 = argument_3 }).field_3 }); \
} \
static MalType_SocketStatus mal_detail_sendPacket( \
    mal_call_t *call, \
    mal_repr_product_f9c88caf7e8dddf4_t value \
)

#define MAL_HAS_EXTERN_receivePacket 1
#define MAL_DEFINE_receivePacket(call, value) \
static MalType_ReceiveResult mal_detail_receivePacket(mal_call_t *call, mal_repr_product_cb68c98eb0579ca1_t value); \
MalType_ReceiveResult mal_ext_receivePacket(MalContext *context MAL_DETAIL_MAYBE_UNUSED, MalType_Socket argument_0, MalType_Address argument_1, MalType_USize argument_2) { \
    mal_call_t call = (mal_call_t){ .mal_detail_context = context }; \
    return mal_detail_receivePacket(&call, (mal_repr_product_cb68c98eb0579ca1_t){ .field_0 = (mal_Socket_t){ .mal_detail_bits = ((MalRepr_Product_cb68c98eb0579ca1){ .field_0 = argument_0, .field_1 = argument_1, .field_2 = argument_2 }).field_0.bits }, .field_1 = ((MalRepr_Product_cb68c98eb0579ca1){ .field_0 = argument_0, .field_1 = argument_1, .field_2 = argument_2 }).field_1, .field_2 = ((MalRepr_Product_cb68c98eb0579ca1){ .field_0 = argument_0, .field_1 = argument_1, .field_2 = argument_2 }).field_2 }); \
} \
static MalType_ReceiveResult mal_detail_receivePacket( \
    mal_call_t *call, \
    mal_repr_product_cb68c98eb0579ca1_t value \
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
#endif
