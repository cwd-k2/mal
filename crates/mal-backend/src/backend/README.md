# backend

Turns an admitted execution plan and checked host interface into build artifacts. The LLVM module owns program-specific
execution; generated C owns the public host surface and shim; selected C11 runtime sources provide program-independent
mechanisms.

| Module | Responsibility |
|---|---|
| `abi` | one internal bridge plan shared by LLVM and generated C |
| `artifact` | generated artifact values and backend failure reporting |
| `c` | public C headers, host stubs, host-visible types, and typed C syntax |
| `llvm` | target admission, execution lowering, LLVM construction, host bridge, and process shim |
| `runtime` | runtime-source selection from symbols referenced by the generated artifacts |
| `source_layout` | canonical memory stride, alignment, and offsets shared by LLVM and C helpers |
| `syntax_interpolation` | normalization used by the restricted C and LLVM construction macros |

This directory does not read or write files and does not start Clang. `mal-backend::pipeline` returns text artifacts;
`mal-compiler::driver` owns their paths, compilation, and linking.
