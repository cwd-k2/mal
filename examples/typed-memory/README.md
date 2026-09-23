# Typed external memory example

This example obtains deliberately unaligned host storage and exchanges two canonical
`SampleRecord` values through `from<SampleRecord>` and `buffer.into`. The mal-owned Buffer supports
ordinary mutation while the `Address` remains opaque; the C host performs one in-place update using
the generated canonical-memory helper.

From the repository root in Nushell:

```nu
nix develop --command cargo run --manifest-path compiler/Cargo.toml -- build examples/typed-memory/program.mal --output /tmp/mal-typed-memory
/tmp/mal-typed-memory
```
