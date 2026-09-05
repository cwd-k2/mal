# malc

Rust reference compiler for mal v0.5.

The compiler implements the pipeline from source loading through native C compilation and linking. The implemented language slice includes closures, products, sums, fixed-width integers, strict floating point, byte and String literals, numeric conversions, untyped `Ptr` memory access, external opaque types, and the aggregate C host ABI. See the [compiler responsibilities](../docs/implementation/responsibilities.md) and [implementation notes](../docs/implementation/compiler.md) for the current structure.

The supported environment, CLI behavior, `CC` contract, linker model, and generated artifact policy are documented in the [`reference compiler usage contract`](../docs/development/compiler-usage.md).

From this directory:

```nu
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
cargo run -- --version
```
