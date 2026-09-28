# core

Desugars a specialized checked program into the core language: explicit evaluation order, lexical joins, and no surface control forms.
[`anf`](../anf/), [`closure`](../closure/), and [`control`](../control/) continue the lowering and document their own representations.

| Module | Responsibility |
|---|---|
| `interface` | extracts the host-visible `ProgramInterface` and nothing else |
| `external` | turns external operation signatures into capture-free lambdas and external calls |
| `expression` | dispatches checked expression kinds to the modules that own control, memory, and `Buffer` lowering |
| `lambda` | lambda parameters and body items as core bindings, lexical joins, and closure captures |
| `buffer` | `Buffer` `make`, `new`, `get`, `put`, `fill`, and `copy` as core operations with logical operands rather than source products, for operands lowered directly and through a lexical join alike |
| `bool` | `Bool` elimination as an explicit `case` |
| `primitive`, `pattern` | binary operators as core primitives, and checked patterns as core patterns |
| `elimination` | sum elimination continuations as pattern-binding `case` arms, result binder names as jumps to the result join, and an elimination that sends every variant to the same position of one result as a plain jump of the scrutinee, which keeps a forwarded call a tail call |
| `completion` | lowers body items iteratively and connects `Value` paths and direct result blocks to lexical joins |
| `completion/abrupt` | result transfer, empty elimination, all-abrupt branches, and terminal control of direct blocks |
| `completion/branch` | sum elimination, `if`, and short-circuit operator bodies whose `Value` paths continue at a lexical join |
| `completion/result_block` | maps direct result binders to join targets and connects block body and continuation |
| `completion/value` | operator values that contain control paths, rebuilt as core primitives and `Bool` elimination |
| `completion/presence` | classifies checked subtrees that need lexical continuations |

Core separates direct `Buffer` storage operations into `BufferOperation`. Snapshot conversion and copies across
the C host-memory boundary remain `MemoryPrimitive`; they have different representation and ownership contracts
even when the checked AST initially classifies both families as memory primitives.
