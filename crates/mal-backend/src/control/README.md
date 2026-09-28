# control

Lowers closure blocks and lexical joins to an explicit state graph. Operations remain typed and logical, while calls,
returns, jumps, and case dispatch become terminators. Each function and top-level initializer records exactly the states
reachable from its entry.

| Module | Responsibility |
|---|---|
| `mod` | state construction, join-target resolution, terminator selection, and per-owner reachable-state lists |
| `ast` | control states, operations, terminators, functions, top-level bindings, and state identities |
| `graph` | graph traversal and reachable-state order |
| `liveness` | values live across transitions, local value sets, and binding use counts |
| `forwarding` | normalization of identity and terminal-`Unit` continuations to tail calls |

Possible callees, call modes, recursive regions, frames, and ownership remain absent from this representation. `flow`
and `execution` derive those facts from the admitted control graph.
