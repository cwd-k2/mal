# specialize

Selects the bindings reachable from the entry and turns each generic use into a monomorphic instance shared per
concrete type argument list. Each instance gives its binders fresh identities, so later stages can key facts by
`ValueId` program-wide.

| Module | Responsibility |
|---|---|
| `mod` | the worklist of requested instances, one per generic and concrete argument list |
| `instance` | the expansion of one instance, after the requirements it carries hold |
| `expression` | specialization of value expressions |
| `structure` | specialization of blocks, bodies, continuations, and completions |
| `substitution` | type substitution over patterns and completions |
| `instance_identity` | fresh identities for the binders of one instance |
| `admission` | the limit on how many instances a program may specialize |
| `selection` | the operation implementation a concrete family reference selects |
