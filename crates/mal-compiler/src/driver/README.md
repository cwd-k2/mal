# driver

The external boundary of `malc`: everything that touches the file system or starts a process. The stages behind it work on
in-memory values and return generated text.

| Module | Responsibility |
|---|---|
| `mod` | the `check` and `emit` use cases |
| `files` | generated files and temporary directories |
| `error` | the driver failure and the message `malc` prints for it |
| `build` | the build use case |
| `toolchain` | compilation of every artifact with the pinned Clang for the host target |
| `toolchain/optimization` | Clang arguments for `baseline` and `production` |

Reading and resolving source files is done by `mal-syntax`; this crate only decides where generated files go.
