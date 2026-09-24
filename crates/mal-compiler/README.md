# mal-compiler

The `malc` command: argument parsing (`cli`) and the external boundary (`driver`). The driver loads source
graphs, writes generated files, and runs the pinned Clang and LLD; everything that understands mal itself
lives in the other crates.

```nu
malc check program.mal
malc build program.mal -o program
malc emit header program.mal -o program.mal.h
malc emit host program.mal
malc emit atcoder program.mal -o Main.cpp
```

`emit` commands print to stdout unless `-o` is given. Supported environments, options, and the artifact
policy are documented in `docs/development/compiler-usage.md`. Crate boundaries are described in
`docs/implementation/responsibilities.md`.

```nu
cargo test -p mal-compiler
cargo run -p mal-compiler -- --version
```
