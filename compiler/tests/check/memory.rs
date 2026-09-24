use super::*;

#[test]
fn checks_managed_buffer_construction_aliasing_and_access() {
    check_ok(
        "create :: Unit -> Buffer<Int32> := () -> {
           values := make<Int32>(4usize);
           alias := values;
           index := values.new(10i32);
           alias.put(index, alias.get(index) + 1i32);
           values.fill(1usize, 3usize, 12i32);
           alias.copy(0usize, values, 1usize, 2usize);
           fill(values, 2usize, 1usize, 13i32);
           copy(alias, 1usize, values, 0usize, 2usize);
           values;
         };",
    );
}

#[test]
fn checks_c_host_copy_primitives_and_symbol_snapshots() {
    check_ok(
        "snapshot :: (Address, USize) -> Symbol := (address, count) -> {
           bytes := from<UInt8>(address, 0usize, count);
           bytes.into(address, 0usize, count);
           *bytes;
         };
         mutable :: Symbol -> Buffer<UInt8> := (symbol) -> *symbol;",
    );
}

#[test]
fn rejects_invalid_buffer_operations_without_preserving_retired_syntax() {
    for text in [
        "bad := make<Symbol>(0usize);",
        "bad := make<Int32>(1i32);",
        "bad :: Buffer<Int32> -> Unit := (values) -> values.put(0usize, 1u32);",
        "bad :: Buffer<Int32> -> Unit := (values) -> values.fill(0usize, 1bytes, 1i32);",
        "bad :: (Buffer<Int32>, Buffer<UInt32>) -> Unit := (target, source) -> target.copy(0usize, source, 0usize, 1usize);",
        "bad :: Buffer<Int32> -> Int32 := (values) -> values.get(0bytes);",
        "bad :: Buffer<Int32> -> Unit := (values) -> values.into(0usize, 0usize, 1usize);",
        "bad :: Address -> Buffer<Symbol> := (address) -> from<Symbol>(address, 0usize, 1usize);",
        "extern bad :: Buffer<Int32> -> Unit;",
    ] {
        assert!(check_error(text).primary.is_some(), "input: {text}");
    }
}

#[test]
fn permits_buffer_values_to_cross_function_and_closure_boundaries() {
    check_ok(
        "identity<A> :: A -> A := (value) -> value;
         capture :: Buffer<UInt8> -> (Unit -> UInt8) := (values) ->
           () -> values.get(0usize);
         pass :: Buffer<UInt8> -> Buffer<UInt8> := (values) ->
           identity<Buffer<UInt8>>(values);",
    );
}
