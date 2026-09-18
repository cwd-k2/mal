# Typed external memory example

This example obtains mutable storage from the host and first uses postfix `!` to place an aligned
canonical `Sample`. It then places two deliberately unaligned `Sample` products in the same storage.
The generic `swap<A>` operation uses the static layout carried by two `Cursor<A>` values to exchange
complete values and specializes to `Sample`. A C host operation then reads, updates, and writes the
first unaligned sample through the generated canonical-memory helpers. The program exits with status
0 when aligned access, exact unaligned access, the swap, and the cross-language update preserve the
products.

From the repository root in Nushell:

```nu
nix develop --command cargo run --manifest-path compiler/Cargo.toml -- build examples/typed-memory/program.mal --output /tmp/mal-typed-memory
/tmp/mal-typed-memory
```
