# mal Language Support

This VS Code extension registers `.mal` files, provides lexical syntax highlighting and editing
configuration, and starts `mal-lsp` for diagnostics, formatting, and semantic editor features.

Build the server and install the extension dependencies before starting the extension host:

```nu
cargo build --manifest-path tools/mal-lsp/Cargo.toml --locked
cd editors/vscode
npm install
```

Set `mal.server.path` to the resulting executable path if `mal-lsp` is not available on `PATH`.

To try the extension from the repository root without packaging it, start VS Code with the extension
development path:

```nu
code --extensionDevelopmentPath (pwd | path join editors/vscode) .
```

Open `editors/vscode/test/highlight.mal` in the resulting window to inspect every supported lexical
category. The language grammar is owned by `docs/spec/grammar.md`; the language server only adapts
compiler queries and does not define separate language rules.
