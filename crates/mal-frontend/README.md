# mal-frontend

Name resolution, type checking, and specialization (`resolve`, `check`), the editor semantic index (`editor`), and
the analysis entry points that run them in order (`analysis`). It stops at a checked program, so the language server
uses it without the backend. Module responsibilities for `check` are in `src/check/README.md`.
