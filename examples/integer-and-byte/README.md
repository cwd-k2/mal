# Integer and byte example

This example exercises byte literals, fixed-width wrapping arithmetic, modulo integer conversion, and the scalar host ABI through the public compiler driver. Shift counts are required to be within the left operand's bit width.

From the repository root in Nushell, build and run the successful program:

```nu
nix develop --command cargo run --manifest-path compiler/Cargo.toml -- check examples/integer-and-byte/program.mal
nix develop --command cargo run --manifest-path compiler/Cargo.toml -- build examples/integer-and-byte/program.mal --output /tmp/mal-integer-and-byte
/tmp/mal-integer-and-byte
```

Expected output:

```text
255
0
18446744073709551615
```
