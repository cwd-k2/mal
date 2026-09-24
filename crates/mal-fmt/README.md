# mal-fmt

The mal formatter, as a library (`format`) and as the `mal-fmt` command.

```nu
mal-fmt program.mal           # print canonical source
mal-fmt --write program.mal   # replace the file atomically
```

The layout rules are documented in `docs/development/formatting.md`. The crate depends only on `mal-syntax`.
