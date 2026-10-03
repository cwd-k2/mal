# continuation_specialization

Admits closed producer-consumer slices in a closure-converted program. The plan starts at the unique application of a
function-valued call result and follows only aliases within an effect-free interval whose application argument already
exists before the producer call. Aggregate storage, another function argument, a block result, or multiple applications
make the value escape and reject the demand.

The stage runs after `call_pattern` and before control lowering. It does not recognize source operation names such as
`State`, `bind`, or `fmap`. The plan records demand seeds, follows direct function results to calls or closure creators,
inventories creator instances and every call site involving candidate code, and validates exact reconstruction. A plan
is closed only when every inventoried call site is its seed producer or consumer, a traced result edge, or an application
inside a demanded closure. Creator provenance records aliases, joins, bounded product packing, captures, parameters,
results, and escaping destinations. Every transport must remain inside the slice before any calling-convention rewrite
is enabled.

| Module | Responsibility |
|---|---|
| `plan` | admitted application demands and exact reconstruction |
| `index` | binding definitions, canonical aliases, uses, escape classification, and effect intervals |
| `trace` | direct call, closure creator, branch, and result-join edges followed from demanded producers |
| `application` | closure-flow candidates reached while executing demanded closure targets |
| `inventory` | exact creator identities and all program call sites involving candidate slice code |
| `provenance` | creator and self-closure origins reaching each use, with transport and escape destinations |
| `request` | all-or-nothing worker identities under the shared closure-copy budget |
| `rewrite` | fresh worker copies and the eventual atomic calling-convention rewrite |
