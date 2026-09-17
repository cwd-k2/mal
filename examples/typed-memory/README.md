# Typed external memory example

This example obtains mutable storage from the host and places two deliberately unaligned canonical
`Sample` products in it. The generic `swap<A>` operation uses the static layout carried by two
`Cursor<A>` values to exchange complete values and specializes to `Sample`. A C host operation then
reads, updates, and writes the first sample through the generated canonical-memory helpers. The
program exits with status 0 when the swap and the cross-language update both preserve the product.

From the repository root in Nushell:

```nu
nix develop --command cargo run --manifest-path compiler/Cargo.toml -- build examples/typed-memory/program.mal --output /tmp/mal-typed-memory
/tmp/mal-typed-memory
```
