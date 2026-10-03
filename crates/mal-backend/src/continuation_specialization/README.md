# continuation_specialization

Admits closed producer-consumer slices in a closure-converted program. The plan starts at the unique application of a
function-valued call result and follows only aliases within an effect-free interval whose application argument already
exists before the producer call. Aggregate storage, another function argument, a block result, or multiple applications
make the value escape and reject the demand.

The stage runs after `call_pattern` and before control lowering. It does not recognize source operation names such as
`State`, `bind`, or `fmap`. The plan records demand seeds, follows direct function results to calls or closure creators,
and validates exact reconstruction. Join, callback, and self-recursive edges must join the same closed slice before any
calling-convention rewrite is enabled.

| Module | Responsibility |
|---|---|
| `plan` | admitted application demands and exact reconstruction |
| `index` | binding definitions, canonical aliases, uses, escape classification, effect intervals, and direct producer-result edges |
