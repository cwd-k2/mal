# specialize

Selects the bindings reachable from the entry and turns each generic use into a monomorphic instance shared per
concrete type argument list.

| Module | Responsibility |
|---|---|
| `mod` | entry point, worklist of requested instances, and identity of the instance bindings |
| `instance` | expansion of one instance: kind requirement checks, `Storable` requirement checks of a selected implementation, and the substituted binding |
| `expression` | substitution and instance requests over value expressions, lambdas, and result blocks |
| `structure` | the same walk over blocks, bodies, continuations, and abrupt completions |
| `substitution` | type substitution over patterns and completions |
| `instance_identity` | fresh identities for the binders of one instance and the references that follow them |
| `admission` | the specialization limit and the top-level binding index |
| `selection` | the operation implementation whose key matches concrete family arguments, and its shared instance |
