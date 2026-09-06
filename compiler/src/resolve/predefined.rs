use super::ast::{TypeId, ValueId};

macro_rules! predefined {
    ($id_type:ident, $bindings:ident; $( $constant:ident = $id:literal => $name:literal ),+ $(,)?) => {
        $(pub const $constant: $id_type = $id_type($id);)+

        pub const $bindings: &[(&str, $id_type)] = &[
            $(($name, $constant),)+
        ];
    };
}

predefined!(TypeId, TYPES;
    UNIT_TYPE = 0 => "Unit",
    INT8_TYPE = 1 => "Int8",
    INT16_TYPE = 2 => "Int16",
    INT32_TYPE = 3 => "Int32",
    INT64_TYPE = 4 => "Int64",
    UINT8_TYPE = 5 => "UInt8",
    UINT16_TYPE = 6 => "UInt16",
    UINT32_TYPE = 7 => "UInt32",
    UINT64_TYPE = 8 => "UInt64",
    BOOL_TYPE = 9 => "Bool",
    SYMBOL_TYPE = 10 => "Symbol",
    FLOAT32_TYPE = 11 => "Float32",
    FLOAT64_TYPE = 12 => "Float64",
    PTR_TYPE = 13 => "Ptr",
);

predefined!(ValueId, VALUES;
    FALSE_VALUE = 0 => "false",
    TRUE_VALUE = 1 => "true",
    LOAD_INT64_VALUE = 2 => "loadInt64",
    STORE_INT64_VALUE = 3 => "storeInt64",
    LOAD_UINT8_VALUE = 4 => "loadUInt8",
    STORE_UINT8_VALUE = 5 => "storeUInt8",
    LOAD_INT8_VALUE = 6 => "loadInt8",
    STORE_INT8_VALUE = 7 => "storeInt8",
    LOAD_INT16_VALUE = 8 => "loadInt16",
    STORE_INT16_VALUE = 9 => "storeInt16",
    LOAD_INT32_VALUE = 10 => "loadInt32",
    STORE_INT32_VALUE = 11 => "storeInt32",
    LOAD_UINT16_VALUE = 12 => "loadUInt16",
    STORE_UINT16_VALUE = 13 => "storeUInt16",
    LOAD_UINT32_VALUE = 14 => "loadUInt32",
    STORE_UINT32_VALUE = 15 => "storeUInt32",
    LOAD_UINT64_VALUE = 16 => "loadUInt64",
    STORE_UINT64_VALUE = 17 => "storeUInt64",
    LOAD_FLOAT32_VALUE = 18 => "loadFloat32",
    STORE_FLOAT32_VALUE = 19 => "storeFloat32",
    LOAD_FLOAT64_VALUE = 20 => "loadFloat64",
    STORE_FLOAT64_VALUE = 21 => "storeFloat64",
    LOAD_PTR_VALUE = 22 => "loadPtr",
    STORE_PTR_VALUE = 23 => "storePtr",
    LOAD_SYMBOL_VALUE = 24 => "loadSymbol",
    STORE_SYMBOL_VALUE = 25 => "storeSymbol",
);

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
        assert_dense(TYPES.iter().map(|(_, id)| id.0));
        assert_dense(VALUES.iter().map(|(_, id)| id.0));
    }

    fn assert_unique(entries: impl Iterator<Item = (&'static str, u32)>) {
        let mut names = HashSet::new();
        let mut ids = HashSet::new();
        for (name, id) in entries {
            assert!(names.insert(name), "duplicate predefined name `{name}`");
            assert!(ids.insert(id), "duplicate predefined ID {id}");
        }
    }

    fn assert_dense(ids: impl Iterator<Item = u32>) {
        let mut ids = ids.collect::<Vec<_>>();
        ids.sort_unstable();
        assert_eq!(ids, (0..ids.len() as u32).collect::<Vec<_>>());
    }
}
