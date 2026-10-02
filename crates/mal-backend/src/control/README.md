# control

Lowers closure blocks and lexical joins to an explicit state graph. Operations remain typed and logical, while calls,
returns, jumps, and case dispatch become terminators. Each function and top-level initializer records exactly the states
reachable from its entry.

| Module | Responsibility |
|---|---|
| `mod` | lowering of closure programs into control states |
| `block` | one closure block as control states |
| `operation` | closure operations as control operations |
| `ast` | the control program |
| `graph` | traversal and reachable-state order |
| `liveness` | values live across transitions |
| `forwarding` | identity and terminal-`Unit` continuations normalized to tail calls |

Possible callees, call modes, recursive regions, frames, and ownership remain absent from this representation. `flow`
and `execution` derive those facts from the admitted control graph.
