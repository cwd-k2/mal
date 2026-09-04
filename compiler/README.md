# malc

Rust reference compiler for mal v0.4.

The executable currently establishes the dependency-free compiler workspace and version contract. Implementation proceeds according to [`docs/implementation/m0.md`](../docs/implementation/m0.md); unsupported compilation commands fail explicitly rather than accepting source incompletely.

From this directory:

```nu
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
cargo run -- --version
```
