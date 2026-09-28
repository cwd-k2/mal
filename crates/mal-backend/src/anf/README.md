# anf

Linearizes each core expression into a block of bindings whose operands are atoms. Operand evaluation follows source
order, and every intermediate value receives a fresh `ValueId`; later stages can therefore inspect order and identity
without walking nested expressions again.

| Module | Responsibility |
|---|---|
| `mod` | core-to-ANF lowering, left-to-right operand collection, and temporary identity allocation |
| `ast` | ANF blocks, atoms, operations, patterns, lambdas, joins, and top-level bindings |

This stage preserves lexical lambdas, captures, and join identities. Closure representation belongs to `closure`, and
call/control-state planning belongs to `control` and `execution`.
