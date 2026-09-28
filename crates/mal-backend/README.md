# mal-backend

Turns a checked program into generated artifacts: lowering (`core`, `anf`, `closure`, `control`), the target-layout-independent
execution plan consumed by the LLVM emitter (`execution`), and LLVM, C header, and shim generation (`backend`) together with the C11 runtime in
`runtime/c11`. It never touches the file system or starts a process; `pipeline::generate` returns the module, shim, header,
and runtime sources as text for the driver to write.

Stage-level responsibilities live beside their code: [`core`](src/core/), [`anf`](src/anf/),
[`closure`](src/closure/), [`control`](src/control/), [`call_pattern`](src/call_pattern/), [`flow`](src/flow/),
[`execution`](src/execution/), and [`backend`](src/backend/).
