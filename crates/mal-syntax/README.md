# mal-syntax

Everything needed to read mal source: file identity and spans (`source`), diagnostics, the lexer, the AST, the parser,
and loading a source graph from disk (`graph`, `requirement`). It has no dependencies and no knowledge of types,
so the formatter, the language server, and the compiler all build on it.

| Module | Responsibility |
|---|---|
| `source` | admitted UTF-8 files, their identities, spans, and positions |
| `diagnostic` | structured diagnostics and their rendering |
| [`lexer`](src/lexer/) | tokens and the lossless trivia stream |
| `ast` | the source-oriented syntax tree |
| [`parser`](src/parser/) | syntax admission from tokens to the AST |
| `requirement` | requirement paths |
| `graph` | loading a program's source files |
