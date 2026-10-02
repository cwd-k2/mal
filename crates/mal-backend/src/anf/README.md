# anf

Linearizes each core expression into a block of bindings whose operands are atoms. Operand evaluation follows source
order, and every intermediate value receives a fresh `ValueId`; later stages can therefore inspect order and identity
without walking nested expressions again.

| Module | Responsibility |
|---|---|
| `mod` | lowering of core expressions into administrative normal form |
| `ast` | the ANF program |

This stage preserves lexical lambdas, captures, and join identities. Closure representation belongs to `closure`, and
call/control-state planning belongs to `control` and `execution`.
