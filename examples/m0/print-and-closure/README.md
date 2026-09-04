# M0 print and closure example

This example exercises a host `Int32 -> Unit` operation, source-ordered effects, an escaping closure, and wrapping `Int32` arithmetic through the public compiler driver.

From the repository root in Nushell:

```nu
nix develop --command cargo run --manifest-path compiler/Cargo.toml -- check examples/m0/print-and-closure/program.mal
nix develop --command cargo run --manifest-path compiler/Cargo.toml -- build examples/m0/print-and-closure/program.mal --output /tmp/mal-m0-example --link examples/m0/print-and-closure/host.c
/tmp/mal-m0-example
```

Expected output:

```text
1
15
-2147483648
```
