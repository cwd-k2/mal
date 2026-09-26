# backend/llvm

Lowers the admitted execution plan to an LLVM module and the C shim that connects it to the host. Program-specific
execution belongs here; program-independent mechanisms live in the C runtime (`crates/mal-backend/runtime/c11`). The
responsibility boundary is documented in `docs/implementation/execution-backend.md`.

| Module | Responsibility |
|---|---|
| `target` | target data layout admission and the scalar, pointer, and index layout used by emission |
| `module` | selection and composition of the LLVM declarations, definitions, metadata, and root bridge required by one program |
| `module/declaration` | typed environment, control, byte-runtime, and intrinsic declaration groups selected by `module` |
| `module/entry`, `module/metadata` | the internal root bridge and Buffer alias metadata |
| `syntax` | LLVM module, function, and basic-block construction followed by textual rendering |
| `host_bridge`, `host_bridge/plan` | marshalling plan and typed C syntax for converting between LLVM values and public C host values |
| `shim` | C11 entry point that passes process arguments and calls the internal root bridge |
| `body/types` | LLVM value types, target pointer size, and scalar and value ABI alignment |
| `body/admission` | target-width literal, layout, and alignment checks before artifact generation |
| `body/plan` | entry functions, reachable states, slots, and constant plans for closed top-level values |
| `body/setup` | identity and frame-tag indexing, function emitter admission, prologue, and output order |
| `body/terminator` | control terminators as branches, calls, returns, and case dispatch |
| `body/operation`, `body/bridge` | control operations, dispatched to the modules below, and external operation calls through the C bridge |
| `body/symbol` | Symbol literals, length, byte access, and concatenation |
| `body/call_emission` | value, environment, and parameter-responsibility handoff at call boundaries |
| `body/control_storage`, `body/control_top` | region-local storage view and top access, synchronized at native Mal call boundaries |
| `body/frame` | frame layout, resume dispatch, and owner transfer |
| `body/aggregate` | product and sum construction, case dispatch, and payload extraction |
| `body/value` | retain, transfer, and release of managed values, pattern destinations, and dead-slot cleanup |
| `body/memory` | `Address` and `Buffer` operations, canonical layout access, and Symbol/Buffer snapshot conversion |
| `body/scalar` | integer and floating-point widths, literals, and instruction selection |
| `optimization/*` | target-specific emission decisions that never change the execution plan |

## Construction boundary

`syntax` owns a restricted, typed model of every LLVM construct emitted by this backend. Body lowering selects
instructions and supplies typed operands; it does not assemble LLVM source lines or rely on indentation, opcode prefixes,
or rendered text to recover structure. The model admits only the LLVM subset used by mal and validates function-local
invariants before rendering. Rendering is the only operation that turns that model into LLVM text.

Template macros cover types, signatures, parameters, declarations, constants, instructions, terminators, globals, and
metadata. `{ ... }` embeds one typed Rust node and `{{ ... }}` splices a runtime-generated node sequence. LLVM functions
and basic blocks remain under `FunctionBuilder`: CFG construction has stateful block, terminator, and entry-instruction
invariants that should not be hidden in a block macro. Module symbol ordering and uniqueness likewise remain under
`Module`. Dynamic lowering policy therefore stays in ordinary Rust code. The shared notation and complete boundary are
documented in
[`docs/implementation/backend-syntax-construction.md`](../../../../../docs/implementation/backend-syntax-construction.md).

`module` builds one logical LLVM module from feature groups. It derives byte-runtime requirements from references in the emitted
typed call structure and adds the matching declaration group. The C shim reports its own runtime requirement, and runtime-source
selection uses the union of both artifacts. The renderer currently writes the logical module to the single `program.ll` artifact;
physical partitioning into several `.ll` inputs is a delivery choice and does not change feature selection or body lowering.
