# mal-syntax

Everything needed to read mal source: file identity and spans (`source`), diagnostics, the lexer, the AST, the parser,
and loading a source graph from disk (`graph`, `requirement`). It has no dependencies and no knowledge of types,
so the formatter, the language server, and the compiler all build on it.
