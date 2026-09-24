# mal support for Helix

The repository-local [`.helix/languages.toml`](../../.helix/languages.toml) registers `.mal` with
`mal-lsp`. The development launcher exposes the generated mal Tree-sitter parser and queries through
an ignored, repository-local Helix runtime:

```nu
nu scripts/dev.nu helix
```

Review and trust the workspace when Helix asks before loading `.helix/languages.toml`. Use
`--prepare-only` to build `mal-lsp` and the ignored local runtime without opening the editor.

The language behavior and development checks are documented in
[`docs/development/editor-tooling.md`](../../docs/development/editor-tooling.md).
