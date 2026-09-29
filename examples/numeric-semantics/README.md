# Numeric semantics

This example makes exact numeric rules observable through one host assertion. It checks byte value
conversion, fixed-width unsigned wrapping before widening, signed-to-unsigned modulo conversion,
ties-to-even `Float32` rounding, the minimum `Float32` subnormal, negative-zero preservation, and
floating-to-integer truncation.

> [!NOTE]
> Conversion observes the source value after its own arithmetic. Thus `255u8 + 1u8` wraps before
> the result is widened to `UInt64`.

```nu
nix develop --command cargo run -p mal-compiler -- build examples/numeric-semantics/program.mal --output /tmp/mal-numeric-semantics
/tmp/mal-numeric-semantics
```

Success is silent and exits with status 0.
