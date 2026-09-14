# mal

mal is a small, strictly evaluated, statically typed functional language. Its v0.5 development
profile combines immutable bindings, lexical closures, lexical result blocks, products and sums, fixed-width numeric
types, immutable byte-valued `Symbol`s, explicit external effects, and untyped `Ptr` memory access.
The language keeps allocation, files, networking, clocks, randomness, and other platform policy on
the host side of an explicit `extern` boundary.

The repository contains the Rust reference compiler (`malc`), an LLVM execution backend with a C11
runtime and generated host interface, checked examples, a formatter, an LSP server, and VS Code
support. The current profile is under development; generated artifacts and host ABI compatibility
are not guaranteed across compiler versions.

## Start here

The pinned development environment is provided by Nix. From the repository root in Nushell:

```nu
nix run . -- --help
nix develop
cargo test --manifest-path compiler/Cargo.toml
cargo run --manifest-path compiler/Cargo.toml -- check examples/print-and-closure/program.mal
cargo run --manifest-path compiler/Cargo.toml -- build examples/print-and-closure/program.mal --output /tmp/mal-example
cargo run --manifest-path compiler/Cargo.toml -- emit-atcoder examples/print-and-closure/program.mal --output /tmp/Main.cpp
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

## License

This repository is licensed under the [MIT License](LICENSE), except for the C11 runtime under
[`compiler/runtime/c11/`](compiler/runtime/c11/), which is licensed under the
[MIT No Attribution License](compiler/runtime/c11/LICENSE). The runtime is incorporated into
programs produced by `malc`; MIT-0 permits distributing those copies without an attribution
condition. These licenses do not claim rights in source programs merely because they are compiled
with `malc`.
