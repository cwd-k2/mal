# check

Type checking, and the specialization that follows it. Checking admits resolved programs into typed programs
with a `Value` or `Abrupt` completion for every expression.

| Module | Responsibility |
|---|---|
| `mod` | program order, value environment, and result targets |
| `binding`, `entry` | bindings, patterns, and reachability of body items; the entry identity and its parameter form |
| `control` | `if`, `when`, direct blocks, and direct result blocks with their completion and local result targets |
| `expression` | expression dispatch, references, literals, and products checked against the expected type |
| `expression/application` | ordinary, receiver-first, and continuation application, result transfer, and empty elimination |
| `inference` | call-local structural constraints for generic value references, including contextual literals and lambda results |
| `expression/elimination` | sum elimination continuations (function values, branches of the enclosing invocation, and result binder names) and their completion join |
| `lambda` | parameters and body completion against the expected function type |
| `operator` | numeric, logical, and Symbol operator rules and left-associative operator chains |
| `integer`, `product` | integer literals and operands against the expected type with fixed-width ranges, and product expressions against an expected product |
| `memory` | `Buffer` access, Symbol snapshot conversion, and C host copy typing; logical operands are distinct from source products |
| `types`, `types/properties`, `types/display` | alias collection, iterative alias cycle detection, canonical type expansion, `Representable` and physical limits, bounded diagnostic display |
| `interface` | extern transport checks and extraction of host-visible metadata |
| `initializer` | admission of closed top-level values |
| `float` | exact rounding of decimal float literals to IEEE 754 binary formats |
| `specialize` | selection of bindings reachable from the entry, exact operation-family lookup, sharing of monomorphic instances per concrete type argument, and a fresh identity for every binder in each instance so that later stages can key facts by `ValueId` program-wide |
| `specialization_identity` | the first value and lambda identities that no checked binder uses, from which specialization and core lowering allocate |
| `type_fingerprint` | memoized structural hashes of canonical types, the keys that specialization and the backend closure flow group types by |

Predefined memory and `Buffer` operations do not follow the ordinary function-call path. `resolve/predefined.rs`
owns their names and stable identities; `expression/application::check_call` dispatches those identities to
`memory/intrinsic.rs` and `memory/access.rs`. Additions must keep the resolver table, this dispatch, and the owned
memory checker in sync.
