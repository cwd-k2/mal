# Strict floating-point example

This example checks exact decimal-to-binary rounding, the minimum binary32 subnormal, signed zero,
float-to-integer truncation, and the scalar C host ABI. The host inspects the received IEEE 754 bit
patterns without relying on decimal printing. This separates source-literal semantics from formatting
and avoids treating a printed decimal as proof of a particular binary value.

From the repository root in Nushell:

```nu
nix develop --command cargo run -p mal-compiler -- check examples/strict-float/program.mal
nix develop --command cargo run -p mal-compiler -- build examples/strict-float/program.mal --output /tmp/mal-strict-float
/tmp/mal-strict-float
```

The executable prints nothing and exits with status 0 after the host validates every value.
