# mal Language Support

This VS Code extension registers `.mal` files, provides lexical syntax highlighting and editing
configuration, and starts `mal-lsp` for diagnostics, formatting, and semantic editor features. From
the repository root, the complete development setup and launch is:

```nu
nu scripts/vscode-dev.nu
```

The script enters the pinned Nix environment when necessary, builds the server, installs locked
extension dependencies, probes `code`, `code-insiders`, and Windows-side WSL installations for a
compatible CLI. A desktop CLI starts the Extension Development Host. A WSL `remote-cli`, which does
not support the development-path option, receives an installable VSIX containing `mal-lsp`. Use
`--code-command /path/to/code` to override discovery, `--dry-run` to print the selected route, or
`--prepare-only` on a GUI-less machine. The equivalent manual setup is:

```nu
cargo build --manifest-path tools/mal-lsp/Cargo.toml --locked --release
cd editors/vscode
npm install
```

Set `mal.server.path` to the resulting executable path if the bundled `mal-lsp` should be overridden.

To try the extension from the repository root without packaging it, start VS Code with the extension
development path:

```nu
let extension_path = (pwd | path join editors/vscode)
run-external code $"--extensionDevelopmentPath=($extension_path)" .
```

Open `editors/vscode/test/highlight.mal` in the resulting window to inspect every supported lexical
category. The language grammar is owned by `docs/spec/grammar.md`; the language server only adapts
compiler queries and does not define separate language rules.
