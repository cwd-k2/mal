# M2 opaque aggregate example

This example passes a copyable opaque handle and an integer through flattened product parameters and returns them inside a sum aggregate. The host adapter is compiled against the generated program header.

From the repository root in Nushell:

```nu
nix develop --command cargo run --manifest-path compiler/Cargo.toml -- check examples/m2/opaque-aggregate/program.mal
nix develop --command cargo run --manifest-path compiler/Cargo.toml -- build examples/m2/opaque-aggregate/program.mal --output /tmp/mal-m2-example --link examples/m2/opaque-aggregate/host.c
/tmp/mal-m2-example
```

Expected output:

```text
42
```

The executable exits with status 0 after checking the returned handle and product fields.
