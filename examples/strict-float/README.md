# Strict floating-point example

This example checks exact decimal-to-binary rounding, the minimum binary32 subnormal, signed zero, float-to-integer truncation, and the scalar C host ABI. The host inspects the received IEEE 754 bit patterns without relying on decimal printing.

From the repository root in Nushell:

```nu
nix develop --command cargo run --manifest-path compiler/Cargo.toml -- check examples/strict-float/program.mal
nix develop --command cargo run --manifest-path compiler/Cargo.toml -- emit-c examples/strict-float/program.mal --output /tmp/mal-strict-float.c
nix develop --command cargo run --manifest-path compiler/Cargo.toml -- build examples/strict-float/program.mal --output /tmp/mal-strict-float
/tmp/mal-strict-float
```

The executable prints nothing and exits with status 0 after the host validates every value.
