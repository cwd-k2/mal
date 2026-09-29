# Language tour

This is the broad syntax and type-system entry point. It combines products and destructuring, a
sum-valued division result, direct result binders, an escaping closure, wrapping `Int32` arithmetic,
and a narrow C host operation. `WithInt32` partially applies a binary type constructor, while
`duplicate<F, A>` accepts that constructor and the imported opaque `Box` as higher-kinded arguments.
The inferred element type supplies each remaining `A`.

The final host assertion checks the exact numeric rules that are otherwise hard to observe: byte
conversion, fixed-width wrapping before widening, signed-to-unsigned modulo conversion, ties-to-even
`Float32` rounding, the minimum subnormal, negative zero, and float-to-integer truncation.

> [!NOTE]
> A result binder such as `valid` is a control edge scoped to that block invocation. It is not a
> sum constructor that can escape as a value.

`box.mal` owns the representation of `Box<A>`. The entry program can pass `Box` through generic
code, but it can only construct and observe boxed values through the module operations.

```nu
nix develop --command cargo run -p mal-compiler -- build examples/language-tour/program.mal --output /tmp/mal-language-tour
/tmp/mal-language-tour
```

It prints `42`, `42`, `15`, `42`, `-1`, and `-2147483648`, one value per line.
