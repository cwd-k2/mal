# Compiler pipeline

This capstone compiles Brainfuck to textual LLVM IR. mal validates bracket nesting, assigns loop
identities, emits a complete module, and reports recoverable file and output errors. Non-command
bytes are comments. The emitted program uses a fixed 30,000-byte tape; moving outside it is invalid
input for this example, and input EOF becomes byte 255.

`compiler.mal` owns translation, an opaque compiler state, and a growable byte emitter. Instructions
append to that emitter and produce one immutable Symbol snapshot only after successful validation,
instead of repeatedly copying the complete generated module. Nested loops use structural recursion;
straight-line source uses a tail edge.

`host.mal` owns a bounded process-lifetime transfer area. The C host reads source into one external
allocation, mal admits it and releases the handle, and later output is copied through the transfer
area in chunks. File and output failures remain typed sums without exposing platform syscall shapes.

> [!NOTE]
> External source storage is authority only while reading. Compilation starts from an immutable
> Symbol after its bytes have been admitted and the allocation has been released.

```nu
nix develop --command cargo run -p mal-compiler -- build examples/compiler-pipeline/program.mal --output /tmp/mal-bfc
/tmp/mal-bfc examples/compiler-pipeline/hello.bf | save --force /tmp/a.ll
nix develop --command clang /tmp/a.ll -o /tmp/a
/tmp/a
```

The generated program prints `A`.
