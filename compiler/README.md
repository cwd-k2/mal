# malc

Rust reference compiler for mal v0.4.

The compiler implements the complete M0 pipeline: source loading, diagnostics, lexing, parsing, name/capture resolution, type checking, typed-core and ANF lowering, closure conversion, C/header emission, and native C compilation/linking. The `check`, `emit-c`, and `build` commands follow [`docs/implementation/m0.md`](../docs/implementation/m0.md).

From this directory:

```nu
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
cargo run -- --version
```
