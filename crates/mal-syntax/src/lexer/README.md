# lexer

Admits UTF-8 source text into parser tokens. `lex` discards trivia; `lex_lossless` also records whitespace and comments
for the formatter. Both paths share token recognition and diagnostics, so formatting cannot accept a different lexical
language from parsing.

| Module | Responsibility |
|---|---|
| `mod` | the lexer |
| `token` | tokens and their lossless lexemes |
| `number` | numeric literal boundaries and components |
| `escape` | byte escapes |
| `symbol` | Symbol literals |

The lexer classifies syntax only from bytes. Name identity, expected types, literal ranges, and numeric values belong to
later stages.
