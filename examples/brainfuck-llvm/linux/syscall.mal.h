#ifndef MAL_BUILD_UMBRELLA
#include <mal.h>
#include <string.h>

_Static_assert(MAL_C_ABI_VERSION == 0x000900u, "generated header requires mal C ABI 0x000900");
_Static_assert((sizeof((size_t)0) * CHAR_BIT) == 64, "size_t does not match the mal target pointer index width");

#ifndef MAL_GENERATED_INTERFACE_38A7FF8ABDD4F328_H
#define MAL_GENERATED_INTERFACE_38A7FF8ABDD4F328_H
/* Host-visible types */

#ifndef MAL_DETAIL_RAW_REPR_4647bb25c84fca76_DECLARED
#define MAL_DETAIL_RAW_REPR_4647bb25c84fca76_DECLARED
typedef struct MalRepr_Sum_4647bb25c84fca76 MalRepr_Sum_4647bb25c84fca76;
#endif
#ifndef MAL_DETAIL_RAW_REPR_fac85322fcb836ab_DECLARED
#define MAL_DETAIL_RAW_REPR_fac85322fcb836ab_DECLARED
typedef struct MalRepr_Product_fac85322fcb836ab MalRepr_Product_fac85322fcb836ab;
#endif
#ifndef MAL_DETAIL_RAW_REPR_4658c725c85e520d_DECLARED
#define MAL_DETAIL_RAW_REPR_4658c725c85e520d_DECLARED
typedef struct MalRepr_Sum_4658c725c85e520d MalRepr_Sum_4658c725c85e520d;
#endif
#ifndef MAL_DETAIL_RAW_REPR_1a78737f3c5897b2_DECLARED
#define MAL_DETAIL_RAW_REPR_1a78737f3c5897b2_DECLARED
typedef struct MalRepr_Product_1a78737f3c5897b2 MalRepr_Product_1a78737f3c5897b2;
#endif
#ifndef MAL_DETAIL_RAW_REPR_1e5c21e9f35816c1_DECLARED
#define MAL_DETAIL_RAW_REPR_1e5c21e9f35816c1_DECLARED
typedef struct MalRepr_Product_1e5c21e9f35816c1 MalRepr_Product_1e5c21e9f35816c1;
#endif
#ifndef MAL_DETAIL_RAW_REPR_4647c725c84fdeda_DECLARED
#define MAL_DETAIL_RAW_REPR_4647c725c84fdeda_DECLARED
typedef struct MalRepr_Sum_4647c725c84fdeda MalRepr_Sum_4647c725c84fdeda;
#endif
#ifndef MAL_DETAIL_RAW_REPR_bb0fbcc1b817345c_DECLARED
#define MAL_DETAIL_RAW_REPR_bb0fbcc1b817345c_DECLARED
typedef struct MalRepr_Product_bb0fbcc1b817345c MalRepr_Product_bb0fbcc1b817345c;
#endif
#ifndef MAL_DETAIL_RAW_REPR_463d9725c84738c5_DECLARED
#define MAL_DETAIL_RAW_REPR_463d9725c84738c5_DECLARED
typedef struct MalRepr_Sum_463d9725c84738c5 MalRepr_Sum_463d9725c84738c5;
#endif
#ifndef MAL_DETAIL_RAW_REPR_03ed250e4033fba7_DECLARED
#define MAL_DETAIL_RAW_REPR_03ed250e4033fba7_DECLARED
typedef struct MalRepr_Product_03ed250e4033fba7 MalRepr_Product_03ed250e4033fba7;
#endif
#ifndef MAL_DETAIL_RAW_REPR_4662f725c866f822_DECLARED
#define MAL_DETAIL_RAW_REPR_4662f725c866f822_DECLARED
typedef struct MalRepr_Sum_4662f725c866f822 MalRepr_Sum_4662f725c866f822;
#endif
#ifndef MAL_DETAIL_RAW_REPR_bb3be5c1b83cb4f2_DECLARED
#define MAL_DETAIL_RAW_REPR_bb3be5c1b83cb4f2_DECLARED
typedef struct MalRepr_Product_bb3be5c1b83cb4f2 MalRepr_Product_bb3be5c1b83cb4f2;
#endif
#ifndef MAL_DETAIL_RAW_REPR_46708725c872772e_DECLARED
#define MAL_DETAIL_RAW_REPR_46708725c872772e_DECLARED
typedef struct MalRepr_Sum_46708725c872772e MalRepr_Sum_46708725c872772e;
#endif

#ifndef MAL_DETAIL_RAW_REPR_4647bb25c84fca76_DEFINED
#define MAL_DETAIL_RAW_REPR_4647bb25c84fca76_DEFINED
struct MalRepr_Sum_4647bb25c84fca76 {
    uint32_t tag;
    union {
        MalType_Unit variant_0;
        MalType_Address variant_1;
    } payload;
};

#endif

#ifndef MAL_DETAIL_RAW_REPR_fac85322fcb836ab_DEFINED
#define MAL_DETAIL_RAW_REPR_fac85322fcb836ab_DEFINED
struct MalRepr_Product_fac85322fcb836ab {
    MalRepr_Sum_4647bb25c84fca76 field_0;
    MalType_ByteSize field_1;
    MalType_Int32 field_2;
    MalType_Int32 field_3;
    MalType_Int32 field_4;
    MalType_UInt64 field_5;
};

#endif

#ifndef MAL_DETAIL_RAW_REPR_4658c725c85e520d_DEFINED
#define MAL_DETAIL_RAW_REPR_4658c725c85e520d_DEFINED
struct MalRepr_Sum_4658c725c85e520d {
    uint32_t tag;
    union {
        MalType_Address variant_0;
        MalType_UInt32 variant_1;
    } payload;
};

#endif

#ifndef MAL_DETAIL_RAW_REPR_1a78737f3c5897b2_DEFINED
#define MAL_DETAIL_RAW_REPR_1a78737f3c5897b2_DEFINED
struct MalRepr_Product_1a78737f3c5897b2 {
    MalType_Address field_0;
    MalType_ByteSize field_1;
    MalType_ByteSize field_2;
    MalType_Int32 field_3;
};

#endif

#ifndef MAL_DETAIL_RAW_REPR_1e5c21e9f35816c1_DEFINED
#define MAL_DETAIL_RAW_REPR_1e5c21e9f35816c1_DEFINED
struct MalRepr_Product_1e5c21e9f35816c1 {
    MalType_Address field_0;
    MalType_ByteSize field_1;
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

#ifndef MAL_DETAIL_RAW_REPR_bb0fbcc1b817345c_DEFINED
#define MAL_DETAIL_RAW_REPR_bb0fbcc1b817345c_DEFINED
struct MalRepr_Product_bb0fbcc1b817345c {
    MalType_Int32 field_0;
    MalType_Address field_1;
    MalType_Int32 field_2;
    MalType_UInt32 field_3;
};

#endif

#ifndef MAL_DETAIL_RAW_REPR_463d9725c84738c5_DEFINED
#define MAL_DETAIL_RAW_REPR_463d9725c84738c5_DEFINED
struct MalRepr_Sum_463d9725c84738c5 {
    uint32_t tag;
    union {
        MalType_Int32 variant_0;
        MalType_UInt32 variant_1;
    } payload;
};

#endif

#ifndef MAL_DETAIL_RAW_REPR_03ed250e4033fba7_DEFINED
#define MAL_DETAIL_RAW_REPR_03ed250e4033fba7_DEFINED
struct MalRepr_Product_03ed250e4033fba7 {
    MalType_Int32 field_0;
    MalType_Int64 field_1;
    MalType_Int32 field_2;
};

#endif

#ifndef MAL_DETAIL_RAW_REPR_4662f725c866f822_DEFINED
#define MAL_DETAIL_RAW_REPR_4662f725c866f822_DEFINED
struct MalRepr_Sum_4662f725c866f822 {
    uint32_t tag;
    union {
        MalType_UInt64 variant_0;
        MalType_UInt32 variant_1;
    } payload;
};

#endif

#ifndef MAL_DETAIL_RAW_REPR_bb3be5c1b83cb4f2_DEFINED
#define MAL_DETAIL_RAW_REPR_bb3be5c1b83cb4f2_DEFINED
struct MalRepr_Product_bb3be5c1b83cb4f2 {
    MalType_Int32 field_0;
    MalType_Address field_1;
    MalType_ByteSize field_2;
    MalType_ByteSize field_3;
};

#endif

#ifndef MAL_DETAIL_RAW_REPR_46708725c872772e_DEFINED
#define MAL_DETAIL_RAW_REPR_46708725c872772e_DEFINED
struct MalRepr_Sum_46708725c872772e {
    uint32_t tag;
    union {
        MalType_ByteSize variant_0;
        MalType_UInt32 variant_1;
    } payload;
};

#endif

typedef MalRepr_Sum_4647bb25c84fca76 MalType_OptionalAddress;
typedef MalRepr_Product_fac85322fcb836ab MalType_MapRequest;
typedef MalRepr_Product_1a78737f3c5897b2 MalType_ResizeRequest;
typedef MalRepr_Sum_4658c725c85e520d MalType_AddressResult;
typedef MalRepr_Sum_463d9725c84738c5 MalType_FileDescriptorResult;
typedef MalRepr_Sum_4662f725c866f822 MalType_OffsetResult;
typedef MalRepr_Sum_46708725c872772e MalType_TransferResult;
typedef MalRepr_Sum_4647c725c84fdeda MalType_SyscallStatus;

#ifndef MAL_DETAIL_HOST_REPR_4647bb25c84fca76_DECLARED
#define MAL_DETAIL_HOST_REPR_4647bb25c84fca76_DECLARED
typedef struct mal_detail_repr_sum_4647bb25c84fca76 mal_repr_sum_4647bb25c84fca76_t;
#endif
#ifndef MAL_DETAIL_HOST_REPR_fac85322fcb836ab_DECLARED
#define MAL_DETAIL_HOST_REPR_fac85322fcb836ab_DECLARED
typedef struct mal_detail_repr_product_fac85322fcb836ab mal_repr_product_fac85322fcb836ab_t;
#endif
#ifndef MAL_DETAIL_HOST_REPR_4658c725c85e520d_DECLARED
#define MAL_DETAIL_HOST_REPR_4658c725c85e520d_DECLARED
typedef struct mal_detail_repr_sum_4658c725c85e520d mal_repr_sum_4658c725c85e520d_t;
#endif
#ifndef MAL_DETAIL_HOST_REPR_1a78737f3c5897b2_DECLARED
#define MAL_DETAIL_HOST_REPR_1a78737f3c5897b2_DECLARED
typedef struct mal_detail_repr_product_1a78737f3c5897b2 mal_repr_product_1a78737f3c5897b2_t;
#endif
#ifndef MAL_DETAIL_HOST_REPR_1e5c21e9f35816c1_DECLARED
#define MAL_DETAIL_HOST_REPR_1e5c21e9f35816c1_DECLARED
typedef struct mal_detail_repr_product_1e5c21e9f35816c1 mal_repr_product_1e5c21e9f35816c1_t;
#endif
#ifndef MAL_DETAIL_HOST_REPR_4647c725c84fdeda_DECLARED
#define MAL_DETAIL_HOST_REPR_4647c725c84fdeda_DECLARED
typedef struct mal_detail_repr_sum_4647c725c84fdeda mal_repr_sum_4647c725c84fdeda_t;
#endif
#ifndef MAL_DETAIL_HOST_REPR_bb0fbcc1b817345c_DECLARED
#define MAL_DETAIL_HOST_REPR_bb0fbcc1b817345c_DECLARED
typedef struct mal_detail_repr_product_bb0fbcc1b817345c mal_repr_product_bb0fbcc1b817345c_t;
#endif
#ifndef MAL_DETAIL_HOST_REPR_463d9725c84738c5_DECLARED
#define MAL_DETAIL_HOST_REPR_463d9725c84738c5_DECLARED
typedef struct mal_detail_repr_sum_463d9725c84738c5 mal_repr_sum_463d9725c84738c5_t;
#endif
#ifndef MAL_DETAIL_HOST_REPR_03ed250e4033fba7_DECLARED
#define MAL_DETAIL_HOST_REPR_03ed250e4033fba7_DECLARED
typedef struct mal_detail_repr_product_03ed250e4033fba7 mal_repr_product_03ed250e4033fba7_t;
#endif
#ifndef MAL_DETAIL_HOST_REPR_4662f725c866f822_DECLARED
#define MAL_DETAIL_HOST_REPR_4662f725c866f822_DECLARED
typedef struct mal_detail_repr_sum_4662f725c866f822 mal_repr_sum_4662f725c866f822_t;
#endif
#ifndef MAL_DETAIL_HOST_REPR_bb3be5c1b83cb4f2_DECLARED
#define MAL_DETAIL_HOST_REPR_bb3be5c1b83cb4f2_DECLARED
typedef struct mal_detail_repr_product_bb3be5c1b83cb4f2 mal_repr_product_bb3be5c1b83cb4f2_t;
#endif
#ifndef MAL_DETAIL_HOST_REPR_46708725c872772e_DECLARED
#define MAL_DETAIL_HOST_REPR_46708725c872772e_DECLARED
typedef struct mal_detail_repr_sum_46708725c872772e mal_repr_sum_46708725c872772e_t;
#endif
typedef mal_repr_sum_4647bb25c84fca76_t mal_OptionalAddress_t;
typedef mal_repr_product_fac85322fcb836ab_t mal_MapRequest_t;
typedef mal_repr_product_1a78737f3c5897b2_t mal_ResizeRequest_t;
typedef mal_repr_sum_4658c725c85e520d_t mal_AddressResult_t;
typedef mal_repr_sum_463d9725c84738c5_t mal_FileDescriptorResult_t;
typedef mal_repr_sum_4662f725c866f822_t mal_OffsetResult_t;
typedef mal_repr_sum_46708725c872772e_t mal_TransferResult_t;
typedef mal_repr_sum_4647c725c84fdeda_t mal_SyscallStatus_t;

#ifndef MAL_DETAIL_HOST_REPR_4647bb25c84fca76_DEFINED
#define MAL_DETAIL_HOST_REPR_4647bb25c84fca76_DEFINED
struct mal_detail_repr_sum_4647bb25c84fca76 {
    uint32_t tag;
    union {
        mal_Unit_t variant_0;
        mal_Address_t variant_1;
    } payload;
};
#endif

#ifndef MAL_DETAIL_HOST_REPR_fac85322fcb836ab_DEFINED
#define MAL_DETAIL_HOST_REPR_fac85322fcb836ab_DEFINED
struct mal_detail_repr_product_fac85322fcb836ab {
    mal_repr_sum_4647bb25c84fca76_t field_0;
    mal_ByteSize_t field_1;
    mal_Int32_t field_2;
    mal_Int32_t field_3;
    mal_Int32_t field_4;
    mal_UInt64_t field_5;
};
#endif

#ifndef MAL_DETAIL_HOST_REPR_4658c725c85e520d_DEFINED
#define MAL_DETAIL_HOST_REPR_4658c725c85e520d_DEFINED
struct mal_detail_repr_sum_4658c725c85e520d {
    uint32_t tag;
    union {
        mal_Address_t variant_0;
        mal_UInt32_t variant_1;
    } payload;
};
#endif

#ifndef MAL_DETAIL_HOST_REPR_1a78737f3c5897b2_DEFINED
#define MAL_DETAIL_HOST_REPR_1a78737f3c5897b2_DEFINED
struct mal_detail_repr_product_1a78737f3c5897b2 {
    mal_Address_t field_0;
    mal_ByteSize_t field_1;
    mal_ByteSize_t field_2;
    mal_Int32_t field_3;
};
#endif

#ifndef MAL_DETAIL_HOST_REPR_1e5c21e9f35816c1_DEFINED
#define MAL_DETAIL_HOST_REPR_1e5c21e9f35816c1_DEFINED
struct mal_detail_repr_product_1e5c21e9f35816c1 {
    mal_Address_t field_0;
    mal_ByteSize_t field_1;
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

#ifndef MAL_DETAIL_HOST_REPR_bb0fbcc1b817345c_DEFINED
#define MAL_DETAIL_HOST_REPR_bb0fbcc1b817345c_DEFINED
struct mal_detail_repr_product_bb0fbcc1b817345c {
    mal_Int32_t field_0;
    mal_Address_t field_1;
    mal_Int32_t field_2;
    mal_UInt32_t field_3;
};
#endif

#ifndef MAL_DETAIL_HOST_REPR_463d9725c84738c5_DEFINED
#define MAL_DETAIL_HOST_REPR_463d9725c84738c5_DEFINED
struct mal_detail_repr_sum_463d9725c84738c5 {
    uint32_t tag;
    union {
        mal_Int32_t variant_0;
        mal_UInt32_t variant_1;
    } payload;
};
#endif

#ifndef MAL_DETAIL_HOST_REPR_03ed250e4033fba7_DEFINED
#define MAL_DETAIL_HOST_REPR_03ed250e4033fba7_DEFINED
struct mal_detail_repr_product_03ed250e4033fba7 {
    mal_Int32_t field_0;
    mal_Int64_t field_1;
    mal_Int32_t field_2;
};
#endif

#ifndef MAL_DETAIL_HOST_REPR_4662f725c866f822_DEFINED
#define MAL_DETAIL_HOST_REPR_4662f725c866f822_DEFINED
struct mal_detail_repr_sum_4662f725c866f822 {
    uint32_t tag;
    union {
        mal_UInt64_t variant_0;
        mal_UInt32_t variant_1;
    } payload;
};
#endif

#ifndef MAL_DETAIL_HOST_REPR_bb3be5c1b83cb4f2_DEFINED
#define MAL_DETAIL_HOST_REPR_bb3be5c1b83cb4f2_DEFINED
struct mal_detail_repr_product_bb3be5c1b83cb4f2 {
    mal_Int32_t field_0;
    mal_Address_t field_1;
    mal_ByteSize_t field_2;
    mal_ByteSize_t field_3;
};
#endif

#ifndef MAL_DETAIL_HOST_REPR_46708725c872772e_DEFINED
#define MAL_DETAIL_HOST_REPR_46708725c872772e_DEFINED
struct mal_detail_repr_sum_46708725c872772e {
    uint32_t tag;
    union {
        mal_ByteSize_t variant_0;
        mal_UInt32_t variant_1;
    } payload;
};
#endif

/* Type helpers */

#ifndef MAL_DETAIL_HOST_REPR_4647bb25c84fca76_HELPERS
#define MAL_DETAIL_HOST_REPR_4647bb25c84fca76_HELPERS
static inline mal_repr_sum_4647bb25c84fca76_t mal_detail_to_host_4647bb25c84fca76(mal_call_t *call, MalRepr_Sum_4647bb25c84fca76 value) {
    switch (value.tag) {
        case UINT32_C(0): {
            return (mal_repr_sum_4647bb25c84fca76_t){ .tag = UINT32_C(0), .payload.variant_0 = value.payload.variant_0 };
        }
        case UINT32_C(1): {
            return (mal_repr_sum_4647bb25c84fca76_t){ .tag = UINT32_C(1), .payload.variant_1 = value.payload.variant_1 };
        }
        default: {
            mal_call_trap(call, "invalid sum tag");
        }
    }
}

static inline MalRepr_Sum_4647bb25c84fca76 mal_detail_to_raw_4647bb25c84fca76(mal_call_t *call, mal_repr_sum_4647bb25c84fca76_t value) {
    switch (value.tag) {
        case UINT32_C(0): {
            return (MalRepr_Sum_4647bb25c84fca76){ .tag = UINT32_C(0), .payload.variant_0 = (MalType_Unit){ 0 } };
        }
        case UINT32_C(1): {
            return (MalRepr_Sum_4647bb25c84fca76){ .tag = UINT32_C(1), .payload.variant_1 = mal_Address_return(call, value.payload.variant_1) };
        }
        default: {
            mal_call_trap(call, "invalid sum tag");
        }
    }
}

#define mal_repr_sum_4647bb25c84fca76_tag_0 UINT32_C(0)
static inline mal_repr_sum_4647bb25c84fca76_t mal_repr_sum_4647bb25c84fca76_make_0(void) {
    return (mal_repr_sum_4647bb25c84fca76_t){ .tag = mal_repr_sum_4647bb25c84fca76_tag_0, .payload.variant_0 = (mal_Unit_t){ 0 } };
}

static inline MalRepr_Sum_4647bb25c84fca76 mal_repr_sum_4647bb25c84fca76_return_0(mal_call_t *call) {
    return mal_detail_to_raw_4647bb25c84fca76(call, (mal_repr_sum_4647bb25c84fca76_t){ .tag = mal_repr_sum_4647bb25c84fca76_tag_0, .payload.variant_0 = (mal_Unit_t){ 0 } });
}

#define mal_repr_sum_4647bb25c84fca76_tag_1 UINT32_C(1)
static inline mal_repr_sum_4647bb25c84fca76_t mal_repr_sum_4647bb25c84fca76_make_1(mal_Address_t value) {
    return (mal_repr_sum_4647bb25c84fca76_t){ .tag = mal_repr_sum_4647bb25c84fca76_tag_1, .payload.variant_1 = value };
}

static inline MalRepr_Sum_4647bb25c84fca76 mal_repr_sum_4647bb25c84fca76_return_1(mal_call_t *call, mal_Address_t value) {
    return mal_detail_to_raw_4647bb25c84fca76(call, (mal_repr_sum_4647bb25c84fca76_t){ .tag = mal_repr_sum_4647bb25c84fca76_tag_1, .payload.variant_1 = value });
}

#endif

#ifndef MAL_DETAIL_HOST_REPR_fac85322fcb836ab_HELPERS
#define MAL_DETAIL_HOST_REPR_fac85322fcb836ab_HELPERS
static inline MalRepr_Product_fac85322fcb836ab mal_repr_product_fac85322fcb836ab_return(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, mal_repr_product_fac85322fcb836ab_t value) {
    return (MalRepr_Product_fac85322fcb836ab){ .field_0 = mal_detail_to_raw_4647bb25c84fca76(call, value.field_0), .field_1 = value.field_1, .field_2 = value.field_2, .field_3 = value.field_3, .field_4 = value.field_4, .field_5 = value.field_5 };
}

#endif

#ifndef MAL_DETAIL_HOST_REPR_4658c725c85e520d_HELPERS
#define MAL_DETAIL_HOST_REPR_4658c725c85e520d_HELPERS
static inline mal_repr_sum_4658c725c85e520d_t mal_detail_to_host_4658c725c85e520d(mal_call_t *call, MalRepr_Sum_4658c725c85e520d value) {
    switch (value.tag) {
        case UINT32_C(0): {
            return (mal_repr_sum_4658c725c85e520d_t){ .tag = UINT32_C(0), .payload.variant_0 = value.payload.variant_0 };
        }
        case UINT32_C(1): {
            return (mal_repr_sum_4658c725c85e520d_t){ .tag = UINT32_C(1), .payload.variant_1 = value.payload.variant_1 };
        }
        default: {
            mal_call_trap(call, "invalid sum tag");
        }
    }
}

static inline MalRepr_Sum_4658c725c85e520d mal_detail_to_raw_4658c725c85e520d(mal_call_t *call, mal_repr_sum_4658c725c85e520d_t value) {
    switch (value.tag) {
        case UINT32_C(0): {
            return (MalRepr_Sum_4658c725c85e520d){ .tag = UINT32_C(0), .payload.variant_0 = mal_Address_return(call, value.payload.variant_0) };
        }
        case UINT32_C(1): {
            return (MalRepr_Sum_4658c725c85e520d){ .tag = UINT32_C(1), .payload.variant_1 = value.payload.variant_1 };
        }
        default: {
            mal_call_trap(call, "invalid sum tag");
        }
    }
}

#define mal_repr_sum_4658c725c85e520d_tag_0 UINT32_C(0)
static inline mal_repr_sum_4658c725c85e520d_t mal_repr_sum_4658c725c85e520d_make_0(mal_Address_t value) {
    return (mal_repr_sum_4658c725c85e520d_t){ .tag = mal_repr_sum_4658c725c85e520d_tag_0, .payload.variant_0 = value };
}

static inline MalRepr_Sum_4658c725c85e520d mal_repr_sum_4658c725c85e520d_return_0(mal_call_t *call, mal_Address_t value) {
    return mal_detail_to_raw_4658c725c85e520d(call, (mal_repr_sum_4658c725c85e520d_t){ .tag = mal_repr_sum_4658c725c85e520d_tag_0, .payload.variant_0 = value });
}

#define mal_repr_sum_4658c725c85e520d_tag_1 UINT32_C(1)
static inline mal_repr_sum_4658c725c85e520d_t mal_repr_sum_4658c725c85e520d_make_1(mal_UInt32_t value) {
    return (mal_repr_sum_4658c725c85e520d_t){ .tag = mal_repr_sum_4658c725c85e520d_tag_1, .payload.variant_1 = value };
}

static inline MalRepr_Sum_4658c725c85e520d mal_repr_sum_4658c725c85e520d_return_1(mal_call_t *call, mal_UInt32_t value) {
    return mal_detail_to_raw_4658c725c85e520d(call, (mal_repr_sum_4658c725c85e520d_t){ .tag = mal_repr_sum_4658c725c85e520d_tag_1, .payload.variant_1 = value });
}

#endif

#ifndef MAL_DETAIL_HOST_REPR_1a78737f3c5897b2_HELPERS
#define MAL_DETAIL_HOST_REPR_1a78737f3c5897b2_HELPERS
static inline MalRepr_Product_1a78737f3c5897b2 mal_repr_product_1a78737f3c5897b2_return(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, mal_repr_product_1a78737f3c5897b2_t value) {
    return (MalRepr_Product_1a78737f3c5897b2){ .field_0 = mal_Address_return(call, value.field_0), .field_1 = value.field_1, .field_2 = value.field_2, .field_3 = value.field_3 };
}

#endif

#ifndef MAL_DETAIL_HOST_REPR_1e5c21e9f35816c1_HELPERS
#define MAL_DETAIL_HOST_REPR_1e5c21e9f35816c1_HELPERS
static inline MalRepr_Product_1e5c21e9f35816c1 mal_repr_product_1e5c21e9f35816c1_return(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, mal_repr_product_1e5c21e9f35816c1_t value) {
    return (MalRepr_Product_1e5c21e9f35816c1){ .field_0 = mal_Address_return(call, value.field_0), .field_1 = value.field_1 };
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

#ifndef MAL_DETAIL_HOST_REPR_bb0fbcc1b817345c_HELPERS
#define MAL_DETAIL_HOST_REPR_bb0fbcc1b817345c_HELPERS
static inline MalRepr_Product_bb0fbcc1b817345c mal_repr_product_bb0fbcc1b817345c_return(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, mal_repr_product_bb0fbcc1b817345c_t value) {
    return (MalRepr_Product_bb0fbcc1b817345c){ .field_0 = value.field_0, .field_1 = mal_Address_return(call, value.field_1), .field_2 = value.field_2, .field_3 = value.field_3 };
}

#endif

#ifndef MAL_DETAIL_HOST_REPR_463d9725c84738c5_HELPERS
#define MAL_DETAIL_HOST_REPR_463d9725c84738c5_HELPERS
static inline mal_repr_sum_463d9725c84738c5_t mal_detail_to_host_463d9725c84738c5(mal_call_t *call, MalRepr_Sum_463d9725c84738c5 value) {
    switch (value.tag) {
        case UINT32_C(0): {
            return (mal_repr_sum_463d9725c84738c5_t){ .tag = UINT32_C(0), .payload.variant_0 = value.payload.variant_0 };
        }
        case UINT32_C(1): {
            return (mal_repr_sum_463d9725c84738c5_t){ .tag = UINT32_C(1), .payload.variant_1 = value.payload.variant_1 };
        }
        default: {
            mal_call_trap(call, "invalid sum tag");
        }
    }
}

static inline MalRepr_Sum_463d9725c84738c5 mal_detail_to_raw_463d9725c84738c5(mal_call_t *call, mal_repr_sum_463d9725c84738c5_t value) {
    switch (value.tag) {
        case UINT32_C(0): {
            return (MalRepr_Sum_463d9725c84738c5){ .tag = UINT32_C(0), .payload.variant_0 = value.payload.variant_0 };
        }
        case UINT32_C(1): {
            return (MalRepr_Sum_463d9725c84738c5){ .tag = UINT32_C(1), .payload.variant_1 = value.payload.variant_1 };
        }
        default: {
            mal_call_trap(call, "invalid sum tag");
        }
    }
}

#define mal_repr_sum_463d9725c84738c5_tag_0 UINT32_C(0)
static inline mal_repr_sum_463d9725c84738c5_t mal_repr_sum_463d9725c84738c5_make_0(mal_Int32_t value) {
    return (mal_repr_sum_463d9725c84738c5_t){ .tag = mal_repr_sum_463d9725c84738c5_tag_0, .payload.variant_0 = value };
}

static inline MalRepr_Sum_463d9725c84738c5 mal_repr_sum_463d9725c84738c5_return_0(mal_call_t *call, mal_Int32_t value) {
    return mal_detail_to_raw_463d9725c84738c5(call, (mal_repr_sum_463d9725c84738c5_t){ .tag = mal_repr_sum_463d9725c84738c5_tag_0, .payload.variant_0 = value });
}

#define mal_repr_sum_463d9725c84738c5_tag_1 UINT32_C(1)
static inline mal_repr_sum_463d9725c84738c5_t mal_repr_sum_463d9725c84738c5_make_1(mal_UInt32_t value) {
    return (mal_repr_sum_463d9725c84738c5_t){ .tag = mal_repr_sum_463d9725c84738c5_tag_1, .payload.variant_1 = value };
}

static inline MalRepr_Sum_463d9725c84738c5 mal_repr_sum_463d9725c84738c5_return_1(mal_call_t *call, mal_UInt32_t value) {
    return mal_detail_to_raw_463d9725c84738c5(call, (mal_repr_sum_463d9725c84738c5_t){ .tag = mal_repr_sum_463d9725c84738c5_tag_1, .payload.variant_1 = value });
}

#endif

#ifndef MAL_DETAIL_HOST_REPR_03ed250e4033fba7_HELPERS
#define MAL_DETAIL_HOST_REPR_03ed250e4033fba7_HELPERS
static inline MalRepr_Product_03ed250e4033fba7 mal_repr_product_03ed250e4033fba7_return(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, mal_repr_product_03ed250e4033fba7_t value) {
    return (MalRepr_Product_03ed250e4033fba7){ .field_0 = value.field_0, .field_1 = value.field_1, .field_2 = value.field_2 };
}

#endif

#ifndef MAL_DETAIL_HOST_REPR_4662f725c866f822_HELPERS
#define MAL_DETAIL_HOST_REPR_4662f725c866f822_HELPERS
static inline mal_repr_sum_4662f725c866f822_t mal_detail_to_host_4662f725c866f822(mal_call_t *call, MalRepr_Sum_4662f725c866f822 value) {
    switch (value.tag) {
        case UINT32_C(0): {
            return (mal_repr_sum_4662f725c866f822_t){ .tag = UINT32_C(0), .payload.variant_0 = value.payload.variant_0 };
        }
        case UINT32_C(1): {
            return (mal_repr_sum_4662f725c866f822_t){ .tag = UINT32_C(1), .payload.variant_1 = value.payload.variant_1 };
        }
        default: {
            mal_call_trap(call, "invalid sum tag");
        }
    }
}

static inline MalRepr_Sum_4662f725c866f822 mal_detail_to_raw_4662f725c866f822(mal_call_t *call, mal_repr_sum_4662f725c866f822_t value) {
    switch (value.tag) {
        case UINT32_C(0): {
            return (MalRepr_Sum_4662f725c866f822){ .tag = UINT32_C(0), .payload.variant_0 = value.payload.variant_0 };
        }
        case UINT32_C(1): {
            return (MalRepr_Sum_4662f725c866f822){ .tag = UINT32_C(1), .payload.variant_1 = value.payload.variant_1 };
        }
        default: {
            mal_call_trap(call, "invalid sum tag");
        }
    }
}

#define mal_repr_sum_4662f725c866f822_tag_0 UINT32_C(0)
static inline mal_repr_sum_4662f725c866f822_t mal_repr_sum_4662f725c866f822_make_0(mal_UInt64_t value) {
    return (mal_repr_sum_4662f725c866f822_t){ .tag = mal_repr_sum_4662f725c866f822_tag_0, .payload.variant_0 = value };
}

static inline MalRepr_Sum_4662f725c866f822 mal_repr_sum_4662f725c866f822_return_0(mal_call_t *call, mal_UInt64_t value) {
    return mal_detail_to_raw_4662f725c866f822(call, (mal_repr_sum_4662f725c866f822_t){ .tag = mal_repr_sum_4662f725c866f822_tag_0, .payload.variant_0 = value });
}

#define mal_repr_sum_4662f725c866f822_tag_1 UINT32_C(1)
static inline mal_repr_sum_4662f725c866f822_t mal_repr_sum_4662f725c866f822_make_1(mal_UInt32_t value) {
    return (mal_repr_sum_4662f725c866f822_t){ .tag = mal_repr_sum_4662f725c866f822_tag_1, .payload.variant_1 = value };
}

static inline MalRepr_Sum_4662f725c866f822 mal_repr_sum_4662f725c866f822_return_1(mal_call_t *call, mal_UInt32_t value) {
    return mal_detail_to_raw_4662f725c866f822(call, (mal_repr_sum_4662f725c866f822_t){ .tag = mal_repr_sum_4662f725c866f822_tag_1, .payload.variant_1 = value });
}

#endif

#ifndef MAL_DETAIL_HOST_REPR_bb3be5c1b83cb4f2_HELPERS
#define MAL_DETAIL_HOST_REPR_bb3be5c1b83cb4f2_HELPERS
static inline MalRepr_Product_bb3be5c1b83cb4f2 mal_repr_product_bb3be5c1b83cb4f2_return(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, mal_repr_product_bb3be5c1b83cb4f2_t value) {
    return (MalRepr_Product_bb3be5c1b83cb4f2){ .field_0 = value.field_0, .field_1 = mal_Address_return(call, value.field_1), .field_2 = value.field_2, .field_3 = value.field_3 };
}

#endif

#ifndef MAL_DETAIL_HOST_REPR_46708725c872772e_HELPERS
#define MAL_DETAIL_HOST_REPR_46708725c872772e_HELPERS
static inline mal_repr_sum_46708725c872772e_t mal_detail_to_host_46708725c872772e(mal_call_t *call, MalRepr_Sum_46708725c872772e value) {
    switch (value.tag) {
        case UINT32_C(0): {
            return (mal_repr_sum_46708725c872772e_t){ .tag = UINT32_C(0), .payload.variant_0 = value.payload.variant_0 };
        }
        case UINT32_C(1): {
            return (mal_repr_sum_46708725c872772e_t){ .tag = UINT32_C(1), .payload.variant_1 = value.payload.variant_1 };
        }
        default: {
            mal_call_trap(call, "invalid sum tag");
        }
    }
}

static inline MalRepr_Sum_46708725c872772e mal_detail_to_raw_46708725c872772e(mal_call_t *call, mal_repr_sum_46708725c872772e_t value) {
    switch (value.tag) {
        case UINT32_C(0): {
            return (MalRepr_Sum_46708725c872772e){ .tag = UINT32_C(0), .payload.variant_0 = value.payload.variant_0 };
        }
        case UINT32_C(1): {
            return (MalRepr_Sum_46708725c872772e){ .tag = UINT32_C(1), .payload.variant_1 = value.payload.variant_1 };
        }
        default: {
            mal_call_trap(call, "invalid sum tag");
        }
    }
}

#define mal_repr_sum_46708725c872772e_tag_0 UINT32_C(0)
static inline mal_repr_sum_46708725c872772e_t mal_repr_sum_46708725c872772e_make_0(mal_ByteSize_t value) {
    return (mal_repr_sum_46708725c872772e_t){ .tag = mal_repr_sum_46708725c872772e_tag_0, .payload.variant_0 = value };
}

static inline MalRepr_Sum_46708725c872772e mal_repr_sum_46708725c872772e_return_0(mal_call_t *call, mal_ByteSize_t value) {
    return mal_detail_to_raw_46708725c872772e(call, (mal_repr_sum_46708725c872772e_t){ .tag = mal_repr_sum_46708725c872772e_tag_0, .payload.variant_0 = value });
}

#define mal_repr_sum_46708725c872772e_tag_1 UINT32_C(1)
static inline mal_repr_sum_46708725c872772e_t mal_repr_sum_46708725c872772e_make_1(mal_UInt32_t value) {
    return (mal_repr_sum_46708725c872772e_t){ .tag = mal_repr_sum_46708725c872772e_tag_1, .payload.variant_1 = value };
}

static inline MalRepr_Sum_46708725c872772e mal_repr_sum_46708725c872772e_return_1(mal_call_t *call, mal_UInt32_t value) {
    return mal_detail_to_raw_46708725c872772e(call, (mal_repr_sum_46708725c872772e_t){ .tag = mal_repr_sum_46708725c872772e_tag_1, .payload.variant_1 = value });
}

#endif

#define mal_OptionalAddress_tag_0 UINT32_C(0)
static inline mal_OptionalAddress_t mal_OptionalAddress_make_0(void) {
    return (mal_OptionalAddress_t){ .tag = mal_OptionalAddress_tag_0, .payload.variant_0 = (mal_Unit_t){ 0 } };
}

static inline MalType_OptionalAddress mal_OptionalAddress_return_0(mal_call_t *call) {
    return mal_detail_to_raw_4647bb25c84fca76(call, (mal_OptionalAddress_t){ .tag = mal_OptionalAddress_tag_0, .payload.variant_0 = (mal_Unit_t){ 0 } });
}

#define mal_OptionalAddress_tag_1 UINT32_C(1)
static inline mal_OptionalAddress_t mal_OptionalAddress_make_1(mal_Address_t value) {
    return (mal_OptionalAddress_t){ .tag = mal_OptionalAddress_tag_1, .payload.variant_1 = value };
}

static inline MalType_OptionalAddress mal_OptionalAddress_return_1(mal_call_t *call, mal_Address_t value) {
    return mal_detail_to_raw_4647bb25c84fca76(call, (mal_OptionalAddress_t){ .tag = mal_OptionalAddress_tag_1, .payload.variant_1 = value });
}

static inline MalType_MapRequest mal_MapRequest_return(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, mal_MapRequest_t value) {
    return (MalRepr_Product_fac85322fcb836ab){ .field_0 = mal_detail_to_raw_4647bb25c84fca76(call, value.field_0), .field_1 = value.field_1, .field_2 = value.field_2, .field_3 = value.field_3, .field_4 = value.field_4, .field_5 = value.field_5 };
}

static inline MalType_ResizeRequest mal_ResizeRequest_return(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, mal_ResizeRequest_t value) {
    return (MalRepr_Product_1a78737f3c5897b2){ .field_0 = mal_Address_return(call, value.field_0), .field_1 = value.field_1, .field_2 = value.field_2, .field_3 = value.field_3 };
}

#define mal_AddressResult_tag_0 UINT32_C(0)
static inline mal_AddressResult_t mal_AddressResult_make_0(mal_Address_t value) {
    return (mal_AddressResult_t){ .tag = mal_AddressResult_tag_0, .payload.variant_0 = value };
}

static inline MalType_AddressResult mal_AddressResult_return_0(mal_call_t *call, mal_Address_t value) {
    return mal_detail_to_raw_4658c725c85e520d(call, (mal_AddressResult_t){ .tag = mal_AddressResult_tag_0, .payload.variant_0 = value });
}

#define mal_AddressResult_tag_1 UINT32_C(1)
static inline mal_AddressResult_t mal_AddressResult_make_1(mal_UInt32_t value) {
    return (mal_AddressResult_t){ .tag = mal_AddressResult_tag_1, .payload.variant_1 = value };
}

static inline MalType_AddressResult mal_AddressResult_return_1(mal_call_t *call, mal_UInt32_t value) {
    return mal_detail_to_raw_4658c725c85e520d(call, (mal_AddressResult_t){ .tag = mal_AddressResult_tag_1, .payload.variant_1 = value });
}

#define mal_FileDescriptorResult_tag_0 UINT32_C(0)
static inline mal_FileDescriptorResult_t mal_FileDescriptorResult_make_0(mal_Int32_t value) {
    return (mal_FileDescriptorResult_t){ .tag = mal_FileDescriptorResult_tag_0, .payload.variant_0 = value };
}

static inline MalType_FileDescriptorResult mal_FileDescriptorResult_return_0(mal_call_t *call, mal_Int32_t value) {
    return mal_detail_to_raw_463d9725c84738c5(call, (mal_FileDescriptorResult_t){ .tag = mal_FileDescriptorResult_tag_0, .payload.variant_0 = value });
}

#define mal_FileDescriptorResult_tag_1 UINT32_C(1)
static inline mal_FileDescriptorResult_t mal_FileDescriptorResult_make_1(mal_UInt32_t value) {
    return (mal_FileDescriptorResult_t){ .tag = mal_FileDescriptorResult_tag_1, .payload.variant_1 = value };
}

static inline MalType_FileDescriptorResult mal_FileDescriptorResult_return_1(mal_call_t *call, mal_UInt32_t value) {
    return mal_detail_to_raw_463d9725c84738c5(call, (mal_FileDescriptorResult_t){ .tag = mal_FileDescriptorResult_tag_1, .payload.variant_1 = value });
}

#define mal_OffsetResult_tag_0 UINT32_C(0)
static inline mal_OffsetResult_t mal_OffsetResult_make_0(mal_UInt64_t value) {
    return (mal_OffsetResult_t){ .tag = mal_OffsetResult_tag_0, .payload.variant_0 = value };
}

static inline MalType_OffsetResult mal_OffsetResult_return_0(mal_call_t *call, mal_UInt64_t value) {
    return mal_detail_to_raw_4662f725c866f822(call, (mal_OffsetResult_t){ .tag = mal_OffsetResult_tag_0, .payload.variant_0 = value });
}

#define mal_OffsetResult_tag_1 UINT32_C(1)
static inline mal_OffsetResult_t mal_OffsetResult_make_1(mal_UInt32_t value) {
    return (mal_OffsetResult_t){ .tag = mal_OffsetResult_tag_1, .payload.variant_1 = value };
}

static inline MalType_OffsetResult mal_OffsetResult_return_1(mal_call_t *call, mal_UInt32_t value) {
    return mal_detail_to_raw_4662f725c866f822(call, (mal_OffsetResult_t){ .tag = mal_OffsetResult_tag_1, .payload.variant_1 = value });
}

#define mal_TransferResult_tag_0 UINT32_C(0)
static inline mal_TransferResult_t mal_TransferResult_make_0(mal_ByteSize_t value) {
    return (mal_TransferResult_t){ .tag = mal_TransferResult_tag_0, .payload.variant_0 = value };
}

static inline MalType_TransferResult mal_TransferResult_return_0(mal_call_t *call, mal_ByteSize_t value) {
    return mal_detail_to_raw_46708725c872772e(call, (mal_TransferResult_t){ .tag = mal_TransferResult_tag_0, .payload.variant_0 = value });
}

#define mal_TransferResult_tag_1 UINT32_C(1)
static inline mal_TransferResult_t mal_TransferResult_make_1(mal_UInt32_t value) {
    return (mal_TransferResult_t){ .tag = mal_TransferResult_tag_1, .payload.variant_1 = value };
}

static inline MalType_TransferResult mal_TransferResult_return_1(mal_call_t *call, mal_UInt32_t value) {
    return mal_detail_to_raw_46708725c872772e(call, (mal_TransferResult_t){ .tag = mal_TransferResult_tag_1, .payload.variant_1 = value });
}

#define mal_SyscallStatus_tag_0 UINT32_C(0)
static inline mal_SyscallStatus_t mal_SyscallStatus_make_0(void) {
    return (mal_SyscallStatus_t){ .tag = mal_SyscallStatus_tag_0, .payload.variant_0 = (mal_Unit_t){ 0 } };
}

static inline MalType_SyscallStatus mal_SyscallStatus_return_0(mal_call_t *call) {
    return mal_detail_to_raw_4647c725c84fdeda(call, (mal_SyscallStatus_t){ .tag = mal_SyscallStatus_tag_0, .payload.variant_0 = (mal_Unit_t){ 0 } });
}

#define mal_SyscallStatus_tag_1 UINT32_C(1)
static inline mal_SyscallStatus_t mal_SyscallStatus_make_1(mal_UInt32_t value) {
    return (mal_SyscallStatus_t){ .tag = mal_SyscallStatus_tag_1, .payload.variant_1 = value };
}

static inline MalType_SyscallStatus mal_SyscallStatus_return_1(mal_call_t *call, mal_UInt32_t value) {
    return mal_detail_to_raw_4647c725c84fdeda(call, (mal_SyscallStatus_t){ .tag = mal_SyscallStatus_tag_1, .payload.variant_1 = value });
}

/* Canonical memory access */

#ifndef MAL_DETAIL_MEMORY_Int32_HELPERS
#define MAL_DETAIL_MEMORY_Int32_HELPERS
static inline mal_Int32_t mal_detail_memory_read_Int32(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, const uint8_t *source) {
    mal_Int32_t value;
    memcpy(&value, source, sizeof(value));
    return value;
}

static inline void mal_detail_memory_write_Int32(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, uint8_t *destination, mal_Int32_t value) {
    memcpy(destination, &value, sizeof(value));
}

#endif

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

#ifndef MAL_DETAIL_MEMORY_ByteSize_HELPERS
#define MAL_DETAIL_MEMORY_ByteSize_HELPERS
static inline mal_ByteSize_t mal_detail_memory_read_ByteSize(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, const uint8_t *source) {
    mal_ByteSize_t value;
    memcpy(&value, source, sizeof(value));
    return value;
}

static inline void mal_detail_memory_write_ByteSize(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, uint8_t *destination, mal_ByteSize_t value) {
    memcpy(destination, &value, sizeof(value));
}

#endif

#ifndef MAL_DETAIL_MEMORY_REPR_4647bb25c84fca76_HELPERS
#define MAL_DETAIL_MEMORY_REPR_4647bb25c84fca76_HELPERS
static inline mal_repr_sum_4647bb25c84fca76_t mal_detail_memory_read_4647bb25c84fca76(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, const uint8_t *source MAL_DETAIL_MAYBE_UNUSED) {
    switch (mal_detail_memory_read_UInt8(call, source)) {
        case 0: {
            return (mal_repr_sum_4647bb25c84fca76_t){ .tag = UINT32_C(0), .payload.variant_0 = (mal_Unit_t){ 0 } };
        }
        case 1: {
            return (mal_repr_sum_4647bb25c84fca76_t){ .tag = UINT32_C(1), .payload.variant_1 = mal_detail_memory_read_Address(call, source + 8) };
        }
        default: {
            mal_call_trap(call, "invalid canonical sum tag");
        }
    }
}

static inline void mal_detail_memory_write_4647bb25c84fca76(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, uint8_t *destination MAL_DETAIL_MAYBE_UNUSED, mal_repr_sum_4647bb25c84fca76_t value) {
    switch (value.tag) {
        case UINT32_C(0): {
            mal_detail_memory_write_UInt8(call, destination, (mal_UInt8_t)value.tag);
            (void)value.payload.variant_0;
            return;
        }
        case UINT32_C(1): {
            mal_detail_memory_write_UInt8(call, destination, (mal_UInt8_t)value.tag);
            mal_detail_memory_write_Address(call, destination + 8, value.payload.variant_1);
            return;
        }
        default: {
            mal_call_trap(call, "invalid sum tag");
        }
    }
}

#endif

#ifndef MAL_DETAIL_MEMORY_REPR_fac85322fcb836ab_HELPERS
#define MAL_DETAIL_MEMORY_REPR_fac85322fcb836ab_HELPERS
static inline mal_repr_product_fac85322fcb836ab_t mal_detail_memory_read_fac85322fcb836ab(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, const uint8_t *source MAL_DETAIL_MAYBE_UNUSED) {
    mal_repr_product_fac85322fcb836ab_t value;
    value.field_0 = mal_detail_memory_read_4647bb25c84fca76(call, source + 0);
    value.field_1 = mal_detail_memory_read_ByteSize(call, source + 16);
    value.field_2 = mal_detail_memory_read_Int32(call, source + 24);
    value.field_3 = mal_detail_memory_read_Int32(call, source + 28);
    value.field_4 = mal_detail_memory_read_Int32(call, source + 32);
    value.field_5 = mal_detail_memory_read_UInt64(call, source + 40);
    return value;
}

static inline void mal_detail_memory_write_fac85322fcb836ab(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, uint8_t *destination MAL_DETAIL_MAYBE_UNUSED, mal_repr_product_fac85322fcb836ab_t value) {
    mal_detail_memory_write_4647bb25c84fca76(call, destination + 0, value.field_0);
    mal_detail_memory_write_ByteSize(call, destination + 16, value.field_1);
    mal_detail_memory_write_Int32(call, destination + 24, value.field_2);
    mal_detail_memory_write_Int32(call, destination + 28, value.field_3);
    mal_detail_memory_write_Int32(call, destination + 32, value.field_4);
    mal_detail_memory_write_UInt64(call, destination + 40, value.field_5);
}

#endif

#ifndef MAL_DETAIL_MEMORY_REPR_4658c725c85e520d_HELPERS
#define MAL_DETAIL_MEMORY_REPR_4658c725c85e520d_HELPERS
static inline mal_repr_sum_4658c725c85e520d_t mal_detail_memory_read_4658c725c85e520d(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, const uint8_t *source MAL_DETAIL_MAYBE_UNUSED) {
    switch (mal_detail_memory_read_UInt8(call, source)) {
        case 0: {
            return (mal_repr_sum_4658c725c85e520d_t){ .tag = UINT32_C(0), .payload.variant_0 = mal_detail_memory_read_Address(call, source + 8) };
        }
        case 1: {
            return (mal_repr_sum_4658c725c85e520d_t){ .tag = UINT32_C(1), .payload.variant_1 = mal_detail_memory_read_UInt32(call, source + 8) };
        }
        default: {
            mal_call_trap(call, "invalid canonical sum tag");
        }
    }
}

static inline void mal_detail_memory_write_4658c725c85e520d(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, uint8_t *destination MAL_DETAIL_MAYBE_UNUSED, mal_repr_sum_4658c725c85e520d_t value) {
    switch (value.tag) {
        case UINT32_C(0): {
            mal_detail_memory_write_UInt8(call, destination, (mal_UInt8_t)value.tag);
            mal_detail_memory_write_Address(call, destination + 8, value.payload.variant_0);
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

#ifndef MAL_DETAIL_MEMORY_REPR_1a78737f3c5897b2_HELPERS
#define MAL_DETAIL_MEMORY_REPR_1a78737f3c5897b2_HELPERS
static inline mal_repr_product_1a78737f3c5897b2_t mal_detail_memory_read_1a78737f3c5897b2(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, const uint8_t *source MAL_DETAIL_MAYBE_UNUSED) {
    mal_repr_product_1a78737f3c5897b2_t value;
    value.field_0 = mal_detail_memory_read_Address(call, source + 0);
    value.field_1 = mal_detail_memory_read_ByteSize(call, source + 8);
    value.field_2 = mal_detail_memory_read_ByteSize(call, source + 16);
    value.field_3 = mal_detail_memory_read_Int32(call, source + 24);
    return value;
}

static inline void mal_detail_memory_write_1a78737f3c5897b2(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, uint8_t *destination MAL_DETAIL_MAYBE_UNUSED, mal_repr_product_1a78737f3c5897b2_t value) {
    mal_detail_memory_write_Address(call, destination + 0, value.field_0);
    mal_detail_memory_write_ByteSize(call, destination + 8, value.field_1);
    mal_detail_memory_write_ByteSize(call, destination + 16, value.field_2);
    mal_detail_memory_write_Int32(call, destination + 24, value.field_3);
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

#ifndef MAL_DETAIL_MEMORY_REPR_463d9725c84738c5_HELPERS
#define MAL_DETAIL_MEMORY_REPR_463d9725c84738c5_HELPERS
static inline mal_repr_sum_463d9725c84738c5_t mal_detail_memory_read_463d9725c84738c5(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, const uint8_t *source MAL_DETAIL_MAYBE_UNUSED) {
    switch (mal_detail_memory_read_UInt8(call, source)) {
        case 0: {
            return (mal_repr_sum_463d9725c84738c5_t){ .tag = UINT32_C(0), .payload.variant_0 = mal_detail_memory_read_Int32(call, source + 4) };
        }
        case 1: {
            return (mal_repr_sum_463d9725c84738c5_t){ .tag = UINT32_C(1), .payload.variant_1 = mal_detail_memory_read_UInt32(call, source + 4) };
        }
        default: {
            mal_call_trap(call, "invalid canonical sum tag");
        }
    }
}

static inline void mal_detail_memory_write_463d9725c84738c5(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, uint8_t *destination MAL_DETAIL_MAYBE_UNUSED, mal_repr_sum_463d9725c84738c5_t value) {
    switch (value.tag) {
        case UINT32_C(0): {
            mal_detail_memory_write_UInt8(call, destination, (mal_UInt8_t)value.tag);
            mal_detail_memory_write_Int32(call, destination + 4, value.payload.variant_0);
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

#ifndef MAL_DETAIL_MEMORY_REPR_4662f725c866f822_HELPERS
#define MAL_DETAIL_MEMORY_REPR_4662f725c866f822_HELPERS
static inline mal_repr_sum_4662f725c866f822_t mal_detail_memory_read_4662f725c866f822(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, const uint8_t *source MAL_DETAIL_MAYBE_UNUSED) {
    switch (mal_detail_memory_read_UInt8(call, source)) {
        case 0: {
            return (mal_repr_sum_4662f725c866f822_t){ .tag = UINT32_C(0), .payload.variant_0 = mal_detail_memory_read_UInt64(call, source + 8) };
        }
        case 1: {
            return (mal_repr_sum_4662f725c866f822_t){ .tag = UINT32_C(1), .payload.variant_1 = mal_detail_memory_read_UInt32(call, source + 8) };
        }
        default: {
            mal_call_trap(call, "invalid canonical sum tag");
        }
    }
}

static inline void mal_detail_memory_write_4662f725c866f822(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, uint8_t *destination MAL_DETAIL_MAYBE_UNUSED, mal_repr_sum_4662f725c866f822_t value) {
    switch (value.tag) {
        case UINT32_C(0): {
            mal_detail_memory_write_UInt8(call, destination, (mal_UInt8_t)value.tag);
            mal_detail_memory_write_UInt64(call, destination + 8, value.payload.variant_0);
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

#ifndef MAL_DETAIL_MEMORY_REPR_46708725c872772e_HELPERS
#define MAL_DETAIL_MEMORY_REPR_46708725c872772e_HELPERS
static inline mal_repr_sum_46708725c872772e_t mal_detail_memory_read_46708725c872772e(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, const uint8_t *source MAL_DETAIL_MAYBE_UNUSED) {
    switch (mal_detail_memory_read_UInt8(call, source)) {
        case 0: {
            return (mal_repr_sum_46708725c872772e_t){ .tag = UINT32_C(0), .payload.variant_0 = mal_detail_memory_read_ByteSize(call, source + 8) };
        }
        case 1: {
            return (mal_repr_sum_46708725c872772e_t){ .tag = UINT32_C(1), .payload.variant_1 = mal_detail_memory_read_UInt32(call, source + 8) };
        }
        default: {
            mal_call_trap(call, "invalid canonical sum tag");
        }
    }
}

static inline void mal_detail_memory_write_46708725c872772e(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, uint8_t *destination MAL_DETAIL_MAYBE_UNUSED, mal_repr_sum_46708725c872772e_t value) {
    switch (value.tag) {
        case UINT32_C(0): {
            mal_detail_memory_write_UInt8(call, destination, (mal_UInt8_t)value.tag);
            mal_detail_memory_write_ByteSize(call, destination + 8, value.payload.variant_0);
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

static inline mal_OptionalAddress_t mal_OptionalAddress_read(mal_call_t *call, mal_Address_t address, mal_USize_t index) {
    mal_Address_return(call, address);
    return mal_detail_memory_read_4647bb25c84fca76(call, (const uint8_t *)address + (index * 16));
}

static inline void mal_OptionalAddress_write(mal_call_t *call, mal_Address_t address, mal_USize_t index, mal_OptionalAddress_t value) {
    mal_Address_return(call, address);
    mal_detail_memory_write_4647bb25c84fca76(call, (uint8_t *)address + (index * 16), value);
}

static inline mal_MapRequest_t mal_MapRequest_read(mal_call_t *call, mal_Address_t address, mal_USize_t index) {
    mal_Address_return(call, address);
    return mal_detail_memory_read_fac85322fcb836ab(call, (const uint8_t *)address + (index * 48));
}

static inline void mal_MapRequest_write(mal_call_t *call, mal_Address_t address, mal_USize_t index, mal_MapRequest_t value) {
    mal_Address_return(call, address);
    mal_detail_memory_write_fac85322fcb836ab(call, (uint8_t *)address + (index * 48), value);
}

static inline mal_ResizeRequest_t mal_ResizeRequest_read(mal_call_t *call, mal_Address_t address, mal_USize_t index) {
    mal_Address_return(call, address);
    return mal_detail_memory_read_1a78737f3c5897b2(call, (const uint8_t *)address + (index * 32));
}

static inline void mal_ResizeRequest_write(mal_call_t *call, mal_Address_t address, mal_USize_t index, mal_ResizeRequest_t value) {
    mal_Address_return(call, address);
    mal_detail_memory_write_1a78737f3c5897b2(call, (uint8_t *)address + (index * 32), value);
}

static inline mal_AddressResult_t mal_AddressResult_read(mal_call_t *call, mal_Address_t address, mal_USize_t index) {
    mal_Address_return(call, address);
    return mal_detail_memory_read_4658c725c85e520d(call, (const uint8_t *)address + (index * 16));
}

static inline void mal_AddressResult_write(mal_call_t *call, mal_Address_t address, mal_USize_t index, mal_AddressResult_t value) {
    mal_Address_return(call, address);
    mal_detail_memory_write_4658c725c85e520d(call, (uint8_t *)address + (index * 16), value);
}

static inline mal_FileDescriptorResult_t mal_FileDescriptorResult_read(mal_call_t *call, mal_Address_t address, mal_USize_t index) {
    mal_Address_return(call, address);
    return mal_detail_memory_read_463d9725c84738c5(call, (const uint8_t *)address + (index * 8));
}

static inline void mal_FileDescriptorResult_write(mal_call_t *call, mal_Address_t address, mal_USize_t index, mal_FileDescriptorResult_t value) {
    mal_Address_return(call, address);
    mal_detail_memory_write_463d9725c84738c5(call, (uint8_t *)address + (index * 8), value);
}

static inline mal_OffsetResult_t mal_OffsetResult_read(mal_call_t *call, mal_Address_t address, mal_USize_t index) {
    mal_Address_return(call, address);
    return mal_detail_memory_read_4662f725c866f822(call, (const uint8_t *)address + (index * 16));
}

static inline void mal_OffsetResult_write(mal_call_t *call, mal_Address_t address, mal_USize_t index, mal_OffsetResult_t value) {
    mal_Address_return(call, address);
    mal_detail_memory_write_4662f725c866f822(call, (uint8_t *)address + (index * 16), value);
}

static inline mal_TransferResult_t mal_TransferResult_read(mal_call_t *call, mal_Address_t address, mal_USize_t index) {
    mal_Address_return(call, address);
    return mal_detail_memory_read_46708725c872772e(call, (const uint8_t *)address + (index * 16));
}

static inline void mal_TransferResult_write(mal_call_t *call, mal_Address_t address, mal_USize_t index, mal_TransferResult_t value) {
    mal_Address_return(call, address);
    mal_detail_memory_write_46708725c872772e(call, (uint8_t *)address + (index * 16), value);
}

static inline mal_SyscallStatus_t mal_SyscallStatus_read(mal_call_t *call, mal_Address_t address, mal_USize_t index) {
    mal_Address_return(call, address);
    return mal_detail_memory_read_4647c725c84fdeda(call, (const uint8_t *)address + (index * 8));
}

static inline void mal_SyscallStatus_write(mal_call_t *call, mal_Address_t address, mal_USize_t index, mal_SyscallStatus_t value) {
    mal_Address_return(call, address);
    mal_detail_memory_write_4647c725c84fdeda(call, (uint8_t *)address + (index * 8), value);
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
    return mal_detail_systemMmap(&call, (mal_MapRequest_t){ .field_0 = mal_detail_to_host_4647bb25c84fca76(&call, ((MalRepr_Product_fac85322fcb836ab){ .field_0 = argument_0, .field_1 = argument_1, .field_2 = argument_2, .field_3 = argument_3, .field_4 = argument_4, .field_5 = argument_5 }).field_0), .field_1 = ((MalRepr_Product_fac85322fcb836ab){ .field_0 = argument_0, .field_1 = argument_1, .field_2 = argument_2, .field_3 = argument_3, .field_4 = argument_4, .field_5 = argument_5 }).field_1, .field_2 = ((MalRepr_Product_fac85322fcb836ab){ .field_0 = argument_0, .field_1 = argument_1, .field_2 = argument_2, .field_3 = argument_3, .field_4 = argument_4, .field_5 = argument_5 }).field_2, .field_3 = ((MalRepr_Product_fac85322fcb836ab){ .field_0 = argument_0, .field_1 = argument_1, .field_2 = argument_2, .field_3 = argument_3, .field_4 = argument_4, .field_5 = argument_5 }).field_3, .field_4 = ((MalRepr_Product_fac85322fcb836ab){ .field_0 = argument_0, .field_1 = argument_1, .field_2 = argument_2, .field_3 = argument_3, .field_4 = argument_4, .field_5 = argument_5 }).field_4, .field_5 = ((MalRepr_Product_fac85322fcb836ab){ .field_0 = argument_0, .field_1 = argument_1, .field_2 = argument_2, .field_3 = argument_3, .field_4 = argument_4, .field_5 = argument_5 }).field_5 }); \
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
    return mal_detail_systemMremap(&call, (mal_ResizeRequest_t){ .field_0 = ((MalRepr_Product_1a78737f3c5897b2){ .field_0 = argument_0, .field_1 = argument_1, .field_2 = argument_2, .field_3 = argument_3 }).field_0, .field_1 = ((MalRepr_Product_1a78737f3c5897b2){ .field_0 = argument_0, .field_1 = argument_1, .field_2 = argument_2, .field_3 = argument_3 }).field_1, .field_2 = ((MalRepr_Product_1a78737f3c5897b2){ .field_0 = argument_0, .field_1 = argument_1, .field_2 = argument_2, .field_3 = argument_3 }).field_2, .field_3 = ((MalRepr_Product_1a78737f3c5897b2){ .field_0 = argument_0, .field_1 = argument_1, .field_2 = argument_2, .field_3 = argument_3 }).field_3 }); \
} \
static MalType_AddressResult mal_detail_systemMremap( \
    mal_call_t *call, \
    mal_ResizeRequest_t value \
)

#define MAL_HAS_EXTERN_systemMunmap 1
#define MAL_DEFINE_systemMunmap(call, value) \
static MalType_SyscallStatus mal_detail_systemMunmap(mal_call_t *call, mal_repr_product_1e5c21e9f35816c1_t value); \
MalType_SyscallStatus mal_ext_systemMunmap(MalContext *context MAL_DETAIL_MAYBE_UNUSED, MalType_Address argument_0, MalType_ByteSize argument_1) { \
    mal_call_t call = (mal_call_t){ .mal_detail_context = context }; \
    return mal_detail_systemMunmap(&call, (mal_repr_product_1e5c21e9f35816c1_t){ .field_0 = ((MalRepr_Product_1e5c21e9f35816c1){ .field_0 = argument_0, .field_1 = argument_1 }).field_0, .field_1 = ((MalRepr_Product_1e5c21e9f35816c1){ .field_0 = argument_0, .field_1 = argument_1 }).field_1 }); \
} \
static MalType_SyscallStatus mal_detail_systemMunmap( \
    mal_call_t *call, \
    mal_repr_product_1e5c21e9f35816c1_t value \
)

#define MAL_HAS_EXTERN_systemOpenat 1
#define MAL_DEFINE_systemOpenat(call, value) \
static MalType_FileDescriptorResult mal_detail_systemOpenat(mal_call_t *call, mal_repr_product_bb0fbcc1b817345c_t value); \
MalType_FileDescriptorResult mal_ext_systemOpenat(MalContext *context MAL_DETAIL_MAYBE_UNUSED, MalType_Int32 argument_0, MalType_Address argument_1, MalType_Int32 argument_2, MalType_UInt32 argument_3) { \
    mal_call_t call = (mal_call_t){ .mal_detail_context = context }; \
    return mal_detail_systemOpenat(&call, (mal_repr_product_bb0fbcc1b817345c_t){ .field_0 = ((MalRepr_Product_bb0fbcc1b817345c){ .field_0 = argument_0, .field_1 = argument_1, .field_2 = argument_2, .field_3 = argument_3 }).field_0, .field_1 = ((MalRepr_Product_bb0fbcc1b817345c){ .field_0 = argument_0, .field_1 = argument_1, .field_2 = argument_2, .field_3 = argument_3 }).field_1, .field_2 = ((MalRepr_Product_bb0fbcc1b817345c){ .field_0 = argument_0, .field_1 = argument_1, .field_2 = argument_2, .field_3 = argument_3 }).field_2, .field_3 = ((MalRepr_Product_bb0fbcc1b817345c){ .field_0 = argument_0, .field_1 = argument_1, .field_2 = argument_2, .field_3 = argument_3 }).field_3 }); \
} \
static MalType_FileDescriptorResult mal_detail_systemOpenat( \
    mal_call_t *call, \
    mal_repr_product_bb0fbcc1b817345c_t value \
)

#define MAL_HAS_EXTERN_systemLseek 1
#define MAL_DEFINE_systemLseek(call, value) \
static MalType_OffsetResult mal_detail_systemLseek(mal_call_t *call, mal_repr_product_03ed250e4033fba7_t value); \
MalType_OffsetResult mal_ext_systemLseek(MalContext *context MAL_DETAIL_MAYBE_UNUSED, MalType_Int32 argument_0, MalType_Int64 argument_1, MalType_Int32 argument_2) { \
    mal_call_t call = (mal_call_t){ .mal_detail_context = context }; \
    return mal_detail_systemLseek(&call, (mal_repr_product_03ed250e4033fba7_t){ .field_0 = ((MalRepr_Product_03ed250e4033fba7){ .field_0 = argument_0, .field_1 = argument_1, .field_2 = argument_2 }).field_0, .field_1 = ((MalRepr_Product_03ed250e4033fba7){ .field_0 = argument_0, .field_1 = argument_1, .field_2 = argument_2 }).field_1, .field_2 = ((MalRepr_Product_03ed250e4033fba7){ .field_0 = argument_0, .field_1 = argument_1, .field_2 = argument_2 }).field_2 }); \
} \
static MalType_OffsetResult mal_detail_systemLseek( \
    mal_call_t *call, \
    mal_repr_product_03ed250e4033fba7_t value \
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
static MalType_TransferResult mal_detail_systemRead(mal_call_t *call, mal_repr_product_bb3be5c1b83cb4f2_t value); \
MalType_TransferResult mal_ext_systemRead(MalContext *context MAL_DETAIL_MAYBE_UNUSED, MalType_Int32 argument_0, MalType_Address argument_1, MalType_ByteSize argument_2, MalType_ByteSize argument_3) { \
    mal_call_t call = (mal_call_t){ .mal_detail_context = context }; \
    return mal_detail_systemRead(&call, (mal_repr_product_bb3be5c1b83cb4f2_t){ .field_0 = ((MalRepr_Product_bb3be5c1b83cb4f2){ .field_0 = argument_0, .field_1 = argument_1, .field_2 = argument_2, .field_3 = argument_3 }).field_0, .field_1 = ((MalRepr_Product_bb3be5c1b83cb4f2){ .field_0 = argument_0, .field_1 = argument_1, .field_2 = argument_2, .field_3 = argument_3 }).field_1, .field_2 = ((MalRepr_Product_bb3be5c1b83cb4f2){ .field_0 = argument_0, .field_1 = argument_1, .field_2 = argument_2, .field_3 = argument_3 }).field_2, .field_3 = ((MalRepr_Product_bb3be5c1b83cb4f2){ .field_0 = argument_0, .field_1 = argument_1, .field_2 = argument_2, .field_3 = argument_3 }).field_3 }); \
} \
static MalType_TransferResult mal_detail_systemRead( \
    mal_call_t *call, \
    mal_repr_product_bb3be5c1b83cb4f2_t value \
)

#define MAL_HAS_EXTERN_systemWrite 1
#define MAL_DEFINE_systemWrite(call, value) \
static MalType_TransferResult mal_detail_systemWrite(mal_call_t *call, mal_repr_product_bb3be5c1b83cb4f2_t value); \
MalType_TransferResult mal_ext_systemWrite(MalContext *context MAL_DETAIL_MAYBE_UNUSED, MalType_Int32 argument_0, MalType_Address argument_1, MalType_ByteSize argument_2, MalType_ByteSize argument_3) { \
    mal_call_t call = (mal_call_t){ .mal_detail_context = context }; \
    return mal_detail_systemWrite(&call, (mal_repr_product_bb3be5c1b83cb4f2_t){ .field_0 = ((MalRepr_Product_bb3be5c1b83cb4f2){ .field_0 = argument_0, .field_1 = argument_1, .field_2 = argument_2, .field_3 = argument_3 }).field_0, .field_1 = ((MalRepr_Product_bb3be5c1b83cb4f2){ .field_0 = argument_0, .field_1 = argument_1, .field_2 = argument_2, .field_3 = argument_3 }).field_1, .field_2 = ((MalRepr_Product_bb3be5c1b83cb4f2){ .field_0 = argument_0, .field_1 = argument_1, .field_2 = argument_2, .field_3 = argument_3 }).field_2, .field_3 = ((MalRepr_Product_bb3be5c1b83cb4f2){ .field_0 = argument_0, .field_1 = argument_1, .field_2 = argument_2, .field_3 = argument_3 }).field_3 }); \
} \
static MalType_TransferResult mal_detail_systemWrite( \
    mal_call_t *call, \
    mal_repr_product_bb3be5c1b83cb4f2_t value \
)

#define MAL_HAS_EXTERN_storeZero 1
#define MAL_DEFINE_storeZero(call, value) \
static MalType_Unit mal_detail_storeZero(mal_call_t *call, mal_repr_product_1e5c21e9f35816c1_t value); \
void mal_ext_storeZero(MalContext *context MAL_DETAIL_MAYBE_UNUSED, MalType_Address argument_0, MalType_ByteSize argument_1) { \
    mal_call_t call = (mal_call_t){ .mal_detail_context = context }; \
    mal_detail_storeZero(&call, (mal_repr_product_1e5c21e9f35816c1_t){ .field_0 = ((MalRepr_Product_1e5c21e9f35816c1){ .field_0 = argument_0, .field_1 = argument_1 }).field_0, .field_1 = ((MalRepr_Product_1e5c21e9f35816c1){ .field_0 = argument_0, .field_1 = argument_1 }).field_1 }); \
} \
static MalType_Unit mal_detail_storeZero( \
    mal_call_t *call, \
    mal_repr_product_1e5c21e9f35816c1_t value \
)

#endif
#endif
