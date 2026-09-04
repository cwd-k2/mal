# M1 integer and byte example

This example exercises byte literals, fixed-width wrapping arithmetic, modulo integer conversion, the scalar host ABI, and an invalid shift trap through the public compiler driver.

From the repository root in Nushell, build and run the successful program:

```nu
nix develop --command cargo run --manifest-path compiler/Cargo.toml -- check examples/m1/integer-and-byte/program.mal
nix develop --command cargo run --manifest-path compiler/Cargo.toml -- build examples/m1/integer-and-byte/program.mal --output /tmp/mal-m1-example --link examples/m1/integer-and-byte/host.c
/tmp/mal-m1-example
```

Expected output:

```text
255
0
18446744073709551615
```

Build and run the trapping program:

```nu
nix develop --command cargo run --manifest-path compiler/Cargo.toml -- build examples/m1/integer-and-byte/trap.mal --output /tmp/mal-m1-trap
/tmp/mal-m1-trap
```

The final command exits unsuccessfully and writes `mal trap: shift count out of range` to stderr.
