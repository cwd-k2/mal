# malc

Rust reference compiler for mal v0.5.

The compiler implements the pipeline from source loading through native C compilation and linking. The implemented language slice includes closures, products, sums, fixed-width integers, strict floating point, byte and String literals, numeric conversions, external opaque types, and the aggregate C host ABI. The active milestone and acceptance criteria live in the [`implementation roadmap`](../docs/implementation/roadmap.md); [`m0.md`](../docs/implementation/m0.md) records the original vertical slice.

The supported environment, CLI behavior, `CC` contract, linker model, and generated artifact policy are documented in the [`reference compiler usage contract`](../docs/development/compiler-usage.md).

From this directory:

```nu
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
cargo run -- --version
```
