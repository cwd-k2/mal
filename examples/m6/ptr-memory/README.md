# M6 Ptr memory example

This example obtains mutable storage from the host, performs deliberately unaligned `Int64` and
`UInt8` access through the minimal `Ptr` primitives, and exits with status 0 when both values round
trip correctly.

From the repository root in Nushell:

```nu
nix develop --command cargo run --manifest-path compiler/Cargo.toml -- build examples/m6/ptr-memory/program.mal --output /tmp/mal-m6-example --link examples/m6/ptr-memory/host.c
/tmp/mal-m6-example
```
