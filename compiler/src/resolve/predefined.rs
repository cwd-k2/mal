use super::ast::{
    BOOL_TYPE, FALSE_VALUE, FLOAT32_TYPE, FLOAT64_TYPE, INT8_TYPE, INT16_TYPE, INT32_TYPE,
    INT64_TYPE, LOAD_FLOAT32_VALUE, LOAD_FLOAT64_VALUE, LOAD_INT8_VALUE, LOAD_INT16_VALUE,
    LOAD_INT32_VALUE, LOAD_INT64_VALUE, LOAD_PTR_VALUE, LOAD_SYMBOL_VALUE, LOAD_UINT8_VALUE,
    LOAD_UINT16_VALUE, LOAD_UINT32_VALUE, LOAD_UINT64_VALUE, PTR_TYPE, STORE_FLOAT32_VALUE,
    STORE_FLOAT64_VALUE, STORE_INT8_VALUE, STORE_INT16_VALUE, STORE_INT32_VALUE, STORE_INT64_VALUE,
    STORE_PTR_VALUE, STORE_SYMBOL_VALUE, STORE_UINT8_VALUE, STORE_UINT16_VALUE, STORE_UINT32_VALUE,
    STORE_UINT64_VALUE, SYMBOL_TYPE, TRUE_VALUE, TypeId, UINT8_TYPE, UINT16_TYPE, UINT32_TYPE,
    UINT64_TYPE, UNIT_TYPE, ValueId,
};

pub const TYPES: &[(&str, TypeId)] = &[
    ("Unit", UNIT_TYPE),
    ("Int8", INT8_TYPE),
    ("Int16", INT16_TYPE),
    ("Int32", INT32_TYPE),
    ("Int64", INT64_TYPE),
    ("UInt8", UINT8_TYPE),
    ("UInt16", UINT16_TYPE),
    ("UInt32", UINT32_TYPE),
    ("UInt64", UINT64_TYPE),
    ("Bool", BOOL_TYPE),
    ("Symbol", SYMBOL_TYPE),
    ("Float32", FLOAT32_TYPE),
    ("Float64", FLOAT64_TYPE),
    ("Ptr", PTR_TYPE),
];

pub const VALUES: &[(&str, ValueId)] = &[
    ("false", FALSE_VALUE),
    ("true", TRUE_VALUE),
    ("loadInt64", LOAD_INT64_VALUE),
    ("storeInt64", STORE_INT64_VALUE),
    ("loadUInt8", LOAD_UINT8_VALUE),
    ("storeUInt8", STORE_UINT8_VALUE),
    ("loadInt8", LOAD_INT8_VALUE),
    ("storeInt8", STORE_INT8_VALUE),
    ("loadInt16", LOAD_INT16_VALUE),
    ("storeInt16", STORE_INT16_VALUE),
    ("loadInt32", LOAD_INT32_VALUE),
    ("storeInt32", STORE_INT32_VALUE),
    ("loadUInt16", LOAD_UINT16_VALUE),
    ("storeUInt16", STORE_UINT16_VALUE),
    ("loadUInt32", LOAD_UINT32_VALUE),
    ("storeUInt32", STORE_UINT32_VALUE),
    ("loadUInt64", LOAD_UINT64_VALUE),
    ("storeUInt64", STORE_UINT64_VALUE),
    ("loadFloat32", LOAD_FLOAT32_VALUE),
    ("storeFloat32", STORE_FLOAT32_VALUE),
    ("loadFloat64", LOAD_FLOAT64_VALUE),
    ("storeFloat64", STORE_FLOAT64_VALUE),
    ("loadPtr", LOAD_PTR_VALUE),
    ("storePtr", STORE_PTR_VALUE),
    ("loadSymbol", LOAD_SYMBOL_VALUE),
    ("storeSymbol", STORE_SYMBOL_VALUE),
];

pub fn first_source_type_id() -> u32 {
    TYPES
        .iter()
        .map(|(_, id)| id.0)
        .max()
        .map_or(0, |id| id + 1)
}

pub fn first_source_value_id() -> u32 {
    VALUES
        .iter()
        .map(|(_, id)| id.0)
        .max()
        .map_or(0, |id| id + 1)
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::*;

    #[test]
    fn predefined_names_and_ids_are_unique() {
        assert_unique(TYPES.iter().map(|(name, id)| (*name, id.0)));
        assert_unique(VALUES.iter().map(|(name, id)| (*name, id.0)));
    }

    fn assert_unique(entries: impl Iterator<Item = (&'static str, u32)>) {
        let mut names = HashSet::new();
        let mut ids = HashSet::new();
        for (name, id) in entries {
            assert!(names.insert(name), "duplicate predefined name `{name}`");
            assert!(ids.insert(id), "duplicate predefined ID {id}");
        }
    }
}
