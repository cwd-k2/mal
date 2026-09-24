# backend/llvm

Lowers the admitted execution plan to an LLVM module and the C shim that connects it to the host. Program-specific
execution belongs here; program-independent mechanisms live in the C runtime (`compiler/runtime/c11`). The
responsibility boundary is documented in `docs/implementation/execution-backend.md`.

| Module | Responsibility |
|---|---|
| `host_bridge`, `host_bridge/plan` | marshalling plan and typed C syntax for converting between LLVM values and public C host values |
| `shim` | C11 entry point that passes process arguments and calls the internal root bridge |
| `body/types` | LLVM value types, target pointer size, and scalar and value ABI alignment |
| `body/admission` | target-width literal, layout, and alignment checks before artifact generation |
| `body/plan` | entry functions, reachable states, slots, and constant plans for closed top-level values |
| `body/setup` | identity and frame-tag indexing, function emitter admission, prologue, and output order |
| `body/terminator` | control terminators as branches, calls, returns, and case dispatch |
| `body/call_emission` | value, environment, and parameter-responsibility handoff at call boundaries |
| `body/control_storage`, `body/control_top` | region-local storage view and top access, synchronized at native Mal call boundaries |
| `body/frame` | frame layout, resume dispatch, and owner transfer |
| `body/aggregate` | product and sum construction, case dispatch, and payload extraction |
| `body/value` | retain, transfer, and release of managed values, pattern destinations, and dead-slot cleanup |
| `body/memory` | `Address` and `Buffer` operations, canonical layout access, and Symbol/Buffer snapshot conversion |
| `body/scalar` | integer and floating-point widths, literals, and instruction selection |
| `optimization/*` | target-specific emission decisions that never change the execution plan |
