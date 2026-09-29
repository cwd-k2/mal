# Language tour

This is the smallest broad example. It combines products and destructuring, a sum-valued division
result, direct result binders, an escaping closure, wrapping `Int32` arithmetic, and a narrow C host
operation. The sum makes division failure part of the value contract; `divideOr` decides how that
failure becomes an ordinary result.

> [!NOTE]
> A result binder such as `valid` is a control edge scoped to that block invocation. It is not a
> sum constructor that can escape as a value.

```nu
nix develop --command cargo run -p mal-compiler -- build examples/language-tour/program.mal --output /tmp/mal-language-tour
/tmp/mal-language-tour
```

It prints `42`, `15`, `42`, `-1`, and `-2147483648`, one value per line.
