# Monads and comonads

This example expresses type classes as operation families keyed by a type constructor, using two
dual pairs:

- `monad.mal` declares the monad pair, `pure<F, A>` and `bind<F, A, B>`.
- `comonad.mal` declares the comonad pair, `extract<W, A>` and `extend<W, A, B>`.

Each file writes its derived operations once against those families: `fmap`, `both`, and a monadic
fold in `monad.mal`, and `duplicate` and `liftW` in `comonad.mal`. Each call names the constructor,
and specialization produces a separate copy for each constructor. No dictionary is passed at run time
and no type is inspected.

`effects.mal` supplies three monads, which build a result step by step:

- `Either<E>` stops at the first failure.
- `Option` stops at the first missing value.
- `State<S>` threads a counter through functions of the state.

`contexts.mal` supplies three comonads, which compute each position from its surroundings:

- `Env<E>` carries a read-only environment beside the focus.
- `Store<S>` is a lookup with a current position. Repeated `extend` runs the rule 90 cellular
  automaton on a ring.
- `Focus` is a Buffer with a current index. `extend` computes a moving average into a new Buffer.

The `Either<E>`, `State<S>`, `Env<E>`, and `Store<S>` keys partially apply a two-parameter
constructor. Their remaining name is a key variable, so one pair of implementations serves every
error, state, environment, or position type. Constructor keys need a nominal head. Here every
constructor is a file-local opaque type, so `program.mal` sees the operations but not the
representation.

> [!NOTE]
> `duplicate<Focus>` would put a `Focus` inside a Buffer, and `Focus` holds a Buffer, which is not
> `Storable`. Specialization rejects that use. The example therefore applies `duplicate` only to `Env`
> and `Store`.

```nu
nix develop --command cargo run -p mal-compiler -- build examples/monads-and-comonads/program.mal --output /tmp/mal-monads-and-comonads
/tmp/mal-monads-and-comonads
```

Success is silent and exits with status 0.
