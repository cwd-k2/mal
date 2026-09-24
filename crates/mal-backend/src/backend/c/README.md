# backend/c

Generates the public C header and the host stub from `ProgramInterface`. The generated C shim that runs the LLVM
module lives in `backend/llvm/shim`.

| Module | Responsibility |
|---|---|
| `header`, `header/prefix` | generated header layout, include guard, portability macros, runtime ABI prefix |
| `host_signature` | host operation signatures |
| `types::TypeRegistry` | structural identity and C type mapping for the whole host interface |
| `types/collect` | postorder collection of host-visible representations from `ProgramInterface` |
| `types::HostTypes` | host-visible types reachable from externs and from aliases the checker admitted for canonical memory access |
| `types/host` | constructors, observers, and projections of host-mappable aggregates |
| `types/host/declaration` | type declarations needed by the host adapter and the public header |
| `types/host/memory` | unaligned-safe C read and write helpers derived from the shared canonical layout plan |
| `syntax` | typed C syntax nodes and their rendering, limited to the constructs the generated output uses |
