# mal

mal is a small, strictly evaluated, statically typed language built around explicit application and
affine control.

## Language at a glance

```mal
ErrorCode :: Int32;
CheckedInt :: [Int32, ErrorCode];

extern readInt :: Unit -> Int32;
extern writeInt :: Int32 -> Unit;

requireNonnegative :: Int32 -> CheckedInt := (value) -> [return, throw] => {
    when (value < 0) {
        throw(1);
    };
    return(value);
};

main :: Unit -> Int32 := () -> [return] => {
    input := readInt();
    when (input == -1) {
        return(0);
    };

    status := requireNonnegative(input)[
        (value) -> {
            writeInt(value);
            0i32;
        },
        (error) -> error
    ];

    return(status);
};
```

This program shows the main control forms:

- `extern` functions provide host operations through ordinary application.
- `[return, throw] =>` selects a sum variant by applying one lexical result continuation.
- `value[first, second]` eliminates a sum and evaluates only its selected continuation.
- `main` is the executable root; its sequential block uses `[return] =>` for early and final results.

`return` and `throw` are ordinary local names, not statements or keywords.

The v0.6 development profile includes:

- immutable bindings, lexical closures and result blocks, products and sums, fixed-width numeric
  types, and parametric polymorphism;
- immutable `Symbol` and `Packed` values, plus typed views over external memory;
- explicit external effects through `extern` operations.

Allocation, files, networking, clocks, randomness, and other platform policy remain on the host
side of the `extern` boundary.

## Repository

The repository contains the Rust reference compiler (`malc`), an LLVM execution backend with a C11
runtime and generated host interface, checked examples, a formatter, an LSP server, and VS Code,
Neovim, and Helix support. Neovim and Helix share a Tree-sitter grammar. The current profile is under
development; generated artifacts and host ABI compatibility are not guaranteed across compiler
versions.

## Try it

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

## Documentation

Use [`docs/README.md`](docs/README.md) as the documentation index. In particular:

- [`docs/design/value-interpretation-and-control.md`](docs/design/value-interpretation-and-control.md)
  explains the relationship among reusable data, application, interpretation, and affine control.
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

The generated Tree-sitter headers under `editors/tree-sitter-mal/src/tree_sitter/` retain the
upstream MIT terms in
[`third-party/tree-sitter/LICENSE`](editors/tree-sitter-mal/third-party/tree-sitter/LICENSE).
