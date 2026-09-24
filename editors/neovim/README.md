# mal support for Neovim

The repository-local [`.nvim.lua`](../../.nvim.lua) registers `.mal`, enables the built-in LSP
client, and loads the generated Tree-sitter parser and highlight query. Neovim 0.11 or newer is
required. From the repository root, prepare and launch the pinned editor environment with:

```nu
nu scripts/dev.nu neovim
```

Neovim only evaluates project-local configuration after it has been enabled and trusted. The launch
script enables `exrc` for that invocation; use `:trust` after reviewing `.nvim.lua` when Neovim asks
for approval. Use `--prepare-only` to build `mal-lsp` and the ignored local runtime without opening
the editor.

The language behavior and development checks are documented in
[`docs/development/editor-tooling.md`](../../docs/development/editor-tooling.md).
