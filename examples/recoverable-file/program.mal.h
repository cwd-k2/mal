#ifndef MAL_BUILD_UMBRELLA
#include <mal.h>

_Static_assert(MAL_C_ABI_VERSION == 0x000900u, "generated header requires mal C ABI 0x000900");
_Static_assert((sizeof((size_t)0) * CHAR_BIT) == 64, "size_t does not match the mal target pointer index width");

#ifndef MAL_GENERATED_INTERFACE_965973C92DD3174E_H
#define MAL_GENERATED_INTERFACE_965973C92DD3174E_H
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
#define MAL_DETAIL_TO_RAW_bebfef0e190e1724 (MalRepr_Product_bebfef0e190e1724){ .field_0 = mal_Address_return(call, value.field_0), .field_1 = value.field_1, .field_2 = value.field_2 }
MAL_DETAIL_DEFINE_CONVERSION(mal_repr_product_bebfef0e190e1724_return, MalRepr_Product_bebfef0e190e1724, mal_repr_product_bebfef0e190e1724_t, MAL_DETAIL_TO_RAW_bebfef0e190e1724)
#endif

#ifndef MAL_DETAIL_HOST_REPR_c02233566497164f_HELPERS
#define MAL_DETAIL_HOST_REPR_c02233566497164f_HELPERS
#define MAL_DETAIL_TO_RAW_c02233566497164f (MalRepr_Product_c02233566497164f){ .field_0 = (MalType_Allocation){ .bits = value.field_0.mal_detail_bits }, .field_1 = (MalRepr_Product_bebfef0e190e1724){ .field_0 = mal_Address_return(call, value.field_1.field_0), .field_1 = value.field_1.field_1, .field_2 = value.field_1.field_2 } }
MAL_DETAIL_DEFINE_CONVERSION(mal_repr_product_c02233566497164f_return, MalRepr_Product_c02233566497164f, mal_repr_product_c02233566497164f_t, MAL_DETAIL_TO_RAW_c02233566497164f)
#endif

#ifndef MAL_DETAIL_HOST_REPR_1e5c20e9f358150e_HELPERS
#define MAL_DETAIL_HOST_REPR_1e5c20e9f358150e_HELPERS
#define MAL_DETAIL_TO_RAW_1e5c20e9f358150e (MalRepr_Product_1e5c20e9f358150e){ .field_0 = mal_Address_return(call, value.field_0), .field_1 = value.field_1 }
MAL_DETAIL_DEFINE_CONVERSION(mal_repr_product_1e5c20e9f358150e_return, MalRepr_Product_1e5c20e9f358150e, mal_repr_product_1e5c20e9f358150e_t, MAL_DETAIL_TO_RAW_1e5c20e9f358150e)
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
#define MAL_DETAIL_TO_RAW_be1b6939f45cf1f4 (MalRepr_Product_be1b6939f45cf1f4){ .field_0 = (MalType_File){ .bits = value.field_0.mal_detail_bits }, .field_1 = (MalRepr_Product_1e5c20e9f358150e){ .field_0 = mal_Address_return(call, value.field_1.field_0), .field_1 = value.field_1.field_1 } }
MAL_DETAIL_DEFINE_CONVERSION(mal_repr_product_be1b6939f45cf1f4_return, MalRepr_Product_be1b6939f45cf1f4, mal_repr_product_be1b6939f45cf1f4_t, MAL_DETAIL_TO_RAW_be1b6939f45cf1f4)
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

MAL_DETAIL_DEFINE_CONVERSION(mal_ByteBuffer_return, MalType_ByteBuffer, mal_ByteBuffer_t, MAL_DETAIL_TO_RAW_bebfef0e190e1724)
MAL_DETAIL_DEFINE_CONVERSION(mal_WritableBytes_return, MalType_WritableBytes, mal_WritableBytes_t, MAL_DETAIL_TO_RAW_1e5c20e9f358150e)
#define MAL_DETAIL_TO_RAW_ALIAS_IoError value
MAL_DETAIL_DEFINE_CONVERSION(mal_IoError_return, MalType_IoError, mal_IoError_t, MAL_DETAIL_TO_RAW_ALIAS_IoError)
MAL_DETAIL_DEFINE_CONVERSION(mal_OwnedBuffer_return, MalType_OwnedBuffer, mal_OwnedBuffer_t, MAL_DETAIL_TO_RAW_c02233566497164f)
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

#ifndef MAL_DETAIL_MEMORY_REPR_bebfef0e190e1724_HELPERS
#define MAL_DETAIL_MEMORY_REPR_bebfef0e190e1724_HELPERS
#define MAL_DETAIL_MEMORY_FIELDS_bebfef0e190e1724(unit, value) \
value(field_0, mal_detail_memory_read_Address, mal_detail_memory_write_Address, 0) \
value(field_1, mal_detail_memory_read_USize, mal_detail_memory_write_USize, 8) \
value(field_2, mal_detail_memory_read_USize, mal_detail_memory_write_USize, 16)
MAL_DETAIL_DEFINE_MEMORY_PRODUCT(mal_detail_memory_read_bebfef0e190e1724, mal_detail_memory_write_bebfef0e190e1724, mal_repr_product_bebfef0e190e1724_t, MAL_DETAIL_MEMORY_FIELDS_bebfef0e190e1724)
#endif

#ifndef MAL_DETAIL_MEMORY_REPR_1e5c20e9f358150e_HELPERS
#define MAL_DETAIL_MEMORY_REPR_1e5c20e9f358150e_HELPERS
#define MAL_DETAIL_MEMORY_FIELDS_1e5c20e9f358150e(unit, value) \
value(field_0, mal_detail_memory_read_Address, mal_detail_memory_write_Address, 0) \
value(field_1, mal_detail_memory_read_USize, mal_detail_memory_write_USize, 8)
MAL_DETAIL_DEFINE_MEMORY_PRODUCT(mal_detail_memory_read_1e5c20e9f358150e, mal_detail_memory_write_1e5c20e9f358150e, mal_repr_product_1e5c20e9f358150e_t, MAL_DETAIL_MEMORY_FIELDS_1e5c20e9f358150e)
#endif

#ifndef MAL_DETAIL_MEMORY_REPR_466d2725c86f9e37_HELPERS
#define MAL_DETAIL_MEMORY_REPR_466d2725c86f9e37_HELPERS
#define MAL_DETAIL_MEMORY_MEMBERS_466d2725c86f9e37(member) \
member(mal_repr_sum_466d2725c86f9e37_t, mal_UInt8_t, mal_detail_memory_write_UInt8, 0, variant_0, mal_detail_memory_read_USize, mal_detail_memory_write_USize, 8) \
member(mal_repr_sum_466d2725c86f9e37_t, mal_UInt8_t, mal_detail_memory_write_UInt8, 1, variant_1, mal_detail_memory_read_UInt32, mal_detail_memory_write_UInt32, 8)
MAL_DETAIL_DEFINE_MEMORY_SUM(mal_detail_memory_read_466d2725c86f9e37, mal_detail_memory_write_466d2725c86f9e37, mal_repr_sum_466d2725c86f9e37_t, mal_detail_memory_read_UInt8, MAL_DETAIL_MEMORY_MEMBERS_466d2725c86f9e37)
#endif

#ifndef MAL_DETAIL_MEMORY_REPR_4647c725c84fdeda_HELPERS
#define MAL_DETAIL_MEMORY_REPR_4647c725c84fdeda_HELPERS
#define MAL_DETAIL_MEMORY_MEMBERS_4647c725c84fdeda(member) \
member(mal_repr_sum_4647c725c84fdeda_t, mal_UInt8_t, mal_detail_memory_write_UInt8, 0, variant_0, mal_detail_memory_read_Unit, mal_detail_memory_write_Unit, 4) \
member(mal_repr_sum_4647c725c84fdeda_t, mal_UInt8_t, mal_detail_memory_write_UInt8, 1, variant_1, mal_detail_memory_read_UInt32, mal_detail_memory_write_UInt32, 4)
MAL_DETAIL_DEFINE_MEMORY_SUM(mal_detail_memory_read_4647c725c84fdeda, mal_detail_memory_write_4647c725c84fdeda, mal_repr_sum_4647c725c84fdeda_t, mal_detail_memory_read_UInt8, MAL_DETAIL_MEMORY_MEMBERS_4647c725c84fdeda)
#endif

MAL_DETAIL_DEFINE_MEMORY_ALIAS(mal_ByteBuffer_read, mal_ByteBuffer_write, mal_ByteBuffer_t, 24, mal_detail_memory_read_bebfef0e190e1724, mal_detail_memory_write_bebfef0e190e1724)

MAL_DETAIL_DEFINE_MEMORY_ALIAS(mal_WritableBytes_read, mal_WritableBytes_write, mal_WritableBytes_t, 16, mal_detail_memory_read_1e5c20e9f358150e, mal_detail_memory_write_1e5c20e9f358150e)

MAL_DETAIL_DEFINE_MEMORY_ALIAS(mal_IoError_read, mal_IoError_write, mal_IoError_t, 4, mal_detail_memory_read_UInt32, mal_detail_memory_write_UInt32)

MAL_DETAIL_DEFINE_MEMORY_ALIAS(mal_ReadResult_read, mal_ReadResult_write, mal_ReadResult_t, 16, mal_detail_memory_read_466d2725c86f9e37, mal_detail_memory_write_466d2725c86f9e37)

MAL_DETAIL_DEFINE_MEMORY_ALIAS(mal_CloseResult_read, mal_CloseResult_write, mal_CloseResult_t, 8, mal_detail_memory_read_4647c725c84fdeda, mal_detail_memory_write_4647c725c84fdeda)

MAL_DETAIL_DEFINE_MEMORY_ALIAS(mal_CopyResult_read, mal_CopyResult_write, mal_CopyResult_t, 8, mal_detail_memory_read_4647c725c84fdeda, mal_detail_memory_write_4647c725c84fdeda)

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
