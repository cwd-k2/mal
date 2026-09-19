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
    SYMBOL_TYPE = 10 => ("Symbol", "Symbol", "An immutable byte string owned by mal. `*symbol` obtains a zero-copy `Packed<UInt8>` view."),
    FLOAT32_TYPE = 11 => ("Float32", "Float32", "An IEEE 754 binary32 floating-point number."),
    FLOAT64_TYPE = 12 => ("Float64", "Float64", "An IEEE 754 binary64 floating-point number."),
    BYTE_SIZE_TYPE = 13 => ("ByteSize", "ByteSize", "A target-width size measured in bytes. Multiply a `USize` by a layout constant such as `#i64` to obtain one."),
    U_SIZE_TYPE = 14 => ("USize", "USize", "A target-width unsigned integer used for element counts, indices, capacities, and element offsets."),
    ADDRESS_TYPE = 15 => ("Address", "Address", "An opaque location in host-managed storage. `Address + ByteSize` derives another location without granting read or write authority."),
    REGION_TYPE = 16 => ("Region", "Region<T>", "A mutable view of external storage, valid only during the invocation that receives it. A `Region<T>` cannot be returned or captured by a nested lambda."),
    PACKED_TYPE = 17 => ("Packed", "Packed<T>", "An immutable, finite sequence owned by mal. Prefix and remainder operations share ownership; concatenation preserves both operands."),
    BUFFER_TYPE = 18 => ("Buffer", "Buffer<T>", "The scoped construction authority supplied by `make` or `edit`. Use `new`, `get`, and `put` only during that callback invocation."),
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
    PACK_VALUE = 2 => ("pack", Some("(Address, USize, USize) -> Packed<T>"), "Admits the half-open element range `[start, end)` at an address into an owned `Packed<T>`. Offsets are measured in elements, and the host must guarantee readable, initialized storage for the entire range.", true),
    EDIT_VALUE = 3 => ("edit", Some("(Packed<T>, Buffer<T> -> Unit) -> Packed<T>"), "Creates a new `Packed<T>` by exposing a scoped `Buffer<T>` initialized from `source`. The source remains an independent immutable value; the buffer cannot escape the callback.", true),
    NEW_VALUE = 4 => ("new", Some("(Buffer<T>, T) -> USize"), "Appends a value to a scoped `Buffer<T>` and returns its stable element index. The buffer may grow, so keep data as `Packed<T>` outside construction callbacks.", true),
    GET_VALUE = 5 => ("get", Some("(Buffer<T>, USize) -> T"), "Reads an element from a scoped `Buffer<T>` or `Region<T>`. The index must be within the buffer's current count or the region's length.", true),
    PUT_VALUE = 6 => ("put", Some("(Buffer<T>, USize, T) -> Unit"), "Replaces an element in a scoped `Buffer<T>` or writes an element through a `Region<T>`. The index must be in bounds and external storage must be writable.", true),
    MAKE_VALUE = 7 => ("make", Some("(USize, Buffer<T> -> Unit) -> Packed<T>"), "Builds a `Packed<T>` with a scoped `Buffer<T>`. `capacity` is an eager initial allocation request; append with `buffer.new(value)` and let the returned `Packed<T>` leave the callback.", true),
    VIEW_VALUE = 8 => ("view", Some("(Address, USize, USize, Region<T> -> R) -> R"), "Exposes the half-open element range `[start, end)` at an address as a `Region<T>` for one callback invocation. Offsets are measured in elements; the region cannot escape or be captured by a nested lambda.", true),
    SET_VALUE = 9 => ("set", Some("(Region<T>, Packed<T>) -> Region<T>"), "Writes every element of a `Packed<T>` to the start of a `Region<T>` and returns the unwritten suffix. The destination must be writable and at least as long as the source.", true),
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
