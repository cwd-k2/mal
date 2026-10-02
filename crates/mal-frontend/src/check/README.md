# check

Type checking, and the specialization that follows it. Checking admits resolved programs into typed programs
with a `Value` or `Abrupt` completion for every expression.

| Module | Responsibility |
|---|---|
| `mod` | the checking entry points and the state shared while checking one program |
| `program` | the order in which top-level items are checked |
| `ast` | the typed program representation that checking emits |
| `binding` | bindings and patterns, and the reachability of body items |
| `entry` | the entry identity and its parameter form |
| `generic` | checking a declaration body once under rigid parameters, and the requirements it leaves for specialization |
| `control` | `if`, `when`, and direct blocks with their completion |
| `expression` | expression dispatch against the expected type |
| `expression/application` | the three application forms and their completion |
| `expression/elimination` | sum elimination continuations and the join of their completions |
| [`inference`](inference/README.md) | type arguments of generic references and calls |
| `lambda` | lambdas against the expected function type |
| `operator` | operator rules |
| [`operation`](operation/README.md) | operation families and their implementations |
| `integer` | integer literals and operands against fixed-width ranges |
| `product` | product expressions against an expected product |
| `memory` | typing of the predefined memory primitives |
| [`types`](types/README.md) | type-level terms and their properties |
| `interface` | the extern boundary and the host-visible metadata it yields |
| `initializer` | admission of closed top-level values |
| `float` | exact rounding of decimal float literals |
| [`specialize`](specialize/README.md) | monomorphic instances reachable from the entry |
| `specialization_identity` | the identities left free for specialization and core lowering to allocate |
| `type_fingerprint` | memoized structural hashes of canonical types |

Predefined memory and `Buffer` operations do not follow the ordinary function-call path. `resolve/predefined.rs`
owns their names and stable identities; `expression/application::check_call` dispatches those identities to
`memory/intrinsic.rs` and `memory/access.rs`. Additions must keep the resolver table, this dispatch, and the owned
memory checker in sync.
