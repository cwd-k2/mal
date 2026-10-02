# execution/ownership

The managed responsibility plan over finalized control transitions: for every managed value, who owns it, which uses
borrow, share, or consume it, and where it is dropped. Phases run in a fixed order and later phases rely on the facts
earlier ones complete. The plan's exact validator rebuilds it from the same authority and compares.

| Module | Responsibility |
|---|---|
| `mod` | the phase order and the plan the backend reads |
| `identity` | identities of edges, uses, and operands, and the use effects |
| `managed` | which types own managed values |
| `operand` | the operands of each operation and terminator, and whether each may pass ownership into the result |
| `liveness` | which managed bindings are live at each point |
| `authority` | the managed values whose lifetime authorizes each borrowed binding |
| `authority_lenders` | the owning binding behind each chain of borrowed aliases |
| `borrow` | the bindings that borrow instead of owning |
| `parameter` | how each function receives its managed parameter |
| `convention` | the call sites that hand their managed argument to the callee |
| `destination` | where a pattern puts the values it binds |
| `use_plan` | the borrow, share, or consume effect of every managed use |
| `drop_plan` | the drops on each control edge |
