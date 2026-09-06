# mal

mal is a small, strictly evaluated, statically typed functional language. Its v0.5 development
profile combines immutable bindings, lexical closures, products and sums, fixed-width numeric
types, immutable byte-valued `Symbol`s, explicit external effects, and untyped `Ptr` memory access.
The language keeps allocation, files, networking, clocks, randomness, and other platform policy on
the host side of an explicit `extern` boundary.

The repository contains the Rust reference compiler (`malc`), a C11 backend and generated host
interface, checked examples, a formatter, an LSP server, and VS Code support. The current profile is
under development; generated C and host ABI compatibility are not guaranteed across compiler
versions.

## Start here

The pinned development environment is provided by Nix. From the repository root in Nushell:

```nu
nix develop
cargo test --manifest-path compiler/Cargo.toml
cargo run --manifest-path compiler/Cargo.toml -- check examples/print-and-closure/program.mal
cargo run --manifest-path compiler/Cargo.toml -- build examples/print-and-closure/program.mal --output /tmp/mal-example
/tmp/mal-example
```

Use [`docs/README.md`](docs/README.md) as the documentation index. In particular:

- [`docs/spec/`](docs/spec/) defines language and host-interface behavior.
- [`docs/development/compiler-usage.md`](docs/development/compiler-usage.md) defines the supported
  commands, toolchain, and generated artifacts.
- [`docs/implementation/responsibilities.md`](docs/implementation/responsibilities.md) maps compiler
  stages to their code ownership and translation boundaries.
- [`docs/development/testing.md`](docs/development/testing.md) defines the required verification.
- [`examples/`](examples/) contains programs exercised through the public compiler driver.

Repository-level documentation outside `docs/` is written in English. Normative, design,
implementation, and development documentation under `docs/` is written in Japanese.
