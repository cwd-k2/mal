#ifndef MAL_BUILD_UMBRELLA
#include <mal.h>

_Static_assert(MAL_C_ABI_VERSION == 0x000900u, "generated header requires mal C ABI 0x000900");
_Static_assert((sizeof((size_t)0) * CHAR_BIT) == 64, "size_t does not match the mal target pointer index width");

#ifndef MAL_GENERATED_INTERFACE_9B7CFD85EDD472B6_H
#define MAL_GENERATED_INTERFACE_9B7CFD85EDD472B6_H
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

#ifndef MAL_DETAIL_REPR_FIELDS_8ae85a9b6e39c816_DEFINED
#define MAL_DETAIL_REPR_FIELDS_8ae85a9b6e39c816_DEFINED
#define MAL_DETAIL_REPR_FIELDS_8ae85a9b6e39c816(field) \
field(field_0, MalType_Socket, mal_Socket_t) \
field(field_1, MalType_Socket, mal_Socket_t)
#endif
#ifndef MAL_DETAIL_RAW_REPR_8ae85a9b6e39c816_DEFINED
#define MAL_DETAIL_RAW_REPR_8ae85a9b6e39c816_DEFINED
MAL_DETAIL_DEFINE_PRODUCT_REPR(MalRepr_Product_8ae85a9b6e39c816, MAL_DETAIL_REPR_FIELDS_8ae85a9b6e39c816, MAL_DETAIL_RAW_REPR_FIELD)

#endif

#ifndef MAL_DETAIL_REPR_FIELDS_1e5c20e9f358150e_DEFINED
#define MAL_DETAIL_REPR_FIELDS_1e5c20e9f358150e_DEFINED
#define MAL_DETAIL_REPR_FIELDS_1e5c20e9f358150e(field) \
field(field_0, MalType_Address, mal_Address_t) \
field(field_1, MalType_USize, mal_USize_t)
#endif
#ifndef MAL_DETAIL_RAW_REPR_1e5c20e9f358150e_DEFINED
#define MAL_DETAIL_RAW_REPR_1e5c20e9f358150e_DEFINED
MAL_DETAIL_DEFINE_PRODUCT_REPR(MalRepr_Product_1e5c20e9f358150e, MAL_DETAIL_REPR_FIELDS_1e5c20e9f358150e, MAL_DETAIL_RAW_REPR_FIELD)

#endif

#ifndef MAL_DETAIL_REPR_FIELDS_f9c88caf7e8dddf4_DEFINED
#define MAL_DETAIL_REPR_FIELDS_f9c88caf7e8dddf4_DEFINED
#define MAL_DETAIL_REPR_FIELDS_f9c88caf7e8dddf4(field) \
field(field_0, MalType_Socket, mal_Socket_t) \
field(field_1, MalType_UInt64, mal_UInt64_t) \
field(field_2, MalType_Address, mal_Address_t) \
field(field_3, MalType_USize, mal_USize_t)
#endif
#ifndef MAL_DETAIL_RAW_REPR_f9c88caf7e8dddf4_DEFINED
#define MAL_DETAIL_RAW_REPR_f9c88caf7e8dddf4_DEFINED
MAL_DETAIL_DEFINE_PRODUCT_REPR(MalRepr_Product_f9c88caf7e8dddf4, MAL_DETAIL_REPR_FIELDS_f9c88caf7e8dddf4, MAL_DETAIL_RAW_REPR_FIELD)

#endif

#ifndef MAL_DETAIL_REPR_FIELDS_4647c725c84fdeda_DEFINED
#define MAL_DETAIL_REPR_FIELDS_4647c725c84fdeda_DEFINED
#define MAL_DETAIL_REPR_FIELDS_4647c725c84fdeda(field) \
field(variant_0, MalType_Unit, mal_Unit_t) \
field(variant_1, MalType_UInt32, mal_UInt32_t)
#endif
#ifndef MAL_DETAIL_RAW_REPR_4647c725c84fdeda_DEFINED
#define MAL_DETAIL_RAW_REPR_4647c725c84fdeda_DEFINED
MAL_DETAIL_DEFINE_SUM_REPR(MalRepr_Sum_4647c725c84fdeda, MAL_DETAIL_REPR_FIELDS_4647c725c84fdeda, MAL_DETAIL_RAW_REPR_FIELD)

#endif

#ifndef MAL_DETAIL_REPR_FIELDS_cb68c98eb0579ca1_DEFINED
#define MAL_DETAIL_REPR_FIELDS_cb68c98eb0579ca1_DEFINED
#define MAL_DETAIL_REPR_FIELDS_cb68c98eb0579ca1(field) \
field(field_0, MalType_Socket, mal_Socket_t) \
field(field_1, MalType_Address, mal_Address_t) \
field(field_2, MalType_USize, mal_USize_t)
#endif
#ifndef MAL_DETAIL_RAW_REPR_cb68c98eb0579ca1_DEFINED
#define MAL_DETAIL_RAW_REPR_cb68c98eb0579ca1_DEFINED
MAL_DETAIL_DEFINE_PRODUCT_REPR(MalRepr_Product_cb68c98eb0579ca1, MAL_DETAIL_REPR_FIELDS_cb68c98eb0579ca1, MAL_DETAIL_RAW_REPR_FIELD)

#endif

#ifndef MAL_DETAIL_REPR_FIELDS_1e5f80e9f35aee05_DEFINED
#define MAL_DETAIL_REPR_FIELDS_1e5f80e9f35aee05_DEFINED
#define MAL_DETAIL_REPR_FIELDS_1e5f80e9f35aee05(field) \
field(field_0, MalType_UInt64, mal_UInt64_t) \
field(field_1, MalType_USize, mal_USize_t)
#endif
#ifndef MAL_DETAIL_RAW_REPR_1e5f80e9f35aee05_DEFINED
#define MAL_DETAIL_RAW_REPR_1e5f80e9f35aee05_DEFINED
MAL_DETAIL_DEFINE_PRODUCT_REPR(MalRepr_Product_1e5f80e9f35aee05, MAL_DETAIL_REPR_FIELDS_1e5f80e9f35aee05, MAL_DETAIL_RAW_REPR_FIELD)

#endif

#ifndef MAL_DETAIL_REPR_FIELDS_df02f3d72e6af636_DEFINED
#define MAL_DETAIL_REPR_FIELDS_df02f3d72e6af636_DEFINED
#define MAL_DETAIL_REPR_FIELDS_df02f3d72e6af636(field) \
field(variant_0, MalRepr_Product_1e5f80e9f35aee05, mal_repr_product_1e5f80e9f35aee05_t) \
field(variant_1, MalType_UInt32, mal_UInt32_t)
#endif
#ifndef MAL_DETAIL_RAW_REPR_df02f3d72e6af636_DEFINED
#define MAL_DETAIL_RAW_REPR_df02f3d72e6af636_DEFINED
MAL_DETAIL_DEFINE_SUM_REPR(MalRepr_Sum_df02f3d72e6af636, MAL_DETAIL_REPR_FIELDS_df02f3d72e6af636, MAL_DETAIL_RAW_REPR_FIELD)

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
MAL_DETAIL_DEFINE_PRODUCT_REPR(mal_detail_repr_product_8ae85a9b6e39c816, MAL_DETAIL_REPR_FIELDS_8ae85a9b6e39c816, MAL_DETAIL_HOST_REPR_FIELD)
#endif

#ifndef MAL_DETAIL_HOST_REPR_1e5c20e9f358150e_DEFINED
#define MAL_DETAIL_HOST_REPR_1e5c20e9f358150e_DEFINED
MAL_DETAIL_DEFINE_PRODUCT_REPR(mal_detail_repr_product_1e5c20e9f358150e, MAL_DETAIL_REPR_FIELDS_1e5c20e9f358150e, MAL_DETAIL_HOST_REPR_FIELD)
#endif

#ifndef MAL_DETAIL_HOST_REPR_f9c88caf7e8dddf4_DEFINED
#define MAL_DETAIL_HOST_REPR_f9c88caf7e8dddf4_DEFINED
MAL_DETAIL_DEFINE_PRODUCT_REPR(mal_detail_repr_product_f9c88caf7e8dddf4, MAL_DETAIL_REPR_FIELDS_f9c88caf7e8dddf4, MAL_DETAIL_HOST_REPR_FIELD)
#endif

#ifndef MAL_DETAIL_HOST_REPR_4647c725c84fdeda_DEFINED
#define MAL_DETAIL_HOST_REPR_4647c725c84fdeda_DEFINED
MAL_DETAIL_DEFINE_SUM_REPR(mal_detail_repr_sum_4647c725c84fdeda, MAL_DETAIL_REPR_FIELDS_4647c725c84fdeda, MAL_DETAIL_HOST_REPR_FIELD)
#endif

#ifndef MAL_DETAIL_HOST_REPR_cb68c98eb0579ca1_DEFINED
#define MAL_DETAIL_HOST_REPR_cb68c98eb0579ca1_DEFINED
MAL_DETAIL_DEFINE_PRODUCT_REPR(mal_detail_repr_product_cb68c98eb0579ca1, MAL_DETAIL_REPR_FIELDS_cb68c98eb0579ca1, MAL_DETAIL_HOST_REPR_FIELD)
#endif

#ifndef MAL_DETAIL_HOST_REPR_1e5f80e9f35aee05_DEFINED
#define MAL_DETAIL_HOST_REPR_1e5f80e9f35aee05_DEFINED
MAL_DETAIL_DEFINE_PRODUCT_REPR(mal_detail_repr_product_1e5f80e9f35aee05, MAL_DETAIL_REPR_FIELDS_1e5f80e9f35aee05, MAL_DETAIL_HOST_REPR_FIELD)
#endif

#ifndef MAL_DETAIL_HOST_REPR_df02f3d72e6af636_DEFINED
#define MAL_DETAIL_HOST_REPR_df02f3d72e6af636_DEFINED
MAL_DETAIL_DEFINE_SUM_REPR(mal_detail_repr_sum_df02f3d72e6af636, MAL_DETAIL_REPR_FIELDS_df02f3d72e6af636, MAL_DETAIL_HOST_REPR_FIELD)
#endif

/* Type helpers */

static inline mal_Socket_t mal_detail_to_host_Socket(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, MalType_Socket value);
static inline MalType_Socket mal_Socket_return(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, mal_Socket_t value);

#ifndef MAL_DETAIL_HOST_REPR_8ae85a9b6e39c816_HELPERS
#define MAL_DETAIL_HOST_REPR_8ae85a9b6e39c816_HELPERS
#define MAL_DETAIL_TO_HOST_8ae85a9b6e39c816 (mal_repr_product_8ae85a9b6e39c816_t){ .field_0 = (mal_Socket_t){ .mal_detail_bits = value.field_0.bits }, .field_1 = (mal_Socket_t){ .mal_detail_bits = value.field_1.bits } }
MAL_DETAIL_DEFINE_CONVERSION(mal_detail_to_host_8ae85a9b6e39c816, mal_repr_product_8ae85a9b6e39c816_t, MalRepr_Product_8ae85a9b6e39c816, MAL_DETAIL_TO_HOST_8ae85a9b6e39c816)
#define MAL_DETAIL_TO_RAW_8ae85a9b6e39c816 (MalRepr_Product_8ae85a9b6e39c816){ .field_0 = (MalType_Socket){ .bits = value.field_0.mal_detail_bits }, .field_1 = (MalType_Socket){ .bits = value.field_1.mal_detail_bits } }
MAL_DETAIL_DEFINE_CONVERSION(mal_repr_product_8ae85a9b6e39c816_return, MalRepr_Product_8ae85a9b6e39c816, mal_repr_product_8ae85a9b6e39c816_t, MAL_DETAIL_TO_RAW_8ae85a9b6e39c816)
#endif

#ifndef MAL_DETAIL_HOST_REPR_1e5c20e9f358150e_HELPERS
#define MAL_DETAIL_HOST_REPR_1e5c20e9f358150e_HELPERS
#define MAL_DETAIL_TO_HOST_1e5c20e9f358150e (mal_repr_product_1e5c20e9f358150e_t){ .field_0 = value.field_0, .field_1 = value.field_1 }
MAL_DETAIL_DEFINE_CONVERSION(mal_detail_to_host_1e5c20e9f358150e, mal_repr_product_1e5c20e9f358150e_t, MalRepr_Product_1e5c20e9f358150e, MAL_DETAIL_TO_HOST_1e5c20e9f358150e)
#define MAL_DETAIL_TO_RAW_1e5c20e9f358150e (MalRepr_Product_1e5c20e9f358150e){ .field_0 = mal_Address_return(call, value.field_0), .field_1 = value.field_1 }
MAL_DETAIL_DEFINE_CONVERSION(mal_repr_product_1e5c20e9f358150e_return, MalRepr_Product_1e5c20e9f358150e, mal_repr_product_1e5c20e9f358150e_t, MAL_DETAIL_TO_RAW_1e5c20e9f358150e)
#endif

#ifndef MAL_DETAIL_HOST_REPR_f9c88caf7e8dddf4_HELPERS
#define MAL_DETAIL_HOST_REPR_f9c88caf7e8dddf4_HELPERS
#define MAL_DETAIL_TO_HOST_f9c88caf7e8dddf4 (mal_repr_product_f9c88caf7e8dddf4_t){ .field_0 = (mal_Socket_t){ .mal_detail_bits = value.field_0.bits }, .field_1 = value.field_1, .field_2 = value.field_2, .field_3 = value.field_3 }
MAL_DETAIL_DEFINE_CONVERSION(mal_detail_to_host_f9c88caf7e8dddf4, mal_repr_product_f9c88caf7e8dddf4_t, MalRepr_Product_f9c88caf7e8dddf4, MAL_DETAIL_TO_HOST_f9c88caf7e8dddf4)
#define MAL_DETAIL_TO_RAW_f9c88caf7e8dddf4 (MalRepr_Product_f9c88caf7e8dddf4){ .field_0 = (MalType_Socket){ .bits = value.field_0.mal_detail_bits }, .field_1 = value.field_1, .field_2 = mal_Address_return(call, value.field_2), .field_3 = value.field_3 }
MAL_DETAIL_DEFINE_CONVERSION(mal_repr_product_f9c88caf7e8dddf4_return, MalRepr_Product_f9c88caf7e8dddf4, mal_repr_product_f9c88caf7e8dddf4_t, MAL_DETAIL_TO_RAW_f9c88caf7e8dddf4)
#endif

#ifndef MAL_DETAIL_HOST_REPR_4647c725c84fdeda_HELPERS
#define MAL_DETAIL_HOST_REPR_4647c725c84fdeda_HELPERS
#define MAL_DETAIL_TO_HOST_MEMBERS_4647c725c84fdeda(case, result_type) \
case(result_type, 0, variant_0, mal_detail_convert_Unit) \
case(result_type, 1, variant_1, mal_UInt32_return)
#define MAL_DETAIL_TO_RAW_MEMBERS_4647c725c84fdeda(case, result_type) \
case(result_type, 0, variant_0, mal_detail_convert_Unit) \
case(result_type, 1, variant_1, mal_UInt32_return)
MAL_DETAIL_DEFINE_SUM_CONVERSIONS(mal_detail_to_host_4647c725c84fdeda, mal_detail_to_raw_4647c725c84fdeda, MalRepr_Sum_4647c725c84fdeda, mal_repr_sum_4647c725c84fdeda_t, MAL_DETAIL_TO_HOST_MEMBERS_4647c725c84fdeda, MAL_DETAIL_TO_RAW_MEMBERS_4647c725c84fdeda)

#define mal_repr_sum_4647c725c84fdeda_tag_0 UINT32_C(0)
#define mal_repr_sum_4647c725c84fdeda_tag_1 UINT32_C(1)
#define MAL_DETAIL_SUM_API_repr_sum_4647c725c84fdeda(unit, value) \
unit(mal_repr_sum_4647c725c84fdeda_make_0, mal_repr_sum_4647c725c84fdeda_return_0, mal_repr_sum_4647c725c84fdeda_t, MalRepr_Sum_4647c725c84fdeda, mal_repr_sum_4647c725c84fdeda_tag_0, variant_0, mal_detail_to_raw_4647c725c84fdeda) \
value(mal_repr_sum_4647c725c84fdeda_make_1, mal_repr_sum_4647c725c84fdeda_return_1, mal_repr_sum_4647c725c84fdeda_t, MalRepr_Sum_4647c725c84fdeda, mal_UInt32_t, mal_repr_sum_4647c725c84fdeda_tag_1, variant_1, mal_detail_to_raw_4647c725c84fdeda)
MAL_DETAIL_SUM_API_repr_sum_4647c725c84fdeda(MAL_DETAIL_DEFINE_SUM_UNIT_API, MAL_DETAIL_DEFINE_SUM_VALUE_API)
#endif

#ifndef MAL_DETAIL_HOST_REPR_cb68c98eb0579ca1_HELPERS
#define MAL_DETAIL_HOST_REPR_cb68c98eb0579ca1_HELPERS
#define MAL_DETAIL_TO_HOST_cb68c98eb0579ca1 (mal_repr_product_cb68c98eb0579ca1_t){ .field_0 = (mal_Socket_t){ .mal_detail_bits = value.field_0.bits }, .field_1 = value.field_1, .field_2 = value.field_2 }
MAL_DETAIL_DEFINE_CONVERSION(mal_detail_to_host_cb68c98eb0579ca1, mal_repr_product_cb68c98eb0579ca1_t, MalRepr_Product_cb68c98eb0579ca1, MAL_DETAIL_TO_HOST_cb68c98eb0579ca1)
#define MAL_DETAIL_TO_RAW_cb68c98eb0579ca1 (MalRepr_Product_cb68c98eb0579ca1){ .field_0 = (MalType_Socket){ .bits = value.field_0.mal_detail_bits }, .field_1 = mal_Address_return(call, value.field_1), .field_2 = value.field_2 }
MAL_DETAIL_DEFINE_CONVERSION(mal_repr_product_cb68c98eb0579ca1_return, MalRepr_Product_cb68c98eb0579ca1, mal_repr_product_cb68c98eb0579ca1_t, MAL_DETAIL_TO_RAW_cb68c98eb0579ca1)
#endif

#ifndef MAL_DETAIL_HOST_REPR_1e5f80e9f35aee05_HELPERS
#define MAL_DETAIL_HOST_REPR_1e5f80e9f35aee05_HELPERS
#define MAL_DETAIL_TO_HOST_1e5f80e9f35aee05 (mal_repr_product_1e5f80e9f35aee05_t){ .field_0 = value.field_0, .field_1 = value.field_1 }
MAL_DETAIL_DEFINE_CONVERSION(mal_detail_to_host_1e5f80e9f35aee05, mal_repr_product_1e5f80e9f35aee05_t, MalRepr_Product_1e5f80e9f35aee05, MAL_DETAIL_TO_HOST_1e5f80e9f35aee05)
#define MAL_DETAIL_TO_RAW_1e5f80e9f35aee05 (MalRepr_Product_1e5f80e9f35aee05){ .field_0 = value.field_0, .field_1 = value.field_1 }
MAL_DETAIL_DEFINE_CONVERSION(mal_repr_product_1e5f80e9f35aee05_return, MalRepr_Product_1e5f80e9f35aee05, mal_repr_product_1e5f80e9f35aee05_t, MAL_DETAIL_TO_RAW_1e5f80e9f35aee05)
#endif

#ifndef MAL_DETAIL_HOST_REPR_df02f3d72e6af636_HELPERS
#define MAL_DETAIL_HOST_REPR_df02f3d72e6af636_HELPERS
#define MAL_DETAIL_TO_HOST_MEMBERS_df02f3d72e6af636(case, result_type) \
case(result_type, 0, variant_0, mal_detail_to_host_1e5f80e9f35aee05) \
case(result_type, 1, variant_1, mal_UInt32_return)
#define MAL_DETAIL_TO_RAW_MEMBERS_df02f3d72e6af636(case, result_type) \
case(result_type, 0, variant_0, mal_repr_product_1e5f80e9f35aee05_return) \
case(result_type, 1, variant_1, mal_UInt32_return)
MAL_DETAIL_DEFINE_SUM_CONVERSIONS(mal_detail_to_host_df02f3d72e6af636, mal_detail_to_raw_df02f3d72e6af636, MalRepr_Sum_df02f3d72e6af636, mal_repr_sum_df02f3d72e6af636_t, MAL_DETAIL_TO_HOST_MEMBERS_df02f3d72e6af636, MAL_DETAIL_TO_RAW_MEMBERS_df02f3d72e6af636)

#define mal_repr_sum_df02f3d72e6af636_tag_0 UINT32_C(0)
#define mal_repr_sum_df02f3d72e6af636_tag_1 UINT32_C(1)
#define MAL_DETAIL_SUM_API_repr_sum_df02f3d72e6af636(unit, value) \
value(mal_repr_sum_df02f3d72e6af636_make_0, mal_repr_sum_df02f3d72e6af636_return_0, mal_repr_sum_df02f3d72e6af636_t, MalRepr_Sum_df02f3d72e6af636, mal_repr_product_1e5f80e9f35aee05_t, mal_repr_sum_df02f3d72e6af636_tag_0, variant_0, mal_detail_to_raw_df02f3d72e6af636) \
value(mal_repr_sum_df02f3d72e6af636_make_1, mal_repr_sum_df02f3d72e6af636_return_1, mal_repr_sum_df02f3d72e6af636_t, MalRepr_Sum_df02f3d72e6af636, mal_UInt32_t, mal_repr_sum_df02f3d72e6af636_tag_1, variant_1, mal_detail_to_raw_df02f3d72e6af636)
MAL_DETAIL_SUM_API_repr_sum_df02f3d72e6af636(MAL_DETAIL_DEFINE_SUM_UNIT_API, MAL_DETAIL_DEFINE_SUM_VALUE_API)
#endif

static inline mal_Socket_t mal_Socket_from_bits(uintptr_t bits) {
    return (mal_Socket_t){ .mal_detail_bits = bits };
}

static inline mal_Socket_t mal_detail_to_host_Socket(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, MalType_Socket value) {
    return mal_Socket_from_bits(value.bits);
}

static inline uintptr_t mal_Socket_to_bits(mal_Socket_t value) {
    return value.mal_detail_bits;
}

static inline MalType_Socket mal_Socket_return(mal_call_t *call MAL_DETAIL_MAYBE_UNUSED, mal_Socket_t value) {
    return (MalType_Socket){ .bits = value.mal_detail_bits };
}

MAL_DETAIL_DEFINE_CONVERSION(mal_SocketPair_return, MalType_SocketPair, mal_SocketPair_t, MAL_DETAIL_TO_RAW_8ae85a9b6e39c816)
#define mal_SocketStatus_tag_0 UINT32_C(0)
#define mal_SocketStatus_tag_1 UINT32_C(1)
#define MAL_DETAIL_SUM_API_SocketStatus(unit, value) \
unit(mal_SocketStatus_make_0, mal_SocketStatus_return_0, mal_SocketStatus_t, MalType_SocketStatus, mal_SocketStatus_tag_0, variant_0, mal_detail_to_raw_4647c725c84fdeda) \
value(mal_SocketStatus_make_1, mal_SocketStatus_return_1, mal_SocketStatus_t, MalType_SocketStatus, mal_UInt32_t, mal_SocketStatus_tag_1, variant_1, mal_detail_to_raw_4647c725c84fdeda)
MAL_DETAIL_SUM_API_SocketStatus(MAL_DETAIL_DEFINE_SUM_UNIT_API, MAL_DETAIL_DEFINE_SUM_VALUE_API)
MAL_DETAIL_DEFINE_CONVERSION(mal_ReceivedPacket_return, MalType_ReceivedPacket, mal_ReceivedPacket_t, MAL_DETAIL_TO_RAW_1e5f80e9f35aee05)
#define mal_ReceiveResult_tag_0 UINT32_C(0)
#define mal_ReceiveResult_tag_1 UINT32_C(1)
#define MAL_DETAIL_SUM_API_ReceiveResult(unit, value) \
value(mal_ReceiveResult_make_0, mal_ReceiveResult_return_0, mal_ReceiveResult_t, MalType_ReceiveResult, mal_ReceivedPacket_t, mal_ReceiveResult_tag_0, variant_0, mal_detail_to_raw_df02f3d72e6af636) \
value(mal_ReceiveResult_make_1, mal_ReceiveResult_return_1, mal_ReceiveResult_t, MalType_ReceiveResult, mal_UInt32_t, mal_ReceiveResult_tag_1, variant_1, mal_detail_to_raw_df02f3d72e6af636)
MAL_DETAIL_SUM_API_ReceiveResult(MAL_DETAIL_DEFINE_SUM_UNIT_API, MAL_DETAIL_DEFINE_SUM_VALUE_API)
MAL_DETAIL_DEFINE_CONVERSION(mal_PacketBuffer_return, MalType_PacketBuffer, mal_PacketBuffer_t, MAL_DETAIL_TO_RAW_1e5c20e9f358150e)

/* Canonical memory access */

#ifndef MAL_DETAIL_MEMORY_REPR_1e5c20e9f358150e_HELPERS
#define MAL_DETAIL_MEMORY_REPR_1e5c20e9f358150e_HELPERS
#define MAL_DETAIL_MEMORY_FIELDS_1e5c20e9f358150e(unit, value) \
value(field_0, mal_detail_memory_read_Address, mal_detail_memory_write_Address, 0) \
value(field_1, mal_detail_memory_read_USize, mal_detail_memory_write_USize, 8)
MAL_DETAIL_DEFINE_MEMORY_PRODUCT(mal_detail_memory_read_1e5c20e9f358150e, mal_detail_memory_write_1e5c20e9f358150e, mal_repr_product_1e5c20e9f358150e_t, MAL_DETAIL_MEMORY_FIELDS_1e5c20e9f358150e)
#endif

#ifndef MAL_DETAIL_MEMORY_REPR_4647c725c84fdeda_HELPERS
#define MAL_DETAIL_MEMORY_REPR_4647c725c84fdeda_HELPERS
#define MAL_DETAIL_MEMORY_MEMBERS_4647c725c84fdeda(member) \
member(mal_repr_sum_4647c725c84fdeda_t, mal_UInt8_t, mal_detail_memory_write_UInt8, 0, variant_0, mal_detail_memory_read_Unit, mal_detail_memory_write_Unit, 4) \
member(mal_repr_sum_4647c725c84fdeda_t, mal_UInt8_t, mal_detail_memory_write_UInt8, 1, variant_1, mal_detail_memory_read_UInt32, mal_detail_memory_write_UInt32, 4)
MAL_DETAIL_DEFINE_MEMORY_SUM(mal_detail_memory_read_4647c725c84fdeda, mal_detail_memory_write_4647c725c84fdeda, mal_repr_sum_4647c725c84fdeda_t, mal_detail_memory_read_UInt8, MAL_DETAIL_MEMORY_MEMBERS_4647c725c84fdeda)
#endif

#ifndef MAL_DETAIL_MEMORY_REPR_1e5f80e9f35aee05_HELPERS
#define MAL_DETAIL_MEMORY_REPR_1e5f80e9f35aee05_HELPERS
#define MAL_DETAIL_MEMORY_FIELDS_1e5f80e9f35aee05(unit, value) \
value(field_0, mal_detail_memory_read_UInt64, mal_detail_memory_write_UInt64, 0) \
value(field_1, mal_detail_memory_read_USize, mal_detail_memory_write_USize, 8)
MAL_DETAIL_DEFINE_MEMORY_PRODUCT(mal_detail_memory_read_1e5f80e9f35aee05, mal_detail_memory_write_1e5f80e9f35aee05, mal_repr_product_1e5f80e9f35aee05_t, MAL_DETAIL_MEMORY_FIELDS_1e5f80e9f35aee05)
#endif

#ifndef MAL_DETAIL_MEMORY_REPR_df02f3d72e6af636_HELPERS
#define MAL_DETAIL_MEMORY_REPR_df02f3d72e6af636_HELPERS
#define MAL_DETAIL_MEMORY_MEMBERS_df02f3d72e6af636(member) \
member(mal_repr_sum_df02f3d72e6af636_t, mal_UInt8_t, mal_detail_memory_write_UInt8, 0, variant_0, mal_detail_memory_read_1e5f80e9f35aee05, mal_detail_memory_write_1e5f80e9f35aee05, 8) \
member(mal_repr_sum_df02f3d72e6af636_t, mal_UInt8_t, mal_detail_memory_write_UInt8, 1, variant_1, mal_detail_memory_read_UInt32, mal_detail_memory_write_UInt32, 8)
MAL_DETAIL_DEFINE_MEMORY_SUM(mal_detail_memory_read_df02f3d72e6af636, mal_detail_memory_write_df02f3d72e6af636, mal_repr_sum_df02f3d72e6af636_t, mal_detail_memory_read_UInt8, MAL_DETAIL_MEMORY_MEMBERS_df02f3d72e6af636)
#endif

MAL_DETAIL_DEFINE_MEMORY_ALIAS(mal_SocketStatus_read, mal_SocketStatus_write, mal_SocketStatus_t, 8, mal_detail_memory_read_4647c725c84fdeda, mal_detail_memory_write_4647c725c84fdeda)

MAL_DETAIL_DEFINE_MEMORY_ALIAS(mal_ReceivedPacket_read, mal_ReceivedPacket_write, mal_ReceivedPacket_t, 16, mal_detail_memory_read_1e5f80e9f35aee05, mal_detail_memory_write_1e5f80e9f35aee05)

MAL_DETAIL_DEFINE_MEMORY_ALIAS(mal_ReceiveResult_read, mal_ReceiveResult_write, mal_ReceiveResult_t, 24, mal_detail_memory_read_df02f3d72e6af636, mal_detail_memory_write_df02f3d72e6af636)

MAL_DETAIL_DEFINE_MEMORY_ALIAS(mal_PacketBuffer_read, mal_PacketBuffer_write, mal_PacketBuffer_t, 16, mal_detail_memory_read_1e5c20e9f358150e, mal_detail_memory_write_1e5c20e9f358150e)

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
    return mal_detail_sendPacket(&call, mal_detail_to_host_f9c88caf7e8dddf4(&call, (MalRepr_Product_f9c88caf7e8dddf4){ .field_0 = argument_0, .field_1 = argument_1, .field_2 = argument_2, .field_3 = argument_3 })); \
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
    return mal_detail_receivePacket(&call, mal_detail_to_host_cb68c98eb0579ca1(&call, (MalRepr_Product_cb68c98eb0579ca1){ .field_0 = argument_0, .field_1 = argument_1, .field_2 = argument_2 })); \
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
