# flow

Closure flow over the control program: which functions can be the callee, and which can be in the argument, of each
application site. Stages before and after `execution` read it; it derives no ownership, call mode, or region.

| Module | Responsibility |
|---|---|
| `mod` | the `ClosureFlow` result queried per application site |
| `analysis` | the fixed point over states: value sets per binding, capture slot, function result, and buffers |
| `transfer` | how each operation and terminator moves function values |
| `assign` | recording function values into patterns, filtered by the pattern's type |
| `owner` | the function or top-level initializer each control state belongs to |
| `compatible` | functions grouped by parameter and result type, the bound for a callee that no flow fact narrows |
