# Operation family example

This example declares the function family `equal<A>` and value family `zero<A>` in one source file,
then adds exact `Int32` implementations and a generic `Buffer<A>` pattern in a directly dependent
file. `same<A>` acquires and propagates an equality requirement without receiving a runtime dictionary.

The call sites omit type arguments. Operands determine the `same<Int32>` specialization, and the
expected first operand type determines `zero<Int32>`. The two Buffer values select the structural
pattern independently of their `Symbol` element type. Non-overlap keeps selection independent of
source order and does not feed implementation information back into inference.

From the repository root in Nushell:

```nu
nix develop --command cargo run -p mal-compiler -- build examples/operation-family/program.mal --output /tmp/mal-operation-family
/tmp/mal-operation-family
```

The executable prints nothing and exits with status 0.
