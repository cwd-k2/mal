# Monads

This example expresses a monad as two operation families keyed by a type constructor: `pure<F, A>`
and `bind<F, A, B>`. `monad.mal` writes `fmap`, `both`, and a monadic fold once against those
families. Each call names the constructor, and specialization produces a separate copy for each
constructor. No dictionary is passed at run time and no type is inspected.

`program.mal` supplies three instances:

- `Either<E>` stops at the first failure.
- `Option` stops at the first missing value.
- `State<S>` threads a counter through functions of the state.

The `Either<E>` and `State<S>` keys partially apply a two-parameter constructor. Their remaining
name (`E` or `S`) is a key variable, so one pair of implementations serves every error type or state
type. Constructor keys need a nominal head. Here every constructor is a file-local opaque type, so
callers see the monad operations but not the representation.

The same `foldEach` parses digits until the first non-digit (`Either<Symbol>`), divides until a zero
divisor (`Option`), and labels elements while advancing a counter (`State<USize>`).

```nu
nix develop --command cargo run -p mal-compiler -- build examples/monads/program.mal --output /tmp/mal-monads
/tmp/mal-monads
```

Success is silent and exits with status 0.
