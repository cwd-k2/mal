use crate::check::ast::Type;

#[derive(Clone, Copy)]
pub(super) struct IntegerType {
    pub(super) index: usize,
    pub(super) name: &'static str,
    pub(super) c_type: &'static str,
    pub(super) unsigned: &'static str,
    pub(super) carrier: &'static str,
    pub(super) constant: &'static str,
    pub(super) minimum: Option<&'static str>,
    pub(super) minimum_value: Option<i128>,
    pub(super) width: u32,
    pub(super) maximum: &'static str,
}

impl IntegerType {
    pub(super) const fn mask(self) -> u16 {
        1 << self.index
    }

    pub(super) const fn signed(self) -> bool {
        self.minimum.is_some()
    }
}

pub(super) const INTEGER_TYPES: [IntegerType; 8] = [
    IntegerType {
        index: 0,
        name: "i8",
        c_type: "int8_t",
        unsigned: "uint8_t",
        carrier: "uint32_t",
        constant: "INT8_C",
        minimum: Some("INT8_MIN"),
        minimum_value: Some(i8::MIN as i128),
        width: 8,
        maximum: "UINT8_MAX",
    },
    IntegerType {
        index: 1,
        name: "i16",
        c_type: "int16_t",
        unsigned: "uint16_t",
        carrier: "uint32_t",
        constant: "INT16_C",
        minimum: Some("INT16_MIN"),
        minimum_value: Some(i16::MIN as i128),
        width: 16,
        maximum: "UINT16_MAX",
    },
    IntegerType {
        index: 2,
        name: "i32",
        c_type: "int32_t",
        unsigned: "uint32_t",
        carrier: "uint32_t",
        constant: "INT32_C",
        minimum: Some("INT32_MIN"),
        minimum_value: Some(i32::MIN as i128),
        width: 32,
        maximum: "UINT32_MAX",
    },
    IntegerType {
        index: 3,
        name: "i64",
        c_type: "int64_t",
        unsigned: "uint64_t",
        carrier: "uint64_t",
        constant: "INT64_C",
        minimum: Some("INT64_MIN"),
        minimum_value: Some(i64::MIN as i128),
        width: 64,
        maximum: "UINT64_MAX",
    },
    IntegerType {
        index: 4,
        name: "u8",
        c_type: "uint8_t",
        unsigned: "uint8_t",
        carrier: "uint32_t",
        constant: "UINT8_C",
        minimum: None,
        minimum_value: None,
        width: 8,
        maximum: "UINT8_MAX",
    },
    IntegerType {
        index: 5,
        name: "u16",
        c_type: "uint16_t",
        unsigned: "uint16_t",
        carrier: "uint32_t",
        constant: "UINT16_C",
        minimum: None,
        minimum_value: None,
        width: 16,
        maximum: "UINT16_MAX",
    },
    IntegerType {
        index: 6,
        name: "u32",
        c_type: "uint32_t",
        unsigned: "uint32_t",
        carrier: "uint32_t",
        constant: "UINT32_C",
        minimum: None,
        minimum_value: None,
        width: 32,
        maximum: "UINT32_MAX",
    },
    IntegerType {
        index: 7,
        name: "u64",
        c_type: "uint64_t",
        unsigned: "uint64_t",
        carrier: "uint64_t",
        constant: "UINT64_C",
        minimum: None,
        minimum_value: None,
        width: 64,
        maximum: "UINT64_MAX",
    },
];

pub(super) fn integer_type(ty: &Type) -> Option<&'static IntegerType> {
    let index = match ty {
        Type::Int8 => 0,
        Type::Int16 => 1,
        Type::Int32 => 2,
        Type::Int64 => 3,
        Type::UInt8 => 4,
        Type::UInt16 => 5,
        Type::UInt32 => 6,
        Type::UInt64 => 7,
        _ => return None,
    };
    Some(&INTEGER_TYPES[index])
}
