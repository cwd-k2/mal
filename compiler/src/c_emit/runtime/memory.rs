use crate::check::ast::MemoryScalar;

const SCALARS: [(MemoryScalar, &str, &str); 10] = [
    (MemoryScalar::Int8, "int8", "int8_t"),
    (MemoryScalar::Int16, "int16", "int16_t"),
    (MemoryScalar::Int32, "int32", "int32_t"),
    (MemoryScalar::Int64, "int64", "int64_t"),
    (MemoryScalar::UInt8, "uint8", "uint8_t"),
    (MemoryScalar::UInt16, "uint16", "uint16_t"),
    (MemoryScalar::UInt32, "uint32", "uint32_t"),
    (MemoryScalar::UInt64, "uint64", "uint64_t"),
    (MemoryScalar::Float32, "float32", "float"),
    (MemoryScalar::Float64, "float64", "double"),
];

pub(crate) fn scalar_mask(scalar: MemoryScalar) -> u16 {
    1 << scalar_index(scalar)
}

pub(crate) fn scalar_name(scalar: MemoryScalar) -> &'static str {
    SCALARS[scalar_index(scalar)].1
}

pub(super) fn emit(
    offsets: (bool, bool),
    loads: u16,
    stores: u16,
    load_ptr: bool,
    store_ptr: bool,
    load_symbol: bool,
    store_symbol: bool,
) -> String {
    let (offset_forward, offset_backward) = offsets;
    let mut output = String::new();
    if offset_forward {
        c_line!(
            &mut output,
            0,
            "static inline MalType_Ptr mal_ptr_offset(MalContext *context, MalType_Ptr pointer, uint64_t offset) {{"
        );
        c_line!(&mut output, 1, "if (offset > SIZE_MAX) {{");
        c_line!(
            &mut output,
            2,
            "mal_trap(context, \"pointer offset is not representable on this target\");"
        );
        c_line!(&mut output, 1, "}}");
        c_line!(
            &mut output,
            1,
            "return (MalType_Ptr){{ pointer.address + (size_t)offset }};"
        );
        c_line!(&mut output, 0, "}}\n");
    }
    if offset_backward {
        c_line!(
            &mut output,
            0,
            "static inline MalType_Ptr mal_ptr_offset_backward(MalContext *context, MalType_Ptr pointer, uint64_t offset) {{"
        );
        c_line!(&mut output, 1, "if (offset > SIZE_MAX) {{");
        c_line!(
            &mut output,
            2,
            "mal_trap(context, \"pointer offset is not representable on this target\");"
        );
        c_line!(&mut output, 1, "}}");
        c_line!(
            &mut output,
            1,
            "return (MalType_Ptr){{ pointer.address - (size_t)offset }};"
        );
        c_line!(&mut output, 0, "}}\n");
    }
    for (index, (_, name, c_type)) in SCALARS.iter().enumerate() {
        let mask = 1 << index;
        if loads & mask != 0 {
            c_line!(
                &mut output,
                0,
                "static inline {c_type} mal_load_{name}(MalType_Ptr pointer) {{"
            );
            c_line!(&mut output, 1, "{c_type} value;");
            c_line!(
                &mut output,
                1,
                "memcpy(&value, pointer.address, sizeof(value));"
            );
            c_line!(&mut output, 1, "return value;");
            c_line!(&mut output, 0, "}}\n");
        }
        if stores & mask != 0 {
            c_line!(
                &mut output,
                0,
                "static inline MalType_Unit mal_store_{name}(MalType_Ptr pointer, {c_type} value) {{"
            );
            c_line!(
                &mut output,
                1,
                "memcpy(pointer.address, &value, sizeof(value));"
            );
            c_line!(&mut output, 1, "return (MalType_Unit){{ UINT8_C(0) }};");
            c_line!(&mut output, 0, "}}\n");
        }
    }
    if load_ptr {
        c_line!(
            &mut output,
            0,
            "static inline MalType_Ptr mal_load_ptr(MalType_Ptr pointer) {{"
        );
        c_line!(&mut output, 1, "MalType_Ptr value;");
        c_line!(
            &mut output,
            1,
            "memcpy(&value, pointer.address, sizeof(value));"
        );
        c_line!(&mut output, 1, "return value;");
        c_line!(&mut output, 0, "}}\n");
    }
    if store_ptr {
        c_line!(
            &mut output,
            0,
            "static inline MalType_Unit mal_store_ptr(MalType_Ptr pointer, MalType_Ptr value) {{"
        );
        c_line!(
            &mut output,
            1,
            "memcpy(pointer.address, &value, sizeof(value));"
        );
        c_line!(&mut output, 1, "return (MalType_Unit){{ UINT8_C(0) }};");
        c_line!(&mut output, 0, "}}\n");
    }
    if load_symbol {
        c_line!(
            &mut output,
            0,
            "static inline MalType_Symbol mal_load_symbol(MalContext *context, MalType_Ptr pointer, uint64_t length) {{"
        );
        c_line!(
            &mut output,
            1,
            "return mal_Symbol_copy_from_bytes(context, pointer.address, length);"
        );
        c_line!(&mut output, 0, "}}\n");
    }
    if store_symbol {
        c_line!(
            &mut output,
            0,
            "static inline MalType_Unit mal_store_symbol(MalType_Ptr pointer, MalType_Symbol value) {{"
        );
        c_line!(&mut output, 1, "if (value.length != UINT64_C(0)) {{");
        c_line!(
            &mut output,
            2,
            "memcpy(pointer.address, value.data, (size_t)value.length);"
        );
        c_line!(&mut output, 1, "}}");
        c_line!(&mut output, 1, "return (MalType_Unit){{ UINT8_C(0) }};");
        c_line!(&mut output, 0, "}}\n");
    }
    output
}

fn scalar_index(scalar: MemoryScalar) -> usize {
    match scalar {
        MemoryScalar::Int8 => 0,
        MemoryScalar::Int16 => 1,
        MemoryScalar::Int32 => 2,
        MemoryScalar::Int64 => 3,
        MemoryScalar::UInt8 => 4,
        MemoryScalar::UInt16 => 5,
        MemoryScalar::UInt32 => 6,
        MemoryScalar::UInt64 => 7,
        MemoryScalar::Float32 => 8,
        MemoryScalar::Float64 => 9,
    }
}
