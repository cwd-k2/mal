# Compiler pipeline

This capstone compiles Brainfuck to textual LLVM IR. mal validates bracket nesting, assigns loop
identities, emits a complete module, and reports recoverable file and output errors. Non-command
bytes are comments. The emitted program uses a fixed 30,000-byte tape; moving outside it is invalid
input for this example, and input EOF becomes byte 255.

`compiler.mal` owns translation and opaque compiler state. The `linux` modules separate syscall
transport, mapped-memory growth, file admission, and partial output. After reading, the initialized
source prefix is admitted into a mal-owned Buffer before the external mapping is released. The C
host only adapts typed extern calls to Linux syscalls.

> [!NOTE]
> The source mapping is external authority only while reading. Compilation begins after its bytes
> have been admitted into managed storage and the mapping has been released.

```nu
nix develop --command cargo run -p mal-compiler -- build examples/compiler-pipeline/program.mal --output /tmp/mal-bfc
/tmp/mal-bfc examples/compiler-pipeline/hello.bf | save --force /tmp/a.ll
nix develop --command clang /tmp/a.ll -o /tmp/a
/tmp/a
```

The generated program prints `A`.
