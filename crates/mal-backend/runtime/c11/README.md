# C11 runtime

Program-independent mechanisms selected and linked as needed into programs produced by `malc`. `core.c` and `control.c`
are always included; byte, Buffer, and Symbol sources are included when the generated artifacts reference that runtime.
Program-specific behavior, such as frame layout, resume targets, and owner transfer order, belongs to the generated LLVM module.

| File | Responsibility |
|---|---|
| `core.c` | closure environment allocation, retain and release dispatch, and the program-independent trap terminal |
| `control.c` | growable control byte storage: capacity, growth, release, and storage access for the internal ABI |
| `bytes.c`, `bytes_internal.h` | byte owner storage policy, and the header layout shared with static owners emitted by LLVM |
| `buffer.c`, `buffer_internal.h` | `Buffer` storage: allocation, growth, `new`, and element ownership callbacks for managed elements |
| `buffer_range.c` | `fill` and `copy`, which extend the count and write a range, including overlapping managed copies |
| `buffer_host.c` | C host copies (`from`, `into`) and the process argument `Buffer<Symbol>` |
| `symbol.c` | `Symbol` indexing, range views, equality, concatenation, and reuse of dead operand storage |
| `runtime.h` | declarations shared by the runtime sources |
