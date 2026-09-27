#ifndef MAL_PROGRAM_MAL_H
#define MAL_PROGRAM_MAL_H

#include <mal.h>

_Static_assert(MAL_C_ABI_VERSION == 0x000900u, "generated header requires mal C ABI 0x000900");
_Static_assert((sizeof((size_t)0) * CHAR_BIT) == 64, "size_t does not match the mal target pointer index width");

/* Host-visible types */

typedef struct MalRepr_Sum_0 MalRepr_Sum_0;
typedef struct MalRepr_Product_1 MalRepr_Product_1;
typedef struct MalRepr_Sum_2 MalRepr_Sum_2;
typedef struct MalRepr_Product_3 MalRepr_Product_3;
typedef struct MalRepr_Product_4 MalRepr_Product_4;
typedef struct MalRepr_Sum_5 MalRepr_Sum_5;
typedef struct MalRepr_Product_6 MalRepr_Product_6;
typedef struct MalRepr_Sum_7 MalRepr_Sum_7;
typedef struct MalRepr_Product_8 MalRepr_Product_8;
typedef struct MalRepr_Sum_9 MalRepr_Sum_9;
typedef struct MalRepr_Product_10 MalRepr_Product_10;
typedef struct MalRepr_Sum_11 MalRepr_Sum_11;

struct MalRepr_Sum_0 {
    uint32_t tag;
    union {
        MalType_Unit variant_0;
        MalType_Address variant_1;
    } payload;
};

struct MalRepr_Product_1 {
    MalRepr_Sum_0 field_0;
    MalType_ByteSize field_1;
    MalType_Int32 field_2;
    MalType_Int32 field_3;
    MalType_Int32 field_4;
    MalType_UInt64 field_5;
};

struct MalRepr_Sum_2 {
    uint32_t tag;
    union {
        MalType_Address variant_0;
        MalType_UInt32 variant_1;
    } payload;
};

struct MalRepr_Product_3 {
    MalType_Address field_0;
    MalType_ByteSize field_1;
    MalType_ByteSize field_2;
    MalType_Int32 field_3;
};

struct MalRepr_Product_4 {
    MalType_Address field_0;
    MalType_ByteSize field_1;
};

struct MalRepr_Sum_5 {
    uint32_t tag;
    union {
        MalType_Unit variant_0;
        MalType_UInt32 variant_1;
    } payload;
};

struct MalRepr_Product_6 {
    MalType_Int32 field_0;
    MalType_Address field_1;
    MalType_Int32 field_2;
    MalType_UInt32 field_3;
};

struct MalRepr_Sum_7 {
    uint32_t tag;
    union {
        MalType_Int32 variant_0;
        MalType_UInt32 variant_1;
    } payload;
};

struct MalRepr_Product_8 {
    MalType_Int32 field_0;
    MalType_Int64 field_1;
    MalType_Int32 field_2;
};

struct MalRepr_Sum_9 {
    uint32_t tag;
    union {
        MalType_UInt64 variant_0;
        MalType_UInt32 variant_1;
    } payload;
};

struct MalRepr_Product_10 {
    MalType_Int32 field_0;
    MalType_Address field_1;
    MalType_ByteSize field_2;
    MalType_ByteSize field_3;
};

struct MalRepr_Sum_11 {
    uint32_t tag;
    union {
        MalType_ByteSize variant_0;
        MalType_UInt32 variant_1;
    } payload;
};

typedef MalRepr_Sum_0 MalType_OptionalAddress;
typedef MalRepr_Product_1 MalType_MapRequest;
typedef MalRepr_Product_3 MalType_ResizeRequest;
typedef MalRepr_Sum_2 MalType_AddressResult;
typedef MalRepr_Sum_7 MalType_FileDescriptorResult;
typedef MalRepr_Sum_9 MalType_OffsetResult;
typedef MalRepr_Sum_11 MalType_TransferResult;
typedef MalRepr_Sum_5 MalType_SyscallStatus;

typedef struct mal_detail_repr_sum_0 mal_repr_sum_0_t;
typedef struct mal_detail_repr_product_1 mal_repr_product_1_t;
typedef struct mal_detail_repr_sum_2 mal_repr_sum_2_t;
typedef struct mal_detail_repr_product_3 mal_repr_product_3_t;
typedef struct mal_detail_repr_product_4 mal_repr_product_4_t;
typedef struct mal_detail_repr_sum_5 mal_repr_sum_5_t;
typedef struct mal_detail_repr_product_6 mal_repr_product_6_t;
typedef struct mal_detail_repr_sum_7 mal_repr_sum_7_t;
typedef struct mal_detail_repr_product_8 mal_repr_product_8_t;
typedef struct mal_detail_repr_sum_9 mal_repr_sum_9_t;
typedef struct mal_detail_repr_product_10 mal_repr_product_10_t;
typedef struct mal_detail_repr_sum_11 mal_repr_sum_11_t;
typedef mal_repr_sum_0_t mal_OptionalAddress_t;
typedef mal_repr_product_1_t mal_MapRequest_t;
typedef mal_repr_product_3_t mal_ResizeRequest_t;
typedef mal_repr_sum_2_t mal_AddressResult_t;
typedef mal_repr_sum_7_t mal_FileDescriptorResult_t;
typedef mal_repr_sum_9_t mal_OffsetResult_t;
typedef mal_repr_sum_11_t mal_TransferResult_t;
typedef mal_repr_sum_5_t mal_SyscallStatus_t;

struct mal_detail_repr_sum_0 {
    uint32_t tag;
    union {
        mal_Unit_t variant_0;
        mal_Address_t variant_1;
    } payload;
};

struct mal_detail_repr_product_1 {
    mal_repr_sum_0_t field_0;
    mal_ByteSize_t field_1;
    mal_Int32_t field_2;
    mal_Int32_t field_3;
    mal_Int32_t field_4;
    mal_UInt64_t field_5;
};

struct mal_detail_repr_sum_2 {
    uint32_t tag;
    union {
        mal_Address_t variant_0;
        mal_UInt32_t variant_1;
    } payload;
};

struct mal_detail_repr_product_3 {
    mal_Address_t field_0;
    mal_ByteSize_t field_1;
    mal_ByteSize_t field_2;
    mal_Int32_t field_3;
};

struct mal_detail_repr_product_4 {
    mal_Address_t field_0;
    mal_ByteSize_t field_1;
};

struct mal_detail_repr_sum_5 {
    uint32_t tag;
    union {
        mal_Unit_t variant_0;
        mal_UInt32_t variant_1;
    } payload;
};

struct mal_detail_repr_product_6 {
    mal_Int32_t field_0;
    mal_Address_t field_1;
    mal_Int32_t field_2;
    mal_UInt32_t field_3;
};

struct mal_detail_repr_sum_7 {
    uint32_t tag;
    union {
        mal_Int32_t variant_0;
        mal_UInt32_t variant_1;
    } payload;
};

struct mal_detail_repr_product_8 {
    mal_Int32_t field_0;
    mal_Int64_t field_1;
    mal_Int32_t field_2;
};

struct mal_detail_repr_sum_9 {
    uint32_t tag;
    union {
        mal_UInt64_t variant_0;
        mal_UInt32_t variant_1;
    } payload;
};

struct mal_detail_repr_product_10 {
    mal_Int32_t field_0;
    mal_Address_t field_1;
    mal_ByteSize_t field_2;
    mal_ByteSize_t field_3;
};

struct mal_detail_repr_sum_11 {
    uint32_t tag;
    union {
        mal_ByteSize_t variant_0;
        mal_UInt32_t variant_1;
    } payload;
};

/* Type helpers */

static inline mal_repr_sum_0_t mal_detail_to_host_0(mal_call_t *call, MalRepr_Sum_0 value) {
    switch (value.tag) {
        case UINT32_C(0): {
            return (mal_repr_sum_0_t){ .tag = UINT32_C(0), .payload.variant_0 = value.payload.variant_0 };
        }
        case UINT32_C(1): {
            return (mal_repr_sum_0_t){ .tag = UINT32_C(1), .payload.variant_1 = value.payload.variant_1 };
        }
        default: {
            mal_call_trap(call, "invalid sum tag");
        }
    }
}

static inline MalRepr_Sum_0 mal_detail_to_raw_0(mal_call_t *call, mal_repr_sum_0_t value) {
    switch (value.tag) {
        case UINT32_C(0): {
            return (MalRepr_Sum_0){ .tag = UINT32_C(0), .payload.variant_0 = (MalType_Unit){ 0 } };
        }
        case UINT32_C(1): {
            return (MalRepr_Sum_0){ .tag = UINT32_C(1), .payload.variant_1 = mal_Address_return(call, value.payload.variant_1) };
        }
        default: {
            mal_call_trap(call, "invalid sum tag");
        }
    }
}

#define mal_repr_sum_0_tag_0 UINT32_C(0)
static inline mal_repr_sum_0_t mal_repr_sum_0_make_0(void) {
    return (mal_repr_sum_0_t){ .tag = mal_repr_sum_0_tag_0, .payload.variant_0 = (mal_Unit_t){ 0 } };
}

static inline MalRepr_Sum_0 mal_repr_sum_0_return_0(mal_call_t *call) {
    return mal_detail_to_raw_0(call, (mal_repr_sum_0_t){ .tag = mal_repr_sum_0_tag_0, .payload.variant_0 = (mal_Unit_t){ 0 } });
}

#define mal_repr_sum_0_tag_1 UINT32_C(1)
static inline mal_repr_sum_0_t mal_repr_sum_0_make_1(mal_Address_t value) {
    return (mal_repr_sum_0_t){ .tag = mal_repr_sum_0_tag_1, .payload.variant_1 = value };
}

static inline MalRepr_Sum_0 mal_repr_sum_0_return_1(mal_call_t *call, mal_Address_t value) {
    return mal_detail_to_raw_0(call, (mal_repr_sum_0_t){ .tag = mal_repr_sum_0_tag_1, .payload.variant_1 = value });
}

static inline MalRepr_Product_1 mal_repr_product_1_return(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, mal_repr_product_1_t value) {
    return (MalRepr_Product_1){ .field_0 = mal_detail_to_raw_0(call, value.field_0), .field_1 = value.field_1, .field_2 = value.field_2, .field_3 = value.field_3, .field_4 = value.field_4, .field_5 = value.field_5 };
}

static inline mal_repr_sum_2_t mal_detail_to_host_2(mal_call_t *call, MalRepr_Sum_2 value) {
    switch (value.tag) {
        case UINT32_C(0): {
            return (mal_repr_sum_2_t){ .tag = UINT32_C(0), .payload.variant_0 = value.payload.variant_0 };
        }
        case UINT32_C(1): {
            return (mal_repr_sum_2_t){ .tag = UINT32_C(1), .payload.variant_1 = value.payload.variant_1 };
        }
        default: {
            mal_call_trap(call, "invalid sum tag");
        }
    }
}

static inline MalRepr_Sum_2 mal_detail_to_raw_2(mal_call_t *call, mal_repr_sum_2_t value) {
    switch (value.tag) {
        case UINT32_C(0): {
            return (MalRepr_Sum_2){ .tag = UINT32_C(0), .payload.variant_0 = mal_Address_return(call, value.payload.variant_0) };
        }
        case UINT32_C(1): {
            return (MalRepr_Sum_2){ .tag = UINT32_C(1), .payload.variant_1 = value.payload.variant_1 };
        }
        default: {
            mal_call_trap(call, "invalid sum tag");
        }
    }
}

#define mal_repr_sum_2_tag_0 UINT32_C(0)
static inline mal_repr_sum_2_t mal_repr_sum_2_make_0(mal_Address_t value) {
    return (mal_repr_sum_2_t){ .tag = mal_repr_sum_2_tag_0, .payload.variant_0 = value };
}

static inline MalRepr_Sum_2 mal_repr_sum_2_return_0(mal_call_t *call, mal_Address_t value) {
    return mal_detail_to_raw_2(call, (mal_repr_sum_2_t){ .tag = mal_repr_sum_2_tag_0, .payload.variant_0 = value });
}

#define mal_repr_sum_2_tag_1 UINT32_C(1)
static inline mal_repr_sum_2_t mal_repr_sum_2_make_1(mal_UInt32_t value) {
    return (mal_repr_sum_2_t){ .tag = mal_repr_sum_2_tag_1, .payload.variant_1 = value };
}

static inline MalRepr_Sum_2 mal_repr_sum_2_return_1(mal_call_t *call, mal_UInt32_t value) {
    return mal_detail_to_raw_2(call, (mal_repr_sum_2_t){ .tag = mal_repr_sum_2_tag_1, .payload.variant_1 = value });
}

static inline MalRepr_Product_3 mal_repr_product_3_return(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, mal_repr_product_3_t value) {
    return (MalRepr_Product_3){ .field_0 = mal_Address_return(call, value.field_0), .field_1 = value.field_1, .field_2 = value.field_2, .field_3 = value.field_3 };
}

static inline MalRepr_Product_4 mal_repr_product_4_return(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, mal_repr_product_4_t value) {
    return (MalRepr_Product_4){ .field_0 = mal_Address_return(call, value.field_0), .field_1 = value.field_1 };
}

static inline mal_repr_sum_5_t mal_detail_to_host_5(mal_call_t *call, MalRepr_Sum_5 value) {
    switch (value.tag) {
        case UINT32_C(0): {
            return (mal_repr_sum_5_t){ .tag = UINT32_C(0), .payload.variant_0 = value.payload.variant_0 };
        }
        case UINT32_C(1): {
            return (mal_repr_sum_5_t){ .tag = UINT32_C(1), .payload.variant_1 = value.payload.variant_1 };
        }
        default: {
            mal_call_trap(call, "invalid sum tag");
        }
    }
}

static inline MalRepr_Sum_5 mal_detail_to_raw_5(mal_call_t *call, mal_repr_sum_5_t value) {
    switch (value.tag) {
        case UINT32_C(0): {
            return (MalRepr_Sum_5){ .tag = UINT32_C(0), .payload.variant_0 = (MalType_Unit){ 0 } };
        }
        case UINT32_C(1): {
            return (MalRepr_Sum_5){ .tag = UINT32_C(1), .payload.variant_1 = value.payload.variant_1 };
        }
        default: {
            mal_call_trap(call, "invalid sum tag");
        }
    }
}

#define mal_repr_sum_5_tag_0 UINT32_C(0)
static inline mal_repr_sum_5_t mal_repr_sum_5_make_0(void) {
    return (mal_repr_sum_5_t){ .tag = mal_repr_sum_5_tag_0, .payload.variant_0 = (mal_Unit_t){ 0 } };
}

static inline MalRepr_Sum_5 mal_repr_sum_5_return_0(mal_call_t *call) {
    return mal_detail_to_raw_5(call, (mal_repr_sum_5_t){ .tag = mal_repr_sum_5_tag_0, .payload.variant_0 = (mal_Unit_t){ 0 } });
}

#define mal_repr_sum_5_tag_1 UINT32_C(1)
static inline mal_repr_sum_5_t mal_repr_sum_5_make_1(mal_UInt32_t value) {
    return (mal_repr_sum_5_t){ .tag = mal_repr_sum_5_tag_1, .payload.variant_1 = value };
}

static inline MalRepr_Sum_5 mal_repr_sum_5_return_1(mal_call_t *call, mal_UInt32_t value) {
    return mal_detail_to_raw_5(call, (mal_repr_sum_5_t){ .tag = mal_repr_sum_5_tag_1, .payload.variant_1 = value });
}

static inline MalRepr_Product_6 mal_repr_product_6_return(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, mal_repr_product_6_t value) {
    return (MalRepr_Product_6){ .field_0 = value.field_0, .field_1 = mal_Address_return(call, value.field_1), .field_2 = value.field_2, .field_3 = value.field_3 };
}

static inline mal_repr_sum_7_t mal_detail_to_host_7(mal_call_t *call, MalRepr_Sum_7 value) {
    switch (value.tag) {
        case UINT32_C(0): {
            return (mal_repr_sum_7_t){ .tag = UINT32_C(0), .payload.variant_0 = value.payload.variant_0 };
        }
        case UINT32_C(1): {
            return (mal_repr_sum_7_t){ .tag = UINT32_C(1), .payload.variant_1 = value.payload.variant_1 };
        }
        default: {
            mal_call_trap(call, "invalid sum tag");
        }
    }
}

static inline MalRepr_Sum_7 mal_detail_to_raw_7(mal_call_t *call, mal_repr_sum_7_t value) {
    switch (value.tag) {
        case UINT32_C(0): {
            return (MalRepr_Sum_7){ .tag = UINT32_C(0), .payload.variant_0 = value.payload.variant_0 };
        }
        case UINT32_C(1): {
            return (MalRepr_Sum_7){ .tag = UINT32_C(1), .payload.variant_1 = value.payload.variant_1 };
        }
        default: {
            mal_call_trap(call, "invalid sum tag");
        }
    }
}

#define mal_repr_sum_7_tag_0 UINT32_C(0)
static inline mal_repr_sum_7_t mal_repr_sum_7_make_0(mal_Int32_t value) {
    return (mal_repr_sum_7_t){ .tag = mal_repr_sum_7_tag_0, .payload.variant_0 = value };
}

static inline MalRepr_Sum_7 mal_repr_sum_7_return_0(mal_call_t *call, mal_Int32_t value) {
    return mal_detail_to_raw_7(call, (mal_repr_sum_7_t){ .tag = mal_repr_sum_7_tag_0, .payload.variant_0 = value });
}

#define mal_repr_sum_7_tag_1 UINT32_C(1)
static inline mal_repr_sum_7_t mal_repr_sum_7_make_1(mal_UInt32_t value) {
    return (mal_repr_sum_7_t){ .tag = mal_repr_sum_7_tag_1, .payload.variant_1 = value };
}

static inline MalRepr_Sum_7 mal_repr_sum_7_return_1(mal_call_t *call, mal_UInt32_t value) {
    return mal_detail_to_raw_7(call, (mal_repr_sum_7_t){ .tag = mal_repr_sum_7_tag_1, .payload.variant_1 = value });
}

static inline MalRepr_Product_8 mal_repr_product_8_return(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, mal_repr_product_8_t value) {
    return (MalRepr_Product_8){ .field_0 = value.field_0, .field_1 = value.field_1, .field_2 = value.field_2 };
}

static inline mal_repr_sum_9_t mal_detail_to_host_9(mal_call_t *call, MalRepr_Sum_9 value) {
    switch (value.tag) {
        case UINT32_C(0): {
            return (mal_repr_sum_9_t){ .tag = UINT32_C(0), .payload.variant_0 = value.payload.variant_0 };
        }
        case UINT32_C(1): {
            return (mal_repr_sum_9_t){ .tag = UINT32_C(1), .payload.variant_1 = value.payload.variant_1 };
        }
        default: {
            mal_call_trap(call, "invalid sum tag");
        }
    }
}

static inline MalRepr_Sum_9 mal_detail_to_raw_9(mal_call_t *call, mal_repr_sum_9_t value) {
    switch (value.tag) {
        case UINT32_C(0): {
            return (MalRepr_Sum_9){ .tag = UINT32_C(0), .payload.variant_0 = value.payload.variant_0 };
        }
        case UINT32_C(1): {
            return (MalRepr_Sum_9){ .tag = UINT32_C(1), .payload.variant_1 = value.payload.variant_1 };
        }
        default: {
            mal_call_trap(call, "invalid sum tag");
        }
    }
}

#define mal_repr_sum_9_tag_0 UINT32_C(0)
static inline mal_repr_sum_9_t mal_repr_sum_9_make_0(mal_UInt64_t value) {
    return (mal_repr_sum_9_t){ .tag = mal_repr_sum_9_tag_0, .payload.variant_0 = value };
}

static inline MalRepr_Sum_9 mal_repr_sum_9_return_0(mal_call_t *call, mal_UInt64_t value) {
    return mal_detail_to_raw_9(call, (mal_repr_sum_9_t){ .tag = mal_repr_sum_9_tag_0, .payload.variant_0 = value });
}

#define mal_repr_sum_9_tag_1 UINT32_C(1)
static inline mal_repr_sum_9_t mal_repr_sum_9_make_1(mal_UInt32_t value) {
    return (mal_repr_sum_9_t){ .tag = mal_repr_sum_9_tag_1, .payload.variant_1 = value };
}

static inline MalRepr_Sum_9 mal_repr_sum_9_return_1(mal_call_t *call, mal_UInt32_t value) {
    return mal_detail_to_raw_9(call, (mal_repr_sum_9_t){ .tag = mal_repr_sum_9_tag_1, .payload.variant_1 = value });
}

static inline MalRepr_Product_10 mal_repr_product_10_return(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, mal_repr_product_10_t value) {
    return (MalRepr_Product_10){ .field_0 = value.field_0, .field_1 = mal_Address_return(call, value.field_1), .field_2 = value.field_2, .field_3 = value.field_3 };
}

static inline mal_repr_sum_11_t mal_detail_to_host_11(mal_call_t *call, MalRepr_Sum_11 value) {
    switch (value.tag) {
        case UINT32_C(0): {
            return (mal_repr_sum_11_t){ .tag = UINT32_C(0), .payload.variant_0 = value.payload.variant_0 };
        }
        case UINT32_C(1): {
            return (mal_repr_sum_11_t){ .tag = UINT32_C(1), .payload.variant_1 = value.payload.variant_1 };
        }
        default: {
            mal_call_trap(call, "invalid sum tag");
        }
    }
}

static inline MalRepr_Sum_11 mal_detail_to_raw_11(mal_call_t *call, mal_repr_sum_11_t value) {
    switch (value.tag) {
        case UINT32_C(0): {
            return (MalRepr_Sum_11){ .tag = UINT32_C(0), .payload.variant_0 = value.payload.variant_0 };
        }
        case UINT32_C(1): {
            return (MalRepr_Sum_11){ .tag = UINT32_C(1), .payload.variant_1 = value.payload.variant_1 };
        }
        default: {
            mal_call_trap(call, "invalid sum tag");
        }
    }
}

#define mal_repr_sum_11_tag_0 UINT32_C(0)
static inline mal_repr_sum_11_t mal_repr_sum_11_make_0(mal_ByteSize_t value) {
    return (mal_repr_sum_11_t){ .tag = mal_repr_sum_11_tag_0, .payload.variant_0 = value };
}

static inline MalRepr_Sum_11 mal_repr_sum_11_return_0(mal_call_t *call, mal_ByteSize_t value) {
    return mal_detail_to_raw_11(call, (mal_repr_sum_11_t){ .tag = mal_repr_sum_11_tag_0, .payload.variant_0 = value });
}

#define mal_repr_sum_11_tag_1 UINT32_C(1)
static inline mal_repr_sum_11_t mal_repr_sum_11_make_1(mal_UInt32_t value) {
    return (mal_repr_sum_11_t){ .tag = mal_repr_sum_11_tag_1, .payload.variant_1 = value };
}

static inline MalRepr_Sum_11 mal_repr_sum_11_return_1(mal_call_t *call, mal_UInt32_t value) {
    return mal_detail_to_raw_11(call, (mal_repr_sum_11_t){ .tag = mal_repr_sum_11_tag_1, .payload.variant_1 = value });
}

#define mal_OptionalAddress_tag_0 UINT32_C(0)
static inline mal_OptionalAddress_t mal_OptionalAddress_make_0(void) {
    return (mal_OptionalAddress_t){ .tag = mal_OptionalAddress_tag_0, .payload.variant_0 = (mal_Unit_t){ 0 } };
}

static inline MalType_OptionalAddress mal_OptionalAddress_return_0(mal_call_t *call) {
    return mal_detail_to_raw_0(call, (mal_OptionalAddress_t){ .tag = mal_OptionalAddress_tag_0, .payload.variant_0 = (mal_Unit_t){ 0 } });
}

#define mal_OptionalAddress_tag_1 UINT32_C(1)
static inline mal_OptionalAddress_t mal_OptionalAddress_make_1(mal_Address_t value) {
    return (mal_OptionalAddress_t){ .tag = mal_OptionalAddress_tag_1, .payload.variant_1 = value };
}

static inline MalType_OptionalAddress mal_OptionalAddress_return_1(mal_call_t *call, mal_Address_t value) {
    return mal_detail_to_raw_0(call, (mal_OptionalAddress_t){ .tag = mal_OptionalAddress_tag_1, .payload.variant_1 = value });
}

static inline MalType_MapRequest mal_MapRequest_return(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, mal_MapRequest_t value) {
    return (MalRepr_Product_1){ .field_0 = mal_detail_to_raw_0(call, value.field_0), .field_1 = value.field_1, .field_2 = value.field_2, .field_3 = value.field_3, .field_4 = value.field_4, .field_5 = value.field_5 };
}

static inline MalType_ResizeRequest mal_ResizeRequest_return(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, mal_ResizeRequest_t value) {
    return (MalRepr_Product_3){ .field_0 = mal_Address_return(call, value.field_0), .field_1 = value.field_1, .field_2 = value.field_2, .field_3 = value.field_3 };
}

#define mal_AddressResult_tag_0 UINT32_C(0)
static inline mal_AddressResult_t mal_AddressResult_make_0(mal_Address_t value) {
    return (mal_AddressResult_t){ .tag = mal_AddressResult_tag_0, .payload.variant_0 = value };
}

static inline MalType_AddressResult mal_AddressResult_return_0(mal_call_t *call, mal_Address_t value) {
    return mal_detail_to_raw_2(call, (mal_AddressResult_t){ .tag = mal_AddressResult_tag_0, .payload.variant_0 = value });
}

#define mal_AddressResult_tag_1 UINT32_C(1)
static inline mal_AddressResult_t mal_AddressResult_make_1(mal_UInt32_t value) {
    return (mal_AddressResult_t){ .tag = mal_AddressResult_tag_1, .payload.variant_1 = value };
}

static inline MalType_AddressResult mal_AddressResult_return_1(mal_call_t *call, mal_UInt32_t value) {
    return mal_detail_to_raw_2(call, (mal_AddressResult_t){ .tag = mal_AddressResult_tag_1, .payload.variant_1 = value });
}

#define mal_FileDescriptorResult_tag_0 UINT32_C(0)
static inline mal_FileDescriptorResult_t mal_FileDescriptorResult_make_0(mal_Int32_t value) {
    return (mal_FileDescriptorResult_t){ .tag = mal_FileDescriptorResult_tag_0, .payload.variant_0 = value };
}

static inline MalType_FileDescriptorResult mal_FileDescriptorResult_return_0(mal_call_t *call, mal_Int32_t value) {
    return mal_detail_to_raw_7(call, (mal_FileDescriptorResult_t){ .tag = mal_FileDescriptorResult_tag_0, .payload.variant_0 = value });
}

#define mal_FileDescriptorResult_tag_1 UINT32_C(1)
static inline mal_FileDescriptorResult_t mal_FileDescriptorResult_make_1(mal_UInt32_t value) {
    return (mal_FileDescriptorResult_t){ .tag = mal_FileDescriptorResult_tag_1, .payload.variant_1 = value };
}

static inline MalType_FileDescriptorResult mal_FileDescriptorResult_return_1(mal_call_t *call, mal_UInt32_t value) {
    return mal_detail_to_raw_7(call, (mal_FileDescriptorResult_t){ .tag = mal_FileDescriptorResult_tag_1, .payload.variant_1 = value });
}

#define mal_OffsetResult_tag_0 UINT32_C(0)
static inline mal_OffsetResult_t mal_OffsetResult_make_0(mal_UInt64_t value) {
    return (mal_OffsetResult_t){ .tag = mal_OffsetResult_tag_0, .payload.variant_0 = value };
}

static inline MalType_OffsetResult mal_OffsetResult_return_0(mal_call_t *call, mal_UInt64_t value) {
    return mal_detail_to_raw_9(call, (mal_OffsetResult_t){ .tag = mal_OffsetResult_tag_0, .payload.variant_0 = value });
}

#define mal_OffsetResult_tag_1 UINT32_C(1)
static inline mal_OffsetResult_t mal_OffsetResult_make_1(mal_UInt32_t value) {
    return (mal_OffsetResult_t){ .tag = mal_OffsetResult_tag_1, .payload.variant_1 = value };
}

static inline MalType_OffsetResult mal_OffsetResult_return_1(mal_call_t *call, mal_UInt32_t value) {
    return mal_detail_to_raw_9(call, (mal_OffsetResult_t){ .tag = mal_OffsetResult_tag_1, .payload.variant_1 = value });
}

#define mal_TransferResult_tag_0 UINT32_C(0)
static inline mal_TransferResult_t mal_TransferResult_make_0(mal_ByteSize_t value) {
    return (mal_TransferResult_t){ .tag = mal_TransferResult_tag_0, .payload.variant_0 = value };
}

static inline MalType_TransferResult mal_TransferResult_return_0(mal_call_t *call, mal_ByteSize_t value) {
    return mal_detail_to_raw_11(call, (mal_TransferResult_t){ .tag = mal_TransferResult_tag_0, .payload.variant_0 = value });
}

#define mal_TransferResult_tag_1 UINT32_C(1)
static inline mal_TransferResult_t mal_TransferResult_make_1(mal_UInt32_t value) {
    return (mal_TransferResult_t){ .tag = mal_TransferResult_tag_1, .payload.variant_1 = value };
}

static inline MalType_TransferResult mal_TransferResult_return_1(mal_call_t *call, mal_UInt32_t value) {
    return mal_detail_to_raw_11(call, (mal_TransferResult_t){ .tag = mal_TransferResult_tag_1, .payload.variant_1 = value });
}

#define mal_SyscallStatus_tag_0 UINT32_C(0)
static inline mal_SyscallStatus_t mal_SyscallStatus_make_0(void) {
    return (mal_SyscallStatus_t){ .tag = mal_SyscallStatus_tag_0, .payload.variant_0 = (mal_Unit_t){ 0 } };
}

static inline MalType_SyscallStatus mal_SyscallStatus_return_0(mal_call_t *call) {
    return mal_detail_to_raw_5(call, (mal_SyscallStatus_t){ .tag = mal_SyscallStatus_tag_0, .payload.variant_0 = (mal_Unit_t){ 0 } });
}

#define mal_SyscallStatus_tag_1 UINT32_C(1)
static inline mal_SyscallStatus_t mal_SyscallStatus_make_1(mal_UInt32_t value) {
    return (mal_SyscallStatus_t){ .tag = mal_SyscallStatus_tag_1, .payload.variant_1 = value };
}

static inline MalType_SyscallStatus mal_SyscallStatus_return_1(mal_call_t *call, mal_UInt32_t value) {
    return mal_detail_to_raw_5(call, (mal_SyscallStatus_t){ .tag = mal_SyscallStatus_tag_1, .payload.variant_1 = value });
}

/* External operations */

MalType_AddressResult mal_ext_systemMmap(MalContext *context, MalType_OptionalAddress argument_0, MalType_ByteSize argument_1, MalType_Int32 argument_2, MalType_Int32 argument_3, MalType_Int32 argument_4, MalType_UInt64 argument_5);
MalType_AddressResult mal_ext_systemMremap(MalContext *context, MalType_Address argument_0, MalType_ByteSize argument_1, MalType_ByteSize argument_2, MalType_Int32 argument_3);
MalType_SyscallStatus mal_ext_systemMunmap(MalContext *context, MalType_Address argument_0, MalType_ByteSize argument_1);
MalType_FileDescriptorResult mal_ext_systemOpenat(MalContext *context, MalType_Int32 argument_0, MalType_Address argument_1, MalType_Int32 argument_2, MalType_UInt32 argument_3);
MalType_OffsetResult mal_ext_systemLseek(MalContext *context, MalType_Int32 argument_0, MalType_Int64 argument_1, MalType_Int32 argument_2);
MalType_SyscallStatus mal_ext_systemClose(MalContext *context, MalType_Int32 value);
MalType_TransferResult mal_ext_systemRead(MalContext *context, MalType_Int32 argument_0, MalType_Address argument_1, MalType_ByteSize argument_2, MalType_ByteSize argument_3);
MalType_TransferResult mal_ext_systemWrite(MalContext *context, MalType_Int32 argument_0, MalType_Address argument_1, MalType_ByteSize argument_2, MalType_ByteSize argument_3);
void mal_ext_storeZero(MalContext *context, MalType_Address argument_0, MalType_ByteSize argument_1);

/* External definition helpers */

#define MAL_HAS_EXTERN_systemMmap 1
#define MAL_DEFINE_systemMmap(call, value) \
static MalType_AddressResult mal_detail_systemMmap(mal_call_t *call, mal_MapRequest_t value); \
MalType_AddressResult mal_ext_systemMmap(MalContext *context MAL_DETAIL_MAYBE_UNUSED, MalType_OptionalAddress argument_0, MalType_ByteSize argument_1, MalType_Int32 argument_2, MalType_Int32 argument_3, MalType_Int32 argument_4, MalType_UInt64 argument_5) { \
    mal_call_t call = (mal_call_t){ .mal_detail_context = context }; \
    return mal_detail_systemMmap(&call, (mal_MapRequest_t){ .field_0 = mal_detail_to_host_0(&call, ((MalRepr_Product_1){ .field_0 = argument_0, .field_1 = argument_1, .field_2 = argument_2, .field_3 = argument_3, .field_4 = argument_4, .field_5 = argument_5 }).field_0), .field_1 = ((MalRepr_Product_1){ .field_0 = argument_0, .field_1 = argument_1, .field_2 = argument_2, .field_3 = argument_3, .field_4 = argument_4, .field_5 = argument_5 }).field_1, .field_2 = ((MalRepr_Product_1){ .field_0 = argument_0, .field_1 = argument_1, .field_2 = argument_2, .field_3 = argument_3, .field_4 = argument_4, .field_5 = argument_5 }).field_2, .field_3 = ((MalRepr_Product_1){ .field_0 = argument_0, .field_1 = argument_1, .field_2 = argument_2, .field_3 = argument_3, .field_4 = argument_4, .field_5 = argument_5 }).field_3, .field_4 = ((MalRepr_Product_1){ .field_0 = argument_0, .field_1 = argument_1, .field_2 = argument_2, .field_3 = argument_3, .field_4 = argument_4, .field_5 = argument_5 }).field_4, .field_5 = ((MalRepr_Product_1){ .field_0 = argument_0, .field_1 = argument_1, .field_2 = argument_2, .field_3 = argument_3, .field_4 = argument_4, .field_5 = argument_5 }).field_5 }); \
} \
static MalType_AddressResult mal_detail_systemMmap( \
    mal_call_t *call, \
    mal_MapRequest_t value \
)

#define MAL_HAS_EXTERN_systemMremap 1
#define MAL_DEFINE_systemMremap(call, value) \
static MalType_AddressResult mal_detail_systemMremap(mal_call_t *call, mal_ResizeRequest_t value); \
MalType_AddressResult mal_ext_systemMremap(MalContext *context MAL_DETAIL_MAYBE_UNUSED, MalType_Address argument_0, MalType_ByteSize argument_1, MalType_ByteSize argument_2, MalType_Int32 argument_3) { \
    mal_call_t call = (mal_call_t){ .mal_detail_context = context }; \
    return mal_detail_systemMremap(&call, (mal_ResizeRequest_t){ .field_0 = ((MalRepr_Product_3){ .field_0 = argument_0, .field_1 = argument_1, .field_2 = argument_2, .field_3 = argument_3 }).field_0, .field_1 = ((MalRepr_Product_3){ .field_0 = argument_0, .field_1 = argument_1, .field_2 = argument_2, .field_3 = argument_3 }).field_1, .field_2 = ((MalRepr_Product_3){ .field_0 = argument_0, .field_1 = argument_1, .field_2 = argument_2, .field_3 = argument_3 }).field_2, .field_3 = ((MalRepr_Product_3){ .field_0 = argument_0, .field_1 = argument_1, .field_2 = argument_2, .field_3 = argument_3 }).field_3 }); \
} \
static MalType_AddressResult mal_detail_systemMremap( \
    mal_call_t *call, \
    mal_ResizeRequest_t value \
)

#define MAL_HAS_EXTERN_systemMunmap 1
#define MAL_DEFINE_systemMunmap(call, value) \
static MalType_SyscallStatus mal_detail_systemMunmap(mal_call_t *call, mal_repr_product_4_t value); \
MalType_SyscallStatus mal_ext_systemMunmap(MalContext *context MAL_DETAIL_MAYBE_UNUSED, MalType_Address argument_0, MalType_ByteSize argument_1) { \
    mal_call_t call = (mal_call_t){ .mal_detail_context = context }; \
    return mal_detail_systemMunmap(&call, (mal_repr_product_4_t){ .field_0 = ((MalRepr_Product_4){ .field_0 = argument_0, .field_1 = argument_1 }).field_0, .field_1 = ((MalRepr_Product_4){ .field_0 = argument_0, .field_1 = argument_1 }).field_1 }); \
} \
static MalType_SyscallStatus mal_detail_systemMunmap( \
    mal_call_t *call, \
    mal_repr_product_4_t value \
)

#define MAL_HAS_EXTERN_systemOpenat 1
#define MAL_DEFINE_systemOpenat(call, value) \
static MalType_FileDescriptorResult mal_detail_systemOpenat(mal_call_t *call, mal_repr_product_6_t value); \
MalType_FileDescriptorResult mal_ext_systemOpenat(MalContext *context MAL_DETAIL_MAYBE_UNUSED, MalType_Int32 argument_0, MalType_Address argument_1, MalType_Int32 argument_2, MalType_UInt32 argument_3) { \
    mal_call_t call = (mal_call_t){ .mal_detail_context = context }; \
    return mal_detail_systemOpenat(&call, (mal_repr_product_6_t){ .field_0 = ((MalRepr_Product_6){ .field_0 = argument_0, .field_1 = argument_1, .field_2 = argument_2, .field_3 = argument_3 }).field_0, .field_1 = ((MalRepr_Product_6){ .field_0 = argument_0, .field_1 = argument_1, .field_2 = argument_2, .field_3 = argument_3 }).field_1, .field_2 = ((MalRepr_Product_6){ .field_0 = argument_0, .field_1 = argument_1, .field_2 = argument_2, .field_3 = argument_3 }).field_2, .field_3 = ((MalRepr_Product_6){ .field_0 = argument_0, .field_1 = argument_1, .field_2 = argument_2, .field_3 = argument_3 }).field_3 }); \
} \
static MalType_FileDescriptorResult mal_detail_systemOpenat( \
    mal_call_t *call, \
    mal_repr_product_6_t value \
)

#define MAL_HAS_EXTERN_systemLseek 1
#define MAL_DEFINE_systemLseek(call, value) \
static MalType_OffsetResult mal_detail_systemLseek(mal_call_t *call, mal_repr_product_8_t value); \
MalType_OffsetResult mal_ext_systemLseek(MalContext *context MAL_DETAIL_MAYBE_UNUSED, MalType_Int32 argument_0, MalType_Int64 argument_1, MalType_Int32 argument_2) { \
    mal_call_t call = (mal_call_t){ .mal_detail_context = context }; \
    return mal_detail_systemLseek(&call, (mal_repr_product_8_t){ .field_0 = ((MalRepr_Product_8){ .field_0 = argument_0, .field_1 = argument_1, .field_2 = argument_2 }).field_0, .field_1 = ((MalRepr_Product_8){ .field_0 = argument_0, .field_1 = argument_1, .field_2 = argument_2 }).field_1, .field_2 = ((MalRepr_Product_8){ .field_0 = argument_0, .field_1 = argument_1, .field_2 = argument_2 }).field_2 }); \
} \
static MalType_OffsetResult mal_detail_systemLseek( \
    mal_call_t *call, \
    mal_repr_product_8_t value \
)

#define MAL_HAS_EXTERN_systemClose 1
#define MAL_DEFINE_systemClose(call, value) \
static MalType_SyscallStatus mal_detail_systemClose(mal_call_t *call, mal_Int32_t value); \
MalType_SyscallStatus mal_ext_systemClose(MalContext *context MAL_DETAIL_MAYBE_UNUSED, MalType_Int32 value) { \
    mal_call_t call = (mal_call_t){ .mal_detail_context = context }; \
    return mal_detail_systemClose(&call, value); \
} \
static MalType_SyscallStatus mal_detail_systemClose( \
    mal_call_t *call, \
    mal_Int32_t value \
)

#define MAL_HAS_EXTERN_systemRead 1
#define MAL_DEFINE_systemRead(call, value) \
static MalType_TransferResult mal_detail_systemRead(mal_call_t *call, mal_repr_product_10_t value); \
MalType_TransferResult mal_ext_systemRead(MalContext *context MAL_DETAIL_MAYBE_UNUSED, MalType_Int32 argument_0, MalType_Address argument_1, MalType_ByteSize argument_2, MalType_ByteSize argument_3) { \
    mal_call_t call = (mal_call_t){ .mal_detail_context = context }; \
    return mal_detail_systemRead(&call, (mal_repr_product_10_t){ .field_0 = ((MalRepr_Product_10){ .field_0 = argument_0, .field_1 = argument_1, .field_2 = argument_2, .field_3 = argument_3 }).field_0, .field_1 = ((MalRepr_Product_10){ .field_0 = argument_0, .field_1 = argument_1, .field_2 = argument_2, .field_3 = argument_3 }).field_1, .field_2 = ((MalRepr_Product_10){ .field_0 = argument_0, .field_1 = argument_1, .field_2 = argument_2, .field_3 = argument_3 }).field_2, .field_3 = ((MalRepr_Product_10){ .field_0 = argument_0, .field_1 = argument_1, .field_2 = argument_2, .field_3 = argument_3 }).field_3 }); \
} \
static MalType_TransferResult mal_detail_systemRead( \
    mal_call_t *call, \
    mal_repr_product_10_t value \
)

#define MAL_HAS_EXTERN_systemWrite 1
#define MAL_DEFINE_systemWrite(call, value) \
static MalType_TransferResult mal_detail_systemWrite(mal_call_t *call, mal_repr_product_10_t value); \
MalType_TransferResult mal_ext_systemWrite(MalContext *context MAL_DETAIL_MAYBE_UNUSED, MalType_Int32 argument_0, MalType_Address argument_1, MalType_ByteSize argument_2, MalType_ByteSize argument_3) { \
    mal_call_t call = (mal_call_t){ .mal_detail_context = context }; \
    return mal_detail_systemWrite(&call, (mal_repr_product_10_t){ .field_0 = ((MalRepr_Product_10){ .field_0 = argument_0, .field_1 = argument_1, .field_2 = argument_2, .field_3 = argument_3 }).field_0, .field_1 = ((MalRepr_Product_10){ .field_0 = argument_0, .field_1 = argument_1, .field_2 = argument_2, .field_3 = argument_3 }).field_1, .field_2 = ((MalRepr_Product_10){ .field_0 = argument_0, .field_1 = argument_1, .field_2 = argument_2, .field_3 = argument_3 }).field_2, .field_3 = ((MalRepr_Product_10){ .field_0 = argument_0, .field_1 = argument_1, .field_2 = argument_2, .field_3 = argument_3 }).field_3 }); \
} \
static MalType_TransferResult mal_detail_systemWrite( \
    mal_call_t *call, \
    mal_repr_product_10_t value \
)

#define MAL_HAS_EXTERN_storeZero 1
#define MAL_DEFINE_storeZero(call, value) \
static MalType_Unit mal_detail_storeZero(mal_call_t *call, mal_repr_product_4_t value); \
void mal_ext_storeZero(MalContext *context MAL_DETAIL_MAYBE_UNUSED, MalType_Address argument_0, MalType_ByteSize argument_1) { \
    mal_call_t call = (mal_call_t){ .mal_detail_context = context }; \
    mal_detail_storeZero(&call, (mal_repr_product_4_t){ .field_0 = ((MalRepr_Product_4){ .field_0 = argument_0, .field_1 = argument_1 }).field_0, .field_1 = ((MalRepr_Product_4){ .field_0 = argument_0, .field_1 = argument_1 }).field_1 }); \
} \
static MalType_Unit mal_detail_storeZero( \
    mal_call_t *call, \
    mal_repr_product_4_t value \
)

#endif
