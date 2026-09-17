# Numeric conversion example

This example exercises byte literals, fixed-width wrapping arithmetic, modulo integer conversion, and
the scalar host ABI through the public compiler driver.

From the repository root in Nushell, build and run the successful program:

```nu
nix develop --command cargo run --manifest-path compiler/Cargo.toml -- check examples/numeric-conversion/program.mal
nix develop --command cargo run --manifest-path compiler/Cargo.toml -- build examples/numeric-conversion/program.mal --output /tmp/mal-numeric-conversion
/tmp/mal-numeric-conversion
```

Expected output:

```text
255
0
18446744073709551615
```
