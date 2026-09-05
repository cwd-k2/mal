# Ptr memory example

This example obtains mutable storage from the host, performs deliberately unaligned `Int64` and
`UInt8` access through the minimal `Ptr` primitives, and exits with status 0 when both values round
trip correctly.

From the repository root in Nushell:

```nu
nix develop --command cargo run --manifest-path compiler/Cargo.toml -- build examples/ptr-memory/program.mal --output /tmp/mal-ptr-memory --link examples/ptr-memory/host.c
/tmp/mal-ptr-memory
```
