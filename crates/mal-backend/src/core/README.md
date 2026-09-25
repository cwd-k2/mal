# core

Desugars a specialized checked program into the core language: explicit evaluation order, lexical joins, and no surface control forms.
`anf`, `closure`, and `control` continue the lowering; each has a single stage and keeps the previous stage's identities.

| Module | Responsibility |
|---|---|
| `interface` | extracts the host-visible `ProgramInterface` and nothing else |
| `external` | turns external operation signatures into capture-free lambdas and external calls |
| `expression` | dispatches checked expression kinds to the modules that own control, memory, and `Buffer` lowering |
| `lambda` | lambda parameters and body items as core bindings, lexical joins, and closure captures |
| `buffer` | `Buffer` `make`, `new`, `get`, `put`, `fill`, and `copy` as core operations with logical operands rather than source products |
| `bool` | `Bool` elimination as an explicit `case` |
| `elimination` | sum elimination continuations as pattern-binding `case` arms, result binder names as jumps to the result join, and an elimination that sends every variant to the same position of one result as a plain jump of the scrutinee, which keeps a forwarded call a tail call |
| `completion` | lowers body items iteratively and connects `Value` paths and direct result blocks to lexical joins |
| `completion/abrupt` | result transfer, empty elimination, all-abrupt branches, and terminal control of direct blocks |
| `completion/result_block` | maps direct result binders to join targets and connects block body and continuation |
| `completion/value` | operator values that contain control paths, rebuilt as core primitives and `Bool` elimination |
| `completion/presence` | classifies checked subtrees that need lexical continuations |

Later stages in this crate:

| Stage | Responsibility |
|---|---|
| `anf` | blocks of atoms and operations, keeping logical operands of memory and `Buffer` primitives and lambda-local join identities |
| `closure` | capture schemas of ordinary functions, entry function identity, and join bodies within a function |
| `control` | states, join targets, terminators, and live values of resume frames, without calls and without duplicating function environment schemas |
| `control/forwarding` | normalizes identity and terminal `Unit` continuations to tail calls |
| `control/liveness` | backward liveness of local values and closure environments, and use counts of control bindings |
