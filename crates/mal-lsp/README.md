# mal-lsp

Language server for mal. It speaks LSP over stdio and reuses the `malc` frontend, so diagnostics and semantic
answers come from the same analysis as `malc check`. Behavior is described in `docs/development/editor-tooling.md`.

| Module | Responsibility |
|---|---|
| `server` | JSON-RPC dispatch and the open-document lifecycle |
| `server/analysis` | the frontend analysis behind each open document |
| `server/requirement` | `require` path completion and links |
| `server/semantic` | LSP results from the semantic index |
| `server/semantic/navigation` | definitions, references, rename, and document symbols |
| `protocol` | LSP message types |
