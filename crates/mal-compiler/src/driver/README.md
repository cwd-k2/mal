# driver

The external boundary of `malc`: everything that touches the file system or starts a process. The stages behind it work on
in-memory values and return generated text.

| Module | Responsibility |
|---|---|
| `mod` | the `check` and `emit` use cases |
| `files` | writing generated files, and the temporary directory a build uses when no artifact directory is given |
| `error` | the single driver failure type, rendered once into the message `malc` prints |
| `build` | one build use case: source graph, optimization mode, artifact directory, generated inputs, Clang, and the AtCoder carrier |
| `toolchain` | queries the pinned Clang for the host target triple and data layout and compiles every artifact for that same target |
| `toolchain/optimization` | Clang optimization arguments for `baseline` and `production`, independent of language semantics |

Reading and resolving source files is done by `mal-syntax`; this crate only decides where generated files go.
