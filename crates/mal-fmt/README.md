# mal-fmt

The mal formatter, as a library (`format`) and as the `mal-fmt` command.

```nu
mal-fmt program.mal           # print canonical source
mal-fmt --write program.mal   # replace the file atomically
```

The layout rules are documented in `docs/development/formatting.md`. The crate depends only on `mal-syntax`.

| Module | Responsibility |
|---|---|
| `layout` | precomputed block compactness, always-expanded `when` bodies, and top-level groups |
| `control` | iterative pass over the AST that classifies block positions, `if` on right-hand sides, and where control expressions end |
| `token` | spacing between ordinary tokens, explicit line breaks inside expressions, and preserved blank lines between statements and top-level groups |
| `token/control` | output state transitions for `if` and block delimiters |
| `generic` | marks the `<` and `>` tokens that delimit generic parameter and argument lists so they are not spaced like operators |
| `file` | reading a file, atomic replacement, and error messages |
| `cli` | argument parsing and exit status |
