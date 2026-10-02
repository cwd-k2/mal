# mal-fmt

The mal formatter, as a library (`format`) and as the `mal-fmt` command.

```nu
mal-fmt program.mal           # print canonical source
mal-fmt --write program.mal   # replace the file atomically
```

The layout rules are documented in `docs/development/formatting.md`. The crate depends only on `mal-syntax`.

| Module | Responsibility |
|---|---|
| `layout` | which blocks stay compact |
| `control` | where control expressions start and end, and which of them expand |
| `token` | spacing and line breaks between tokens |
| `token/control` | output state for `if` and block delimiters |
| `token/breaks` | line breaks the source wrote inside expressions |
| `generic` | the angle brackets that delimit generic lists, so they are not spaced as operators |
| `file` | reading a file and replacing it atomically |
| `cli` | arguments and exit status |
