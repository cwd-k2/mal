# C11 runtime

Program-independent mechanisms selected and linked as needed into programs produced by `malc`. `core.c` and `control.c`
are always included; byte, Buffer, and Symbol sources are included when the generated artifacts reference that runtime.
Program-specific behavior, such as frame layout, resume targets, and owner transfer order, belongs to the generated LLVM module.

| File | Responsibility |
|---|---|
| `core.c` | the mechanisms every program links: reference-counted managed owners used by closure environments, Buffers, and Pools, plus the trap terminal |
| `control.c` | growable control storage and the native recursion bound |
| `bytes.c`, `bytes_internal.h` | byte owners, and the header layout static owners share with LLVM |
| `buffer.c`, `buffer_internal.h` | stable `Buffer` objects, small inline storage, flat-storage growth, and managed-element ownership |
| `buffer_range.c` | `fill` and `copy` |
| `buffer_host.c` | Buffers exchanged with the host: `from`, `into`, and the process arguments |
| `buffer_symbol.c` | byte `*` at the operand's last use |
| `pool.c`, `pool_internal.h` | internal Pool object, metadata, occupancy bitmap, payload relocation, and core exchanges; not selected by a source construct yet |
| `symbol.c` | `Symbol` operations |
| `runtime.h` | declarations shared by the runtime sources |
