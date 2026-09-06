# malc

Rust reference compiler for mal v0.5.

The compiler implements the pipeline from source loading through native C compilation and linking.
It also exposes frontend analysis, formatting, host-interface generation, and C emission as separate
in-memory paths. Host header and adapter generation consume the checked `ProgramInterface` without
lowering executable value bodies; C translation-unit generation continues through core, ANF, and
closure conversion.

The implemented language slice includes closures, products, sums, fixed-width integers, strict
floating point, byte and `Symbol` literals, `Symbol` observation and concatenation, numeric
conversions, target storage-size expressions, untyped `Ptr` memory access for numeric scalars and
pointers, explicit `Symbol` byte admission and storage, external opaque types, and the aggregate C
host ABI. See the [compiler responsibilities](../docs/implementation/responsibilities.md) and
[implementation notes](../docs/implementation/compiler.md) for the current structure.

The supported environment, CLI behavior, `CC` contract, linker model, and generated artifact policy are documented in the [`reference compiler usage contract`](../docs/development/compiler-usage.md).

From this directory:

```nu
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
cargo run -- --version
```
