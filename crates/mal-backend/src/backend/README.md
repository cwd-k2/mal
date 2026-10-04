# backend

Turns an admitted execution plan and checked host interface into build artifacts. The LLVM module owns program-specific
execution; generated C owns the public host surface and shim; selected C11 runtime sources provide program-independent
mechanisms.

| Module | Responsibility |
|---|---|
| `abi` | the internal bridge plan LLVM and generated C share |
| `artifact` | generated artifacts and backend failures |
| [`c`](c/README.md) | generated C: public headers, host stubs, and host-visible types |
| [`llvm`](llvm/README.md) | the LLVM backend |
| `runtime` | the runtime sources a program's artifacts reference |
| `source_layout` | canonical memory layout shared by LLVM and C helpers |

This directory does not read or write files and does not start Clang. `mal-backend::pipeline` returns text artifacts;
`mal-compiler::driver` owns their paths, compilation, and linking.
