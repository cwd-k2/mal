# C11 runtime

Program-independent mechanisms linked into every program produced by `malc`. Program-specific behavior, such as frame
layout, resume targets, and owner transfer order, belongs to the generated LLVM module.

| File | Responsibility |
|---|---|
| `core.c` | closure environment allocation, retain and release dispatch, and the program-independent trap terminal |
| `control.c` | growable control byte storage: capacity, growth, release, and storage access for the internal ABI |
| `bytes.c`, `bytes_internal.h` | byte owner and managed `Buffer` storage policy, and the header layout shared with static owners emitted by LLVM |
| `symbol.c` | `Symbol` indexing, equality, concatenation, and reuse of dead operand storage |
| `runtime.h` | declarations shared by the runtime sources |
