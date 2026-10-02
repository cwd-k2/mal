# core

Desugars a specialized checked program into the core language: explicit evaluation order, lexical joins, and no surface control forms.
[`anf`](../anf/), [`closure`](../closure/), and [`control`](../control/) continue the lowering and document their own representations.

| Module | Responsibility |
|---|---|
| `interface` | the host-visible `ProgramInterface` |
| `external` | external operations as capture-free lambdas around external calls |
| `expression` | dispatch of checked expressions to the modules below |
| `lambda` | lambdas as core bindings, lexical joins, and closure captures |
| `buffer` | `Buffer` primitives as core operations with logical operands |
| `bool` | `Bool` elimination as an explicit `case` |
| `pattern` | checked patterns as core patterns |
| `elimination` | sum elimination continuations as `case` arms and jumps |
| `completion` | body items and the lexical joins their `Value` paths continue at |
| `completion/abrupt` | completions that leave without a value |
| `completion/branch` | branching bodies whose `Value` paths meet at a lexical join |
| `completion/result_block` | direct result blocks and their join targets |
| `completion/value` | operator values that contain control paths |
| `completion/presence` | which checked subtrees need lexical continuations |

Core separates direct `Buffer` storage operations into `BufferOperation`. Snapshot conversion and copies across
the C host-memory boundary remain `MemoryPrimitive`; they have different representation and ownership contracts
even when the checked AST initially classifies both families as memory primitives.

Symbol operations arrive from the checker as one `SymbolOperation` with a closed `SymbolPrimitive` and logical
operands, and every later stage carries them as `Symbol` operations. No stage selects a Symbol operation from a binary
operator and operand types.
