# mal-lsp

Language server for mal. It speaks LSP over stdio and reuses the `malc` frontend, so diagnostics and semantic
answers come from the same analysis as `malc check`. Behavior is described in `docs/development/editor-tooling.md`.

| Module | Responsibility |
|---|---|
| `server` | JSON-RPC method dispatch and the open-document lifecycle |
| `server/analysis` | source graph, frontend analysis, semantic index state transitions, and diagnostic conversion |
| `server/requirement` | `require` path completion and document links |
| `server/semantic` | LSP representations of hover, navigation, rename, symbols, completion, and semantic tokens |
| `protocol` | LSP message types |
