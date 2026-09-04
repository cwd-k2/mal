# malc

Rust reference compiler for mal v0.4.

The compiler implements the pipeline from source loading through native C compilation and linking. The implemented language slice includes closures, sums, fixed-width integers, byte literals, integer conversions, and the scalar C host ABI. The active milestone and acceptance criteria live in the [`implementation roadmap`](../docs/implementation/roadmap.md); [`m0.md`](../docs/implementation/m0.md) records the original vertical slice.

From this directory:

```nu
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
cargo run -- --version
```
