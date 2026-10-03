# continuation_specialization

Admits closed producer-consumer slices in a closure-converted program. The plan starts at the unique application of a
function-valued call result and follows only aliases within an effect-free interval whose application argument already
exists before the producer call. Aggregate storage, another function argument, a block result, or multiple applications
make the value escape and reject the demand.

The stage runs after `call_pattern` and before control lowering. It does not recognize source operation names such as
`State`, `bind`, or `fmap`. The initial plan records demand seeds and validates exact reconstruction; extending a seed
through producer results, joins, callbacks, and self-recursive edges precedes any calling-convention rewrite.

| Module | Responsibility |
|---|---|
| `plan` | admitted application demands and exact reconstruction |
| `index` | binding definitions, canonical aliases, uses, escape classification, and effect intervals |
