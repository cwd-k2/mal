# Exact operation family example

This example declares the function family `equal<A>` and value family `zero<A>` in one source file,
then adds their exact `Int32` implementations in a directly dependent file. `same<A>` acquires and
propagates an equality requirement without receiving a runtime dictionary.

The call sites omit type arguments. Operands determine the `same<Int32>` specialization, and the
expected first operand type determines `zero<Int32>`. Selection remains exact: adding an
implementation for another closed type does not affect inference or the selected `Int32`
implementation.

From the repository root in Nushell:

```nu
nix develop --command cargo run -p mal-compiler -- build examples/operation-family/program.mal --output /tmp/mal-operation-family
/tmp/mal-operation-family
```

The executable prints nothing and exits with status 0.
