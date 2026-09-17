# Typed external memory example

This example obtains mutable storage from the host and performs deliberately unaligned `Int64` and
`UInt8` access through typed `Cursor` placement on an `Address`. Generic `readCursor<A>` and
`writeCursor<A>` bindings preserve the cursor's static layout and specialize for both element types.
The program exits with status 0 when both values round trip correctly.

From the repository root in Nushell:

```nu
nix develop --command cargo run --manifest-path compiler/Cargo.toml -- build examples/typed-memory/program.mal --output /tmp/mal-typed-memory
/tmp/mal-typed-memory
```
