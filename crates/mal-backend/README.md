# mal-backend

Turns a checked program into generated artifacts: lowering (`core`, `anf`, `closure`, `control`), the backend-independent
execution plan (`execution`), and LLVM, C header, and shim generation (`backend`) together with the C11 runtime in
`runtime/c11`. It never touches the file system or starts a process; `pipeline::generate` returns the module, shim, header,
and runtime sources as text for the driver to write.
