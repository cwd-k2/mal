use super::ast::{TypeId, ValueId};

#[derive(Clone, Copy)]
pub struct PredefinedType {
    pub name: &'static str,
    pub id: TypeId,
    pub detail: &'static str,
    pub documentation: &'static str,
}

#[derive(Clone, Copy)]
pub struct PredefinedValue {
    pub name: &'static str,
    pub id: ValueId,
    pub detail: Option<&'static str>,
    pub documentation: &'static str,
    pub callable: bool,
}

macro_rules! predefined_types {
    ($( $constant:ident = $id:literal => ($name:literal, $detail:literal, $documentation:literal) ),+ $(,)?) => {
        $(pub const $constant: TypeId = TypeId($id);)+

        pub const TYPES: &[PredefinedType] = &[
            $(PredefinedType {
                name: $name,
                id: $constant,
                detail: $detail,
                documentation: $documentation,
            },)+
        ];
    };
}

predefined_types!(
    UNIT_TYPE = 0 => ("Unit", "Unit", "The unit type. Its sole value is `()`, used when an expression carries no data."),
    INT8_TYPE = 1 => ("Int8", "Int8", "An 8-bit signed integer."),
    INT16_TYPE = 2 => ("Int16", "Int16", "A 16-bit signed integer."),
    INT32_TYPE = 3 => ("Int32", "Int32", "A 32-bit signed integer."),
    INT64_TYPE = 4 => ("Int64", "Int64", "A 64-bit signed integer."),
    UINT8_TYPE = 5 => ("UInt8", "UInt8", "An 8-bit unsigned integer. Byte literals have this type."),
    UINT16_TYPE = 6 => ("UInt16", "UInt16", "A 16-bit unsigned integer."),
    UINT32_TYPE = 7 => ("UInt32", "UInt32", "A 32-bit unsigned integer."),
    UINT64_TYPE = 8 => ("UInt64", "UInt64", "A 64-bit unsigned integer."),
    BOOL_TYPE = 9 => ("Bool", "Bool", "A Boolean value: either `true` or `false`."),
    SYMBOL_TYPE = 10 => ("Symbol", "Symbol", "An immutable byte string owned by mal. `*symbol` creates a mutable `Buffer<UInt8>` snapshot."),
    FLOAT32_TYPE = 11 => ("Float32", "Float32", "An IEEE 754 binary32 floating-point number."),
    FLOAT64_TYPE = 12 => ("Float64", "Float64", "An IEEE 754 binary64 floating-point number."),
    BYTE_SIZE_TYPE = 13 => ("ByteSize", "ByteSize", "A target-width unsigned quantity measured in bytes for host contracts."),
    U_SIZE_TYPE = 14 => ("USize", "USize", "A target-width unsigned integer used for element counts, indices, capacities, and element offsets."),
    ADDRESS_TYPE = 15 => ("Address", "Address", "An opaque capability for host-managed storage. Only the C host copy primitives and extern contracts interpret its referent."),
    BUFFER_TYPE = 16 => ("Buffer", "Buffer<T>", "A mutable mal-owned sequence. Copies share the same buffer, and storage is reclaimed after its references disappear."),
);

macro_rules! predefined_values {
    ($( $constant:ident = $id:literal => ($name:literal, $detail:expr, $documentation:literal, $callable:literal) ),+ $(,)?) => {
        $(pub const $constant: ValueId = ValueId($id);)+

        pub const VALUES: &[PredefinedValue] = &[
            $(PredefinedValue {
                name: $name,
                id: $constant,
                detail: $detail,
                documentation: $documentation,
                callable: $callable,
            },)+
        ];
    };
}

predefined_values!(
    FALSE_VALUE = 0 => ("false", Some("Bool"), "The Boolean value for a false condition.", false),
    TRUE_VALUE = 1 => ("true", Some("Bool"), "The Boolean value for a true condition.", false),
    NEW_VALUE = 2 => ("new", Some("(Buffer<T>, T) -> USize"), "Appends a value to a `Buffer<T>` and returns its stable element index.", true),
    GET_VALUE = 3 => ("get", Some("(Buffer<T>, USize) -> T"), "Reads an element from a `Buffer<T>`. The index must be within its current count.", true),
    PUT_VALUE = 4 => ("put", Some("(Buffer<T>, USize, T) -> Unit"), "Replaces an element in a `Buffer<T>`. The index must be within its current count.", true),
    MAKE_VALUE = 5 => ("make", Some("USize -> Buffer<T>"), "Creates an empty `Buffer<T>` with the requested initial capacity.", true),
    FROM_VALUE = 6 => ("from", Some("(Address, USize, USize) -> Buffer<T>"), "Copies an exact element range from initialized C-host storage into a new `Buffer<T>`.", true),
    INTO_VALUE = 7 => ("into", Some("(Buffer<T>, Address, USize, USize) -> Unit"), "Copies a Buffer range into C-host storage without consuming or mutating the Buffer.", true),
    FILL_VALUE = 8 => ("fill", Some("(Buffer<T>, USize, USize, T) -> Unit"), "Assigns one value to a Buffer range, extending its count without creating a gap.", true),
    COPY_VALUE = 9 => ("copy", Some("(Buffer<T>, USize, Buffer<T>, USize, USize) -> Unit"), "Copies a Buffer range into another range, extending the destination count without creating a gap.", true),
);

pub fn first_source_type_id() -> u32 {
    TYPES
        .iter()
        .map(|entry| entry.id.0)
        .max()
        .map_or(0, |id| id + 1)
}

pub fn first_source_value_id() -> u32 {
    VALUES
        .iter()
        .map(|entry| entry.id.0)
        .max()
        .map_or(0, |id| id + 1)
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::*;

    #[test]
    fn predefined_names_and_ids_are_unique_and_dense() {
        assert_unique(TYPES.iter().map(|entry| (entry.name, entry.id.0)));
        assert_unique(VALUES.iter().map(|entry| (entry.name, entry.id.0)));
        assert_dense(TYPES.iter().map(|entry| entry.id.0));
        assert_dense(VALUES.iter().map(|entry| entry.id.0));
        assert!(
            TYPES
                .iter()
                .all(|entry| !entry.detail.is_empty() && !entry.documentation.is_empty())
        );
        assert!(
            VALUES
                .iter()
                .all(|entry| entry.detail.is_some() && !entry.documentation.is_empty())
        );
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
