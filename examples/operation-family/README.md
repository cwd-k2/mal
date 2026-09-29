# Operation family example

This example declares the function family `equal<A>`, value family `zero<A>`, and
constructor-indexed family `extent<F, A>` in one source file. A directly dependent file adds exact,
structural, and closed-constructor implementations. `same<A>` and `extentOf<F, A>` acquire and
propagate requirements without receiving runtime dictionaries.

The call sites omit type arguments. Operands determine the `same<Int32>` specialization, and the
expected first operand type determines `zero<Int32>`. The two Buffer values select the structural
equality pattern independently of their `Symbol` element type. Calls to `extentOf` explicitly name
the rigid `Buffer` or `Pair` constructor while `A` remains inferred. Non-overlap keeps selection
independent of source order and does not feed implementation information back into inference.

From the repository root in Nushell:

```nu
nix develop --command cargo run -p mal-compiler -- build examples/operation-family/program.mal --output /tmp/mal-operation-family
/tmp/mal-operation-family
```

The executable prints nothing and exits with status 0.
