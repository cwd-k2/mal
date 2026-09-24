# Opaque aggregate example

This example passes a copyable opaque `Allocation` handle and a `USize` count as one product parameter
and returns them inside a sum aggregate. The host adapter is compiled against the generated program
header.

`ResizeResult` documents the variant roles, but remains a structural sum. Likewise, placing
`Allocation` in a product neither transfers ownership nor makes the handle linear; the host contract
still defines which copies may be used and when the referent remains live.

From the repository root in Nushell:

```nu
nix develop --command cargo run -p mal-compiler -- check examples/opaque-aggregate/program.mal
nix develop --command cargo run -p mal-compiler -- build examples/opaque-aggregate/program.mal --output /tmp/mal-opaque-aggregate
/tmp/mal-opaque-aggregate
```

Expected output:

```text
42
```

The executable exits with status 0 after checking the returned handle and product fields.
