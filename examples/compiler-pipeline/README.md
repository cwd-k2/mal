# Compiler pipeline

This capstone compiles Brainfuck to textual LLVM IR. mal validates bracket nesting, assigns loop
identities, emits a complete module, and reports recoverable file and output errors. Non-command
bytes are comments. The emitted program uses a fixed 30,000-byte tape; moving outside it is invalid
input for this example, and input EOF becomes byte 255.

`compiler.mal` owns translation, an opaque compiler state, and a growable byte emitter. Instructions
append to that emitter and produce one immutable Symbol snapshot only after successful validation,
instead of repeatedly copying the complete generated module. Nested loops use structural recursion;
straight-line source uses a tail edge.

`host.mal` owns the runtime extension surface. The C extension borrows the path as a `Symbol`, reads the
file directly into a mal-owned `Buffer<UInt8>`, and moves that buffer into mal. Generated LLVM is
returned to C as a borrowed `Symbol` and written without a transfer area. File and output failures
remain typed sums without exposing platform syscall shapes.

> [!NOTE]
> The host may inspect and construct runtime carriers directly. Correct Buffer ownership and the
> validity of external file resources are part of this example's extern contract, not language
> safety guarantees.

```nu
nix develop --command cargo run -p mal-compiler -- build examples/compiler-pipeline/program.mal --output /tmp/mal-bfc
/tmp/mal-bfc examples/compiler-pipeline/hello.bf | save --force /tmp/a.ll
nix develop --command clang /tmp/a.ll -o /tmp/a
/tmp/a
```

The generated program prints `A`.
