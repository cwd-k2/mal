# Generic loop example

This example implements iteration as the generic function
`loop<A, B> :: (A, A -> [A, B]) -> B`. The callback returns variant 0 with the next complete state
or variant 1 with the loop result. Call sites name the existing direct result binders `continue` and
`break`; those names are not special syntax.

`sumOddBelow` carries an index and accumulator, uses an early `continue` to skip even values, and
performs one million transitions. `repeat` carries an owned `Symbol` through the same abstraction and
returns a different type from its complete state. Together they exercise scalar, product, captured,
and managed values without adding mutable bindings or a loop primitive.

The recursive call in `loop` has no work pending after it. The reference compiler closes the step
closure and sum continuation into a recursive control region, places activation-local temporaries in
the region function's entry block, and executes the back edge without growing the native stack with
the transition count. The example is built and executed in both baseline and production profiles.

From the repository root in Nushell:

```nu
nix develop --command cargo run --manifest-path compiler/Cargo.toml -- build examples/generic-loop/program.mal --output /tmp/mal-generic-loop
/tmp/mal-generic-loop
```

The executable prints nothing and exits with status 0 after checking both results.
