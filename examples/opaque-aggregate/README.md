# Opaque aggregate example

This example passes a copyable opaque handle and an integer as one product parameter and returns
them inside a sum aggregate. The host adapter is compiled against the generated program header.

From the repository root in Nushell:

```nu
nix develop --command cargo run --manifest-path compiler/Cargo.toml -- check examples/opaque-aggregate/program.mal
nix develop --command cargo run --manifest-path compiler/Cargo.toml -- build examples/opaque-aggregate/program.mal --output /tmp/mal-opaque-aggregate
/tmp/mal-opaque-aggregate
```

Expected output:

```text
42
```

The executable exits with status 0 after checking the returned handle and product fields.
