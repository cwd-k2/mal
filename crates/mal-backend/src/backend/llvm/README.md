# backend/llvm

Lowers the admitted execution plan to an LLVM module and the C shim that connects it to the host. Program-specific
execution belongs here; program-independent mechanisms live in the C runtime (`crates/mal-backend/runtime/c11`). The
responsibility boundary is documented in `docs/implementation/execution-backend.md`.

| Module | Responsibility |
|---|---|
| `target` | the target data layout that emission sizes values with |
| `module` | the one LLVM module a program needs, assembled from feature groups |
| `module/declaration` | runtime and intrinsic declarations, one group per feature that calls them |
| `module/entry` | the internal root bridge that the C shim calls |
| `module/metadata` | alias metadata that keeps Buffer element accesses apart |
| `syntax` | the typed LLVM syntax model and its rendering to text |
| `host_bridge` | conversion between LLVM values and public C host values, planned per type in `host_bridge/plan` |
| `shim` | the C11 entry point that calls the root bridge with the process arguments |
| `body/types` | the LLVM representation of mal value types |
| `body/admission` | rejection of a program the target cannot represent, before any function is emitted |
| `body/admission/layout` | host memory layouts and runtime storage the target must represent |
| `body/admission/operation` | the storage and layouts each operation needs |
| `body/constants` | closed top-level values, evaluated once into LLVM constants |
| `body/setup` | one function emitter, from the states it owns to its emitted definition |
| `body/terminator` | control terminators |
| `body/operation` | dispatch of control operations to the modules below |
| `body/closure` | closure values and their capture environments |
| `body/primitive` | primitive scalar operators and numeric conversion |
| `body/bridge` | external operation calls through the C bridge |
| `body/symbol` | Symbol operations |
| `body/call_emission` | handoff of values, environments, and parameter responsibility at call boundaries |
| `body/control_storage`, `body/control_top` | region-local control storage and top, synchronized at native call boundaries |
| `body/frame` | suspension of a caller into a control frame |
| `body/frame/layout` | the field layout of a frame |
| `body/frame/continuation` | returning a value through a continuation |
| `body/frame/resume` | resumption of a suspended frame |
| `body/frame/region` | transitions between the functions of a common control region |
| `body/frame/native` | bounded native recursion for a self-recursive function |
| `body/aggregate` | products and sums |
| `body/value/atom` | materialization of operands |
| `body/value/slot` | slot states and the release of slots whose owners die |
| `body/value/lifetime` | typed share and drop of managed values |
| `body/value/pattern` | binding destinations of patterns |
| `body/value/use_effect` | the use effect an operand receives: borrow, share, consume, or take |
| `body/memory` | dispatch of `Address` and `Buffer` primitives |
| `body/memory/storage` | values in their canonical memory layout |
| `body/memory/view` | byte views, and the Symbol and Buffer conversions that copy |
| `body/memory/transfer` | the Symbol and Buffer conversions that move the byte owner at the operand's last use |
| [`body/memory/buffer`](body/memory/buffer/README.md) | Buffer operations |
| `body/scalar` | scalar literals and instruction selection |
| `optimization/*` | target-specific emission decisions that never change the execution plan |

## Construction boundary

`syntax` owns a restricted, typed model of every LLVM construct emitted by this backend. Body lowering selects
instructions and supplies typed operands; it does not assemble LLVM source lines or rely on indentation, opcode prefixes,
or rendered text to recover structure. The model admits only the LLVM subset used by mal and validates function-local
invariants before rendering. Rendering is the only operation that turns that model into LLVM text.

Procedural macros directly parse types, signatures, parameters, declarations, constants, instructions, terminators,
globals, and metadata. Instruction fields use Rust-like named fields, typed operands use `(type, value)`, and static types
such as `int(32)` and `ptr` need no implementation-driven wrapper. `{ ... }` embeds one typed Rust node and `..{ ... }`
splices a runtime-generated node sequence. `emit_instruction!` and `emit_terminator!` combine syntax admission with the
only function-emission registration boundary. LLVM functions and basic blocks remain under `FunctionBuilder`: CFG
construction has stateful block, terminator, and entry-instruction invariants that should not be hidden in a block macro.
Module symbol ordering and uniqueness likewise remain under `Module`. Dynamic lowering policy therefore stays in ordinary
Rust code. The shared notation and complete boundary are documented in
[`docs/implementation/backend-syntax-construction.md`](../../../../../docs/implementation/backend-syntax-construction.md).

`module` builds one logical LLVM module from feature groups. It derives byte-runtime requirements from references in the emitted
typed call structure and adds the matching declaration group. The C shim reports its own runtime requirement, and runtime-source
selection uses the union of both artifacts. The renderer currently writes the logical module to the single `program.ll` artifact;
physical partitioning into several `.ll` inputs is a delivery choice and does not change feature selection or body lowering.
