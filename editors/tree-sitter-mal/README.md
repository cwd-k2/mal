# tree-sitter-mal

This directory contains the Tree-sitter grammar and editor queries shared by the repository's
Neovim and Helix support. The language grammar is defined by
[`docs/spec/grammar.md`](../../docs/spec/grammar.md); this parser is an editor-facing representation,
not a separate language authority.

Generate and test the distributable parser sources from the repository's Nix development shell:

```nu
tree-sitter generate
tree-sitter test
```

`src/parser.c`, `src/grammar.json`, and `src/node-types.json` are committed so consumers can build
the parser without running the JavaScript grammar generator. The repository flake builds the
platform-specific parser and queries as `packages.${system}.editor-runtime`. Local editor
development also creates shared libraries under the ignored `.artifacts/editor-runtime/` directory
through `scripts/editor-dev.nu`.

The grammar, queries, and generated parser are covered by the repository's MIT license. Headers
copied from Tree-sitter during generation retain Tree-sitter's MIT terms in
[`third-party/tree-sitter/LICENSE`](third-party/tree-sitter/LICENSE). This directory contains the
upstream notice and is not part of the `tree-sitter-mal` project's own licensing declaration.
