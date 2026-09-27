#ifndef MAL_BUILD_UMBRELLA
#include <mal.h>
#include <string.h>

_Static_assert(MAL_C_ABI_VERSION == 0x000900u, "generated header requires mal C ABI 0x000900");
_Static_assert((sizeof((size_t)0) * CHAR_BIT) == 64, "size_t does not match the mal target pointer index width");

#ifndef MAL_GENERATED_INTERFACE_BCE378523B8417B4_H
#define MAL_GENERATED_INTERFACE_BCE378523B8417B4_H
/* Host-visible types */

typedef struct { uintptr_t bits; } MalType_Allocation;
typedef struct { uintptr_t bits; } MalType_File;

#ifndef MAL_DETAIL_RAW_REPR_bebfef0e190e1724_DECLARED
#define MAL_DETAIL_RAW_REPR_bebfef0e190e1724_DECLARED
typedef struct MalRepr_Product_bebfef0e190e1724 MalRepr_Product_bebfef0e190e1724;
#endif
#ifndef MAL_DETAIL_RAW_REPR_c02233566497164f_DECLARED
#define MAL_DETAIL_RAW_REPR_c02233566497164f_DECLARED
typedef struct MalRepr_Product_c02233566497164f MalRepr_Product_c02233566497164f;
#endif
#ifndef MAL_DETAIL_RAW_REPR_1e5c20e9f358150e_DECLARED
#define MAL_DETAIL_RAW_REPR_1e5c20e9f358150e_DECLARED
typedef struct MalRepr_Product_1e5c20e9f358150e MalRepr_Product_1e5c20e9f358150e;
#endif
#ifndef MAL_DETAIL_RAW_REPR_24e3218f7b4d917f_DECLARED
#define MAL_DETAIL_RAW_REPR_24e3218f7b4d917f_DECLARED
typedef struct MalRepr_Sum_24e3218f7b4d917f MalRepr_Sum_24e3218f7b4d917f;
#endif
#ifndef MAL_DETAIL_RAW_REPR_be1b6939f45cf1f4_DECLARED
#define MAL_DETAIL_RAW_REPR_be1b6939f45cf1f4_DECLARED
typedef struct MalRepr_Product_be1b6939f45cf1f4 MalRepr_Product_be1b6939f45cf1f4;
#endif
#ifndef MAL_DETAIL_RAW_REPR_466d2725c86f9e37_DECLARED
#define MAL_DETAIL_RAW_REPR_466d2725c86f9e37_DECLARED
typedef struct MalRepr_Sum_466d2725c86f9e37 MalRepr_Sum_466d2725c86f9e37;
#endif
#ifndef MAL_DETAIL_RAW_REPR_4647c725c84fdeda_DECLARED
#define MAL_DETAIL_RAW_REPR_4647c725c84fdeda_DECLARED
typedef struct MalRepr_Sum_4647c725c84fdeda MalRepr_Sum_4647c725c84fdeda;
#endif

#ifndef MAL_DETAIL_RAW_REPR_bebfef0e190e1724_DEFINED
#define MAL_DETAIL_RAW_REPR_bebfef0e190e1724_DEFINED
struct MalRepr_Product_bebfef0e190e1724 {
    MalType_Address field_0;
    MalType_USize field_1;
    MalType_USize field_2;
};

#endif

#ifndef MAL_DETAIL_RAW_REPR_c02233566497164f_DEFINED
#define MAL_DETAIL_RAW_REPR_c02233566497164f_DEFINED
struct MalRepr_Product_c02233566497164f {
    MalType_Allocation field_0;
    MalRepr_Product_bebfef0e190e1724 field_1;
};

#endif

#ifndef MAL_DETAIL_RAW_REPR_1e5c20e9f358150e_DEFINED
#define MAL_DETAIL_RAW_REPR_1e5c20e9f358150e_DEFINED
struct MalRepr_Product_1e5c20e9f358150e {
    MalType_Address field_0;
    MalType_USize field_1;
};

#endif

#ifndef MAL_DETAIL_RAW_REPR_24e3218f7b4d917f_DEFINED
#define MAL_DETAIL_RAW_REPR_24e3218f7b4d917f_DEFINED
struct MalRepr_Sum_24e3218f7b4d917f {
    uint32_t tag;
    union {
        MalType_File variant_0;
        MalType_UInt32 variant_1;
    } payload;
};

#endif

#ifndef MAL_DETAIL_RAW_REPR_be1b6939f45cf1f4_DEFINED
#define MAL_DETAIL_RAW_REPR_be1b6939f45cf1f4_DEFINED
struct MalRepr_Product_be1b6939f45cf1f4 {
    MalType_File field_0;
    MalRepr_Product_1e5c20e9f358150e field_1;
};

#endif

#ifndef MAL_DETAIL_RAW_REPR_466d2725c86f9e37_DEFINED
#define MAL_DETAIL_RAW_REPR_466d2725c86f9e37_DEFINED
struct MalRepr_Sum_466d2725c86f9e37 {
    uint32_t tag;
    union {
        MalType_USize variant_0;
        MalType_UInt32 variant_1;
    } payload;
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

typedef MalRepr_Product_bebfef0e190e1724 MalType_ByteBuffer;
typedef MalRepr_Product_1e5c20e9f358150e MalType_WritableBytes;
typedef MalType_UInt32 MalType_IoError;
typedef MalRepr_Product_c02233566497164f MalType_OwnedBuffer;
typedef MalRepr_Sum_24e3218f7b4d917f MalType_OpenResult;
typedef MalRepr_Sum_466d2725c86f9e37 MalType_ReadResult;
typedef MalRepr_Sum_4647c725c84fdeda MalType_CloseResult;

typedef struct { uintptr_t mal_detail_bits; } mal_Allocation_t;
typedef struct { uintptr_t mal_detail_bits; } mal_File_t;
#ifndef MAL_DETAIL_HOST_REPR_bebfef0e190e1724_DECLARED
#define MAL_DETAIL_HOST_REPR_bebfef0e190e1724_DECLARED
typedef struct mal_detail_repr_product_bebfef0e190e1724 mal_repr_product_bebfef0e190e1724_t;
#endif
#ifndef MAL_DETAIL_HOST_REPR_c02233566497164f_DECLARED
#define MAL_DETAIL_HOST_REPR_c02233566497164f_DECLARED
typedef struct mal_detail_repr_product_c02233566497164f mal_repr_product_c02233566497164f_t;
#endif
#ifndef MAL_DETAIL_HOST_REPR_1e5c20e9f358150e_DECLARED
#define MAL_DETAIL_HOST_REPR_1e5c20e9f358150e_DECLARED
typedef struct mal_detail_repr_product_1e5c20e9f358150e mal_repr_product_1e5c20e9f358150e_t;
#endif
#ifndef MAL_DETAIL_HOST_REPR_24e3218f7b4d917f_DECLARED
#define MAL_DETAIL_HOST_REPR_24e3218f7b4d917f_DECLARED
typedef struct mal_detail_repr_sum_24e3218f7b4d917f mal_repr_sum_24e3218f7b4d917f_t;
#endif
#ifndef MAL_DETAIL_HOST_REPR_be1b6939f45cf1f4_DECLARED
#define MAL_DETAIL_HOST_REPR_be1b6939f45cf1f4_DECLARED
typedef struct mal_detail_repr_product_be1b6939f45cf1f4 mal_repr_product_be1b6939f45cf1f4_t;
#endif
#ifndef MAL_DETAIL_HOST_REPR_466d2725c86f9e37_DECLARED
#define MAL_DETAIL_HOST_REPR_466d2725c86f9e37_DECLARED
typedef struct mal_detail_repr_sum_466d2725c86f9e37 mal_repr_sum_466d2725c86f9e37_t;
#endif
#ifndef MAL_DETAIL_HOST_REPR_4647c725c84fdeda_DECLARED
#define MAL_DETAIL_HOST_REPR_4647c725c84fdeda_DECLARED
typedef struct mal_detail_repr_sum_4647c725c84fdeda mal_repr_sum_4647c725c84fdeda_t;
#endif
typedef mal_repr_product_bebfef0e190e1724_t mal_ByteBuffer_t;
typedef mal_repr_product_1e5c20e9f358150e_t mal_WritableBytes_t;
typedef mal_UInt32_t mal_IoError_t;
typedef mal_repr_product_c02233566497164f_t mal_OwnedBuffer_t;
typedef mal_repr_sum_24e3218f7b4d917f_t mal_OpenResult_t;
typedef mal_repr_sum_466d2725c86f9e37_t mal_ReadResult_t;
typedef mal_repr_sum_4647c725c84fdeda_t mal_CloseResult_t;
typedef mal_repr_sum_4647c725c84fdeda_t mal_CopyResult_t;

#ifndef MAL_DETAIL_HOST_REPR_bebfef0e190e1724_DEFINED
#define MAL_DETAIL_HOST_REPR_bebfef0e190e1724_DEFINED
struct mal_detail_repr_product_bebfef0e190e1724 {
    mal_Address_t field_0;
    mal_USize_t field_1;
    mal_USize_t field_2;
};
#endif

#ifndef MAL_DETAIL_HOST_REPR_c02233566497164f_DEFINED
#define MAL_DETAIL_HOST_REPR_c02233566497164f_DEFINED
struct mal_detail_repr_product_c02233566497164f {
    mal_Allocation_t field_0;
    mal_repr_product_bebfef0e190e1724_t field_1;
};
#endif

#ifndef MAL_DETAIL_HOST_REPR_1e5c20e9f358150e_DEFINED
#define MAL_DETAIL_HOST_REPR_1e5c20e9f358150e_DEFINED
struct mal_detail_repr_product_1e5c20e9f358150e {
    mal_Address_t field_0;
    mal_USize_t field_1;
};
#endif

#ifndef MAL_DETAIL_HOST_REPR_24e3218f7b4d917f_DEFINED
#define MAL_DETAIL_HOST_REPR_24e3218f7b4d917f_DEFINED
struct mal_detail_repr_sum_24e3218f7b4d917f {
    uint32_t tag;
    union {
        mal_File_t variant_0;
        mal_UInt32_t variant_1;
    } payload;
};
#endif

#ifndef MAL_DETAIL_HOST_REPR_be1b6939f45cf1f4_DEFINED
#define MAL_DETAIL_HOST_REPR_be1b6939f45cf1f4_DEFINED
struct mal_detail_repr_product_be1b6939f45cf1f4 {
    mal_File_t field_0;
    mal_repr_product_1e5c20e9f358150e_t field_1;
};
#endif

#ifndef MAL_DETAIL_HOST_REPR_466d2725c86f9e37_DEFINED
#define MAL_DETAIL_HOST_REPR_466d2725c86f9e37_DEFINED
struct mal_detail_repr_sum_466d2725c86f9e37 {
    uint32_t tag;
    union {
        mal_USize_t variant_0;
        mal_UInt32_t variant_1;
    } payload;
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

/* Type helpers */

#ifndef MAL_DETAIL_HOST_REPR_bebfef0e190e1724_HELPERS
#define MAL_DETAIL_HOST_REPR_bebfef0e190e1724_HELPERS
static inline MalRepr_Product_bebfef0e190e1724 mal_repr_product_bebfef0e190e1724_return(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, mal_repr_product_bebfef0e190e1724_t value) {
    return (MalRepr_Product_bebfef0e190e1724){ .field_0 = mal_Address_return(call, value.field_0), .field_1 = value.field_1, .field_2 = value.field_2 };
}

#endif

#ifndef MAL_DETAIL_HOST_REPR_c02233566497164f_HELPERS
#define MAL_DETAIL_HOST_REPR_c02233566497164f_HELPERS
static inline MalRepr_Product_c02233566497164f mal_repr_product_c02233566497164f_return(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, mal_repr_product_c02233566497164f_t value) {
    return (MalRepr_Product_c02233566497164f){ .field_0 = (MalType_Allocation){ .bits = value.field_0.mal_detail_bits }, .field_1 = (MalRepr_Product_bebfef0e190e1724){ .field_0 = mal_Address_return(call, value.field_1.field_0), .field_1 = value.field_1.field_1, .field_2 = value.field_1.field_2 } };
}

#endif

#ifndef MAL_DETAIL_HOST_REPR_1e5c20e9f358150e_HELPERS
#define MAL_DETAIL_HOST_REPR_1e5c20e9f358150e_HELPERS
static inline MalRepr_Product_1e5c20e9f358150e mal_repr_product_1e5c20e9f358150e_return(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, mal_repr_product_1e5c20e9f358150e_t value) {
    return (MalRepr_Product_1e5c20e9f358150e){ .field_0 = mal_Address_return(call, value.field_0), .field_1 = value.field_1 };
}

#endif

#ifndef MAL_DETAIL_HOST_REPR_24e3218f7b4d917f_HELPERS
#define MAL_DETAIL_HOST_REPR_24e3218f7b4d917f_HELPERS
static inline mal_repr_sum_24e3218f7b4d917f_t mal_detail_to_host_24e3218f7b4d917f(mal_call_t *call, MalRepr_Sum_24e3218f7b4d917f value) {
    switch (value.tag) {
        case UINT32_C(0): {
            return (mal_repr_sum_24e3218f7b4d917f_t){ .tag = UINT32_C(0), .payload.variant_0 = (mal_File_t){ .mal_detail_bits = value.payload.variant_0.bits } };
        }
        case UINT32_C(1): {
            return (mal_repr_sum_24e3218f7b4d917f_t){ .tag = UINT32_C(1), .payload.variant_1 = value.payload.variant_1 };
        }
        default: {
            mal_call_trap(call, "invalid sum tag");
        }
    }
}

static inline MalRepr_Sum_24e3218f7b4d917f mal_detail_to_raw_24e3218f7b4d917f(mal_call_t *call, mal_repr_sum_24e3218f7b4d917f_t value) {
    switch (value.tag) {
        case UINT32_C(0): {
            return (MalRepr_Sum_24e3218f7b4d917f){ .tag = UINT32_C(0), .payload.variant_0 = (MalType_File){ .bits = value.payload.variant_0.mal_detail_bits } };
        }
        case UINT32_C(1): {
            return (MalRepr_Sum_24e3218f7b4d917f){ .tag = UINT32_C(1), .payload.variant_1 = value.payload.variant_1 };
        }
        default: {
            mal_call_trap(call, "invalid sum tag");
        }
    }
}

#define mal_repr_sum_24e3218f7b4d917f_tag_0 UINT32_C(0)
static inline mal_repr_sum_24e3218f7b4d917f_t mal_repr_sum_24e3218f7b4d917f_make_0(mal_File_t value) {
    return (mal_repr_sum_24e3218f7b4d917f_t){ .tag = mal_repr_sum_24e3218f7b4d917f_tag_0, .payload.variant_0 = value };
}

static inline MalRepr_Sum_24e3218f7b4d917f mal_repr_sum_24e3218f7b4d917f_return_0(mal_call_t *call, mal_File_t value) {
    return mal_detail_to_raw_24e3218f7b4d917f(call, (mal_repr_sum_24e3218f7b4d917f_t){ .tag = mal_repr_sum_24e3218f7b4d917f_tag_0, .payload.variant_0 = value });
}

#define mal_repr_sum_24e3218f7b4d917f_tag_1 UINT32_C(1)
static inline mal_repr_sum_24e3218f7b4d917f_t mal_repr_sum_24e3218f7b4d917f_make_1(mal_UInt32_t value) {
    return (mal_repr_sum_24e3218f7b4d917f_t){ .tag = mal_repr_sum_24e3218f7b4d917f_tag_1, .payload.variant_1 = value };
}

static inline MalRepr_Sum_24e3218f7b4d917f mal_repr_sum_24e3218f7b4d917f_return_1(mal_call_t *call, mal_UInt32_t value) {
    return mal_detail_to_raw_24e3218f7b4d917f(call, (mal_repr_sum_24e3218f7b4d917f_t){ .tag = mal_repr_sum_24e3218f7b4d917f_tag_1, .payload.variant_1 = value });
}

#endif

#ifndef MAL_DETAIL_HOST_REPR_be1b6939f45cf1f4_HELPERS
#define MAL_DETAIL_HOST_REPR_be1b6939f45cf1f4_HELPERS
static inline MalRepr_Product_be1b6939f45cf1f4 mal_repr_product_be1b6939f45cf1f4_return(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, mal_repr_product_be1b6939f45cf1f4_t value) {
    return (MalRepr_Product_be1b6939f45cf1f4){ .field_0 = (MalType_File){ .bits = value.field_0.mal_detail_bits }, .field_1 = (MalRepr_Product_1e5c20e9f358150e){ .field_0 = mal_Address_return(call, value.field_1.field_0), .field_1 = value.field_1.field_1 } };
}

#endif

#ifndef MAL_DETAIL_HOST_REPR_466d2725c86f9e37_HELPERS
#define MAL_DETAIL_HOST_REPR_466d2725c86f9e37_HELPERS
static inline mal_repr_sum_466d2725c86f9e37_t mal_detail_to_host_466d2725c86f9e37(mal_call_t *call, MalRepr_Sum_466d2725c86f9e37 value) {
    switch (value.tag) {
        case UINT32_C(0): {
            return (mal_repr_sum_466d2725c86f9e37_t){ .tag = UINT32_C(0), .payload.variant_0 = value.payload.variant_0 };
        }
        case UINT32_C(1): {
            return (mal_repr_sum_466d2725c86f9e37_t){ .tag = UINT32_C(1), .payload.variant_1 = value.payload.variant_1 };
        }
        default: {
            mal_call_trap(call, "invalid sum tag");
        }
    }
}

static inline MalRepr_Sum_466d2725c86f9e37 mal_detail_to_raw_466d2725c86f9e37(mal_call_t *call, mal_repr_sum_466d2725c86f9e37_t value) {
    switch (value.tag) {
        case UINT32_C(0): {
            return (MalRepr_Sum_466d2725c86f9e37){ .tag = UINT32_C(0), .payload.variant_0 = value.payload.variant_0 };
        }
        case UINT32_C(1): {
            return (MalRepr_Sum_466d2725c86f9e37){ .tag = UINT32_C(1), .payload.variant_1 = value.payload.variant_1 };
        }
        default: {
            mal_call_trap(call, "invalid sum tag");
        }
    }
}

#define mal_repr_sum_466d2725c86f9e37_tag_0 UINT32_C(0)
static inline mal_repr_sum_466d2725c86f9e37_t mal_repr_sum_466d2725c86f9e37_make_0(mal_USize_t value) {
    return (mal_repr_sum_466d2725c86f9e37_t){ .tag = mal_repr_sum_466d2725c86f9e37_tag_0, .payload.variant_0 = value };
}

static inline MalRepr_Sum_466d2725c86f9e37 mal_repr_sum_466d2725c86f9e37_return_0(mal_call_t *call, mal_USize_t value) {
    return mal_detail_to_raw_466d2725c86f9e37(call, (mal_repr_sum_466d2725c86f9e37_t){ .tag = mal_repr_sum_466d2725c86f9e37_tag_0, .payload.variant_0 = value });
}

#define mal_repr_sum_466d2725c86f9e37_tag_1 UINT32_C(1)
static inline mal_repr_sum_466d2725c86f9e37_t mal_repr_sum_466d2725c86f9e37_make_1(mal_UInt32_t value) {
    return (mal_repr_sum_466d2725c86f9e37_t){ .tag = mal_repr_sum_466d2725c86f9e37_tag_1, .payload.variant_1 = value };
}

static inline MalRepr_Sum_466d2725c86f9e37 mal_repr_sum_466d2725c86f9e37_return_1(mal_call_t *call, mal_UInt32_t value) {
    return mal_detail_to_raw_466d2725c86f9e37(call, (mal_repr_sum_466d2725c86f9e37_t){ .tag = mal_repr_sum_466d2725c86f9e37_tag_1, .payload.variant_1 = value });
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

static inline mal_Allocation_t mal_Allocation_from_bits(uintptr_t bits) {
    return (mal_Allocation_t){ .mal_detail_bits = bits };
}

static inline uintptr_t mal_Allocation_to_bits(mal_Allocation_t value) {
    return value.mal_detail_bits;
}

static inline MalType_Allocation mal_Allocation_return(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, mal_Allocation_t value) {
    return (MalType_Allocation){ .bits = value.mal_detail_bits };
}

static inline mal_File_t mal_File_from_bits(uintptr_t bits) {
    return (mal_File_t){ .mal_detail_bits = bits };
}

static inline uintptr_t mal_File_to_bits(mal_File_t value) {
    return value.mal_detail_bits;
}

static inline MalType_File mal_File_return(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, mal_File_t value) {
    return (MalType_File){ .bits = value.mal_detail_bits };
}

static inline MalType_ByteBuffer mal_ByteBuffer_return(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, mal_ByteBuffer_t value) {
    return (MalRepr_Product_bebfef0e190e1724){ .field_0 = mal_Address_return(call, value.field_0), .field_1 = value.field_1, .field_2 = value.field_2 };
}

static inline MalType_WritableBytes mal_WritableBytes_return(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, mal_WritableBytes_t value) {
    return (MalRepr_Product_1e5c20e9f358150e){ .field_0 = mal_Address_return(call, value.field_0), .field_1 = value.field_1 };
}

static inline MalType_IoError mal_IoError_return(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, mal_IoError_t value) {
    return value;
}

static inline MalType_OwnedBuffer mal_OwnedBuffer_return(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, mal_OwnedBuffer_t value) {
    return (MalRepr_Product_c02233566497164f){ .field_0 = (MalType_Allocation){ .bits = value.field_0.mal_detail_bits }, .field_1 = (MalRepr_Product_bebfef0e190e1724){ .field_0 = mal_Address_return(call, value.field_1.field_0), .field_1 = value.field_1.field_1, .field_2 = value.field_1.field_2 } };
}

#define mal_OpenResult_tag_0 UINT32_C(0)
static inline mal_OpenResult_t mal_OpenResult_make_0(mal_File_t value) {
    return (mal_OpenResult_t){ .tag = mal_OpenResult_tag_0, .payload.variant_0 = value };
}

static inline MalType_OpenResult mal_OpenResult_return_0(mal_call_t *call, mal_File_t value) {
    return mal_detail_to_raw_24e3218f7b4d917f(call, (mal_OpenResult_t){ .tag = mal_OpenResult_tag_0, .payload.variant_0 = value });
}

#define mal_OpenResult_tag_1 UINT32_C(1)
static inline mal_OpenResult_t mal_OpenResult_make_1(mal_IoError_t value) {
    return (mal_OpenResult_t){ .tag = mal_OpenResult_tag_1, .payload.variant_1 = value };
}

static inline MalType_OpenResult mal_OpenResult_return_1(mal_call_t *call, mal_IoError_t value) {
    return mal_detail_to_raw_24e3218f7b4d917f(call, (mal_OpenResult_t){ .tag = mal_OpenResult_tag_1, .payload.variant_1 = value });
}

#define mal_ReadResult_tag_0 UINT32_C(0)
static inline mal_ReadResult_t mal_ReadResult_make_0(mal_USize_t value) {
    return (mal_ReadResult_t){ .tag = mal_ReadResult_tag_0, .payload.variant_0 = value };
}

static inline MalType_ReadResult mal_ReadResult_return_0(mal_call_t *call, mal_USize_t value) {
    return mal_detail_to_raw_466d2725c86f9e37(call, (mal_ReadResult_t){ .tag = mal_ReadResult_tag_0, .payload.variant_0 = value });
}

#define mal_ReadResult_tag_1 UINT32_C(1)
static inline mal_ReadResult_t mal_ReadResult_make_1(mal_IoError_t value) {
    return (mal_ReadResult_t){ .tag = mal_ReadResult_tag_1, .payload.variant_1 = value };
}

static inline MalType_ReadResult mal_ReadResult_return_1(mal_call_t *call, mal_IoError_t value) {
    return mal_detail_to_raw_466d2725c86f9e37(call, (mal_ReadResult_t){ .tag = mal_ReadResult_tag_1, .payload.variant_1 = value });
}

#define mal_CloseResult_tag_0 UINT32_C(0)
static inline mal_CloseResult_t mal_CloseResult_make_0(void) {
    return (mal_CloseResult_t){ .tag = mal_CloseResult_tag_0, .payload.variant_0 = (mal_Unit_t){ 0 } };
}

static inline MalType_CloseResult mal_CloseResult_return_0(mal_call_t *call) {
    return mal_detail_to_raw_4647c725c84fdeda(call, (mal_CloseResult_t){ .tag = mal_CloseResult_tag_0, .payload.variant_0 = (mal_Unit_t){ 0 } });
}

#define mal_CloseResult_tag_1 UINT32_C(1)
static inline mal_CloseResult_t mal_CloseResult_make_1(mal_IoError_t value) {
    return (mal_CloseResult_t){ .tag = mal_CloseResult_tag_1, .payload.variant_1 = value };
}

static inline MalType_CloseResult mal_CloseResult_return_1(mal_call_t *call, mal_IoError_t value) {
    return mal_detail_to_raw_4647c725c84fdeda(call, (mal_CloseResult_t){ .tag = mal_CloseResult_tag_1, .payload.variant_1 = value });
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

#ifndef MAL_DETAIL_MEMORY_REPR_bebfef0e190e1724_HELPERS
#define MAL_DETAIL_MEMORY_REPR_bebfef0e190e1724_HELPERS
static inline mal_repr_product_bebfef0e190e1724_t mal_detail_memory_read_bebfef0e190e1724(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, const uint8_t *source MAL_DETAIL_MAYBE_UNUSED) {
    mal_repr_product_bebfef0e190e1724_t value;
    value.field_0 = mal_detail_memory_read_Address(call, source + 0);
    value.field_1 = mal_detail_memory_read_USize(call, source + 8);
    value.field_2 = mal_detail_memory_read_USize(call, source + 16);
    return value;
}

static inline void mal_detail_memory_write_bebfef0e190e1724(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, uint8_t *destination MAL_DETAIL_MAYBE_UNUSED, mal_repr_product_bebfef0e190e1724_t value) {
    mal_detail_memory_write_Address(call, destination + 0, value.field_0);
    mal_detail_memory_write_USize(call, destination + 8, value.field_1);
    mal_detail_memory_write_USize(call, destination + 16, value.field_2);
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

#ifndef MAL_DETAIL_MEMORY_REPR_466d2725c86f9e37_HELPERS
#define MAL_DETAIL_MEMORY_REPR_466d2725c86f9e37_HELPERS
static inline mal_repr_sum_466d2725c86f9e37_t mal_detail_memory_read_466d2725c86f9e37(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, const uint8_t *source MAL_DETAIL_MAYBE_UNUSED) {
    switch (mal_detail_memory_read_UInt8(call, source)) {
        case 0: {
            return (mal_repr_sum_466d2725c86f9e37_t){ .tag = UINT32_C(0), .payload.variant_0 = mal_detail_memory_read_USize(call, source + 8) };
        }
        case 1: {
            return (mal_repr_sum_466d2725c86f9e37_t){ .tag = UINT32_C(1), .payload.variant_1 = mal_detail_memory_read_UInt32(call, source + 8) };
        }
        default: {
            mal_call_trap(call, "invalid canonical sum tag");
        }
    }
}

static inline void mal_detail_memory_write_466d2725c86f9e37(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, uint8_t *destination MAL_DETAIL_MAYBE_UNUSED, mal_repr_sum_466d2725c86f9e37_t value) {
    switch (value.tag) {
        case UINT32_C(0): {
            mal_detail_memory_write_UInt8(call, destination, (mal_UInt8_t)value.tag);
            mal_detail_memory_write_USize(call, destination + 8, value.payload.variant_0);
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

static inline mal_ByteBuffer_t mal_ByteBuffer_read(mal_call_t *call, mal_Address_t address, mal_USize_t index) {
    mal_Address_return(call, address);
    return mal_detail_memory_read_bebfef0e190e1724(call, (const uint8_t *)address + (index * 24));
}

static inline void mal_ByteBuffer_write(mal_call_t *call, mal_Address_t address, mal_USize_t index, mal_ByteBuffer_t value) {
    mal_Address_return(call, address);
    mal_detail_memory_write_bebfef0e190e1724(call, (uint8_t *)address + (index * 24), value);
}

static inline mal_WritableBytes_t mal_WritableBytes_read(mal_call_t *call, mal_Address_t address, mal_USize_t index) {
    mal_Address_return(call, address);
    return mal_detail_memory_read_1e5c20e9f358150e(call, (const uint8_t *)address + (index * 16));
}

static inline void mal_WritableBytes_write(mal_call_t *call, mal_Address_t address, mal_USize_t index, mal_WritableBytes_t value) {
    mal_Address_return(call, address);
    mal_detail_memory_write_1e5c20e9f358150e(call, (uint8_t *)address + (index * 16), value);
}

static inline mal_IoError_t mal_IoError_read(mal_call_t *call, mal_Address_t address, mal_USize_t index) {
    mal_Address_return(call, address);
    return mal_detail_memory_read_UInt32(call, (const uint8_t *)address + (index * 4));
}

static inline void mal_IoError_write(mal_call_t *call, mal_Address_t address, mal_USize_t index, mal_IoError_t value) {
    mal_Address_return(call, address);
    mal_detail_memory_write_UInt32(call, (uint8_t *)address + (index * 4), value);
}

static inline mal_ReadResult_t mal_ReadResult_read(mal_call_t *call, mal_Address_t address, mal_USize_t index) {
    mal_Address_return(call, address);
    return mal_detail_memory_read_466d2725c86f9e37(call, (const uint8_t *)address + (index * 16));
}

static inline void mal_ReadResult_write(mal_call_t *call, mal_Address_t address, mal_USize_t index, mal_ReadResult_t value) {
    mal_Address_return(call, address);
    mal_detail_memory_write_466d2725c86f9e37(call, (uint8_t *)address + (index * 16), value);
}

static inline mal_CloseResult_t mal_CloseResult_read(mal_call_t *call, mal_Address_t address, mal_USize_t index) {
    mal_Address_return(call, address);
    return mal_detail_memory_read_4647c725c84fdeda(call, (const uint8_t *)address + (index * 8));
}

static inline void mal_CloseResult_write(mal_call_t *call, mal_Address_t address, mal_USize_t index, mal_CloseResult_t value) {
    mal_Address_return(call, address);
    mal_detail_memory_write_4647c725c84fdeda(call, (uint8_t *)address + (index * 8), value);
}

static inline mal_CopyResult_t mal_CopyResult_read(mal_call_t *call, mal_Address_t address, mal_USize_t index) {
    mal_Address_return(call, address);
    return mal_detail_memory_read_4647c725c84fdeda(call, (const uint8_t *)address + (index * 8));
}

static inline void mal_CopyResult_write(mal_call_t *call, mal_Address_t address, mal_USize_t index, mal_CopyResult_t value) {
    mal_Address_return(call, address);
    mal_detail_memory_write_4647c725c84fdeda(call, (uint8_t *)address + (index * 8), value);
}

/* External operations */

MalType_OwnedBuffer mal_ext_allocateBuffer(MalContext *context, MalType_USize value);
void mal_ext_releaseBuffer(MalContext *context, MalType_Allocation value);
MalType_OpenResult mal_ext_openReadOnly(MalContext *context, MalType_Address argument_0, MalType_USize argument_1);
MalType_ReadResult mal_ext_readFile(MalContext *context, MalType_File argument_0, MalType_WritableBytes argument_1);
MalType_CloseResult mal_ext_closeFile(MalContext *context, MalType_File value);
void mal_ext_writeBytes(MalContext *context, MalType_Address argument_0, MalType_USize argument_1);
void mal_ext_writeError(MalContext *context, MalType_IoError value);

/* External definition helpers */

#define MAL_HAS_EXTERN_allocateBuffer 1
#define MAL_DEFINE_allocateBuffer(call, value) \
static MalType_OwnedBuffer mal_detail_allocateBuffer(mal_call_t *call, mal_USize_t value); \
MalType_OwnedBuffer mal_ext_allocateBuffer(MalContext *context MAL_DETAIL_MAYBE_UNUSED, MalType_USize value) { \
    mal_call_t call = (mal_call_t){ .mal_detail_context = context }; \
    return mal_detail_allocateBuffer(&call, value); \
} \
static MalType_OwnedBuffer mal_detail_allocateBuffer( \
    mal_call_t *call, \
    mal_USize_t value \
)

#define MAL_HAS_EXTERN_releaseBuffer 1
#define MAL_DEFINE_releaseBuffer(call, value) \
static MalType_Unit mal_detail_releaseBuffer(mal_call_t *call, mal_Allocation_t value); \
void mal_ext_releaseBuffer(MalContext *context MAL_DETAIL_MAYBE_UNUSED, MalType_Allocation value) { \
    mal_call_t call = (mal_call_t){ .mal_detail_context = context }; \
    mal_detail_releaseBuffer(&call, (mal_Allocation_t){ .mal_detail_bits = value.bits }); \
} \
static MalType_Unit mal_detail_releaseBuffer( \
    mal_call_t *call, \
    mal_Allocation_t value \
)

#define MAL_HAS_EXTERN_openReadOnly 1
#define MAL_DEFINE_openReadOnly(call, value) \
static MalType_OpenResult mal_detail_openReadOnly(mal_call_t *call, mal_repr_product_1e5c20e9f358150e_t value); \
MalType_OpenResult mal_ext_openReadOnly(MalContext *context MAL_DETAIL_MAYBE_UNUSED, MalType_Address argument_0, MalType_USize argument_1) { \
    mal_call_t call = (mal_call_t){ .mal_detail_context = context }; \
    return mal_detail_openReadOnly(&call, (mal_repr_product_1e5c20e9f358150e_t){ .field_0 = ((MalRepr_Product_1e5c20e9f358150e){ .field_0 = argument_0, .field_1 = argument_1 }).field_0, .field_1 = ((MalRepr_Product_1e5c20e9f358150e){ .field_0 = argument_0, .field_1 = argument_1 }).field_1 }); \
} \
static MalType_OpenResult mal_detail_openReadOnly( \
    mal_call_t *call, \
    mal_repr_product_1e5c20e9f358150e_t value \
)

#define MAL_HAS_EXTERN_readFile 1
#define MAL_DEFINE_readFile(call, value) \
static MalType_ReadResult mal_detail_readFile(mal_call_t *call, mal_repr_product_be1b6939f45cf1f4_t value); \
MalType_ReadResult mal_ext_readFile(MalContext *context MAL_DETAIL_MAYBE_UNUSED, MalType_File argument_0, MalType_WritableBytes argument_1) { \
    mal_call_t call = (mal_call_t){ .mal_detail_context = context }; \
    return mal_detail_readFile(&call, (mal_repr_product_be1b6939f45cf1f4_t){ .field_0 = (mal_File_t){ .mal_detail_bits = ((MalRepr_Product_be1b6939f45cf1f4){ .field_0 = argument_0, .field_1 = argument_1 }).field_0.bits }, .field_1 = (mal_repr_product_1e5c20e9f358150e_t){ .field_0 = ((MalRepr_Product_be1b6939f45cf1f4){ .field_0 = argument_0, .field_1 = argument_1 }).field_1.field_0, .field_1 = ((MalRepr_Product_be1b6939f45cf1f4){ .field_0 = argument_0, .field_1 = argument_1 }).field_1.field_1 } }); \
} \
static MalType_ReadResult mal_detail_readFile( \
    mal_call_t *call, \
    mal_repr_product_be1b6939f45cf1f4_t value \
)

#define MAL_HAS_EXTERN_closeFile 1
#define MAL_DEFINE_closeFile(call, value) \
static MalType_CloseResult mal_detail_closeFile(mal_call_t *call, mal_File_t value); \
MalType_CloseResult mal_ext_closeFile(MalContext *context MAL_DETAIL_MAYBE_UNUSED, MalType_File value) { \
    mal_call_t call = (mal_call_t){ .mal_detail_context = context }; \
    return mal_detail_closeFile(&call, (mal_File_t){ .mal_detail_bits = value.bits }); \
} \
static MalType_CloseResult mal_detail_closeFile( \
    mal_call_t *call, \
    mal_File_t value \
)

#define MAL_HAS_EXTERN_writeBytes 1
#define MAL_DEFINE_writeBytes(call, value) \
static MalType_Unit mal_detail_writeBytes(mal_call_t *call, mal_repr_product_1e5c20e9f358150e_t value); \
void mal_ext_writeBytes(MalContext *context MAL_DETAIL_MAYBE_UNUSED, MalType_Address argument_0, MalType_USize argument_1) { \
    mal_call_t call = (mal_call_t){ .mal_detail_context = context }; \
    mal_detail_writeBytes(&call, (mal_repr_product_1e5c20e9f358150e_t){ .field_0 = ((MalRepr_Product_1e5c20e9f358150e){ .field_0 = argument_0, .field_1 = argument_1 }).field_0, .field_1 = ((MalRepr_Product_1e5c20e9f358150e){ .field_0 = argument_0, .field_1 = argument_1 }).field_1 }); \
} \
static MalType_Unit mal_detail_writeBytes( \
    mal_call_t *call, \
    mal_repr_product_1e5c20e9f358150e_t value \
)

#define MAL_HAS_EXTERN_writeError 1
#define MAL_DEFINE_writeError(call, value) \
static MalType_Unit mal_detail_writeError(mal_call_t *call, mal_IoError_t value); \
void mal_ext_writeError(MalContext *context MAL_DETAIL_MAYBE_UNUSED, MalType_IoError value) { \
    mal_call_t call = (mal_call_t){ .mal_detail_context = context }; \
    mal_detail_writeError(&call, value); \
} \
static MalType_Unit mal_detail_writeError( \
    mal_call_t *call, \
    mal_IoError_t value \
)

#endif
#endif
