# mal-compiler

The `malc` command: argument parsing (`cli`) and the external boundary (`driver`). The driver loads source graphs, writes
generated files, and runs the pinned Clang and LLD; everything that understands mal itself lives in the other crates.

Commands, options, supported environments, and the artifact policy are documented in
`docs/development/compiler-usage.md`. Crate boundaries are described in `docs/implementation/responsibilities.md`, and the
driver modules in `src/driver/README.md`.

```nu
cargo test -p mal-compiler
cargo run -p mal-compiler -- --help
```
