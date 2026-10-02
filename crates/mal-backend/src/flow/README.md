# flow

Closure flow over the control program: which functions can be the callee, and which can be in the argument, of each
application site. Stages before and after `execution` read it; it derives no ownership, call mode, or region.

| Module | Responsibility |
|---|---|
| `mod` | the `ClosureFlow` result queried per application site |
| `analysis` | the fixed point of function values over states |
| `transfer` | how each operation and terminator moves function values |
| `assign` | function values recorded into patterns |
| `owner` | the function or top-level initializer each state belongs to |
| `compatible` | the functions a callee may be when no flow fact narrows it |
