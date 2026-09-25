# check

Type checking, and the specialization that follows it. Checking admits resolved programs into typed programs
with a `Value` or `Abrupt` completion for every expression.

| Module | Responsibility |
|---|---|
| `mod` | program order, value environment, entry identity and parameter form, result targets, reachability of body items |
| `control` | `if`, `when`, direct blocks, and direct result blocks with their completion and local result targets |
| `expression` | expression dispatch, references, literals, and products checked against the expected type |
| `expression/application` | ordinary, receiver-first, and continuation application, result transfer, and empty elimination |
| `expression/elimination` | sum elimination continuations (function values, branches of the enclosing invocation, and result binder names) and their completion join |
| `lambda` | parameters and body completion against the expected function type |
| `operator` | numeric, logical, and Symbol operator rules and left-associative operator chains |
| `memory` | `Buffer` access, Symbol snapshot conversion, and C host copy typing; logical operands are distinct from source products |
| `types`, `types/properties`, `types/display` | alias collection, iterative alias cycle detection, canonical type expansion, `Representable` and physical limits, bounded diagnostic display |
| `interface` | extern transport checks and extraction of host-visible metadata |
| `initializer` | admission of closed top-level values |
| `float` | exact rounding of decimal float literals to IEEE 754 binary formats |
| `specialize` | selection of bindings reachable from the entry and sharing of monomorphic instances per concrete type argument |
