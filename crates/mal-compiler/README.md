# malc

Rust reference compiler for mal v0.6.

The compiler implements the pipeline from source loading through LLVM module generation, C11
shim/runtime compilation, and native linking. It also exposes frontend analysis, formatting, and
host-interface generation as separate in-memory paths. Host header and adapter generation consume
the checked `ProgramInterface` without lowering executable value bodies; executable generation
continues through core, ANF, closure conversion, application control planning, and LLVM lowering.

The implemented language slice includes closures, products, sums, fixed-width integers, strict
floating point, byte and `Symbol` literals, postfix numeric conversions, explicit generics,
target-sized quantities, opaque `Address` capabilities, managed shared-mutable `Buffer` values,
C-host copy primitives, external opaque types, and the aggregate C host
ABI. See the
[compiler responsibilities](../docs/implementation/responsibilities.md) and
[implementation notes](../docs/implementation/compiler.md) for the current structure.

The supported environment, CLI behavior, `CC` contract, linker model, and generated artifact policy are documented in the [`reference compiler usage contract`](../docs/development/compiler-usage.md).

From this directory:

```nu
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
cargo run -- --version
```
