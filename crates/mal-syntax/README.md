# mal-syntax

Everything needed to read mal source: file identity and spans (`source`), diagnostics, the lexer, the AST, the parser,
and loading a source graph from disk (`graph`, `requirement`). It has no dependencies and no knowledge of types,
so the formatter, the language server, and the compiler all build on it.

| Module | Responsibility |
|---|---|
| `source` | admitted UTF-8 files, stable file identities, byte spans, line locations, and UTF-16 editor positions |
| `diagnostic` | structured source diagnostics and rendering against a source provider |
| [`lexer`](src/lexer/) | parser tokens and the lossless trivia stream used by the formatter |
| `ast` | source-oriented declarations, types, patterns, expressions, and literal spelling |
| [`parser`](src/parser/) | syntax admission from tokens to the AST |
| `requirement` | requirement-path decoding, validation, and completion candidates |
| `graph` | recursive source loading, open-document overlays, canonical file identity, and cycle detection |
