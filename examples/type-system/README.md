# Type-system example

This executable combines the type-system features that matter when writing ordinary mal code:
kind-polymorphic transparent aliases, partial type application, constructor-polymorphic functions,
a generic opaque constructor, an open `Storable(F<A>)` requirement, and an operation family selected
by a closed constructor key.

`Id` is used once with `Pair`, proving that an unconstrained alias parameter is not fixed to kind
`Type`. `WithInt32` partially applies the binary `Product` constructor. `duplicate<F, A>` stores an
open application in a Buffer, and `project<F, A>` has implementations for both a transparent product
constructor and the opaque `Box` constructor. Constructor arguments remain explicit at calls while
ordinary element arguments are inferred.

Build and run from the repository root in Nushell:

```nu
nix develop --command cargo run -p mal-compiler -- build examples/type-system/program.mal --output /tmp/mal-type-system
/tmp/mal-type-system
```

The executable prints nothing and exits with status 0.
