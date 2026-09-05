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
    offset: bool,
    loads: u16,
    stores: u16,
    load_ptr: bool,
    store_ptr: bool,
    load_string: bool,
    store_string: bool,
) -> String {
    let mut output = String::new();
    if offset {
        c_line!(
            &mut output,
            0,
            "static inline MalPtr mal_ptr_offset(MalContext *context, MalPtr pointer, uint64_t offset) {{"
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
            "return (MalPtr){{ pointer.address + (size_t)offset }};"
        );
        c_line!(&mut output, 0, "}}\n");
    }
    for (index, (_, name, c_type)) in SCALARS.iter().enumerate() {
        let mask = 1 << index;
        if loads & mask != 0 {
            c_line!(
                &mut output,
                0,
                "static inline {c_type} mal_load_{name}(MalPtr pointer) {{"
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
                "static inline MalUnit mal_store_{name}(MalPtr pointer, {c_type} value) {{"
            );
            c_line!(
                &mut output,
                1,
                "memcpy(pointer.address, &value, sizeof(value));"
            );
            c_line!(&mut output, 1, "return (MalUnit){{ UINT8_C(0) }};");
            c_line!(&mut output, 0, "}}\n");
        }
    }
    if load_ptr {
        c_line!(
            &mut output,
            0,
            "static inline MalPtr mal_load_ptr(MalPtr pointer) {{"
        );
        c_line!(&mut output, 1, "MalPtr value;");
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
            "static inline MalUnit mal_store_ptr(MalPtr pointer, MalPtr value) {{"
        );
        c_line!(
            &mut output,
            1,
            "memcpy(pointer.address, &value, sizeof(value));"
        );
        c_line!(&mut output, 1, "return (MalUnit){{ UINT8_C(0) }};");
        c_line!(&mut output, 0, "}}\n");
    }
    if load_string {
        c_line!(
            &mut output,
            0,
            "static inline MalString mal_load_string(MalPtr pointer) {{"
        );
        c_line!(&mut output, 1, "MalPtr data;");
        c_line!(&mut output, 1, "uint64_t length;");
        c_line!(
            &mut output,
            1,
            "memcpy(&data, pointer.address, sizeof(data));"
        );
        c_line!(
            &mut output,
            1,
            "memcpy(&length, pointer.address + sizeof(data), sizeof(length));"
        );
        c_line!(
            &mut output,
            1,
            "return (MalString){{ (const uint8_t *)data.address, length }};"
        );
        c_line!(&mut output, 0, "}}\n");
    }
    if store_string {
        c_line!(
            &mut output,
            0,
            "static inline MalUnit mal_store_string(MalPtr pointer, MalString value) {{"
        );
        c_line!(&mut output, 1, "MalPtr data = {{ (uint8_t *)value.data }};");
        c_line!(
            &mut output,
            1,
            "memcpy(pointer.address, &data, sizeof(data));"
        );
        c_line!(
            &mut output,
            1,
            "memcpy(pointer.address + sizeof(data), &value.length, sizeof(value.length));"
        );
        c_line!(&mut output, 1, "return (MalUnit){{ UINT8_C(0) }};");
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
