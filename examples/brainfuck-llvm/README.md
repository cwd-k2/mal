# Brainfuck to LLVM IR compiler

This compiler is implemented in mal. Its C host implementations only invoke the Linux `mmap`,
`mremap`, `munmap`, `openat`, `lseek`, `close`, `read`, and `write` system calls and translate their
results to typed mal sums. mal constructs the terminated path, reads the Brainfuck source through
a growable anonymous mapping, handles partial reads and writes, validates bracket nesting, and
writes a complete LLVM IR module to standard output. The generated program uses a fixed 30,000-byte
tape and the host C library's `getchar` and `putchar`; moving outside the tape is invalid Brainfuck
input for this example, and end-of-file on input becomes byte `255`.

After the final read, `linux/file.mal` admits the initialized source prefix into a mal-owned
`Packed<UInt8>` and immediately releases the external mapping. Compilation therefore depends only on
the admitted source value, not on mapping lifetime or release authority. Non-command bytes are
comments. Recursive compilation of `[` assigns a unique LLVM block identity and stops at the matching
`]`, while straight-line input is processed by tail recursion.

The implementation is split by responsibility:

- `compiler.mal` translates Brainfuck instructions and loops to LLVM IR.
- `linux/syscall.mal` declares only typed Linux syscall transports.
- `linux/memory.mal` constructs and resizes anonymous mappings.
- `linux/file.mal` terminates paths, reads files through growable mappings, and admits source values.
- `linux/output.mal` copies `Symbol` bytes to mappings and handles partial writes.
- `linux/syscall.c` moves values between the generated C ABI and Linux syscall registers.

From the repository root in Nushell:

```nu
nix develop --command cargo run --manifest-path compiler/Cargo.toml -- build examples/brainfuck-llvm/program.mal --output /tmp/brainfuck-llvm
/tmp/brainfuck-llvm examples/brainfuck-llvm/hello.bf | save --force /tmp/a.ll
nix develop --command clang /tmp/a.ll -o /tmp/a
/tmp/a
```

The final command prints `A`.
