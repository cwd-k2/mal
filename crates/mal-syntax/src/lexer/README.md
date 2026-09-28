# lexer

Admits UTF-8 source text into parser tokens. `lex` discards trivia; `lex_lossless` also records whitespace and comments
for the formatter. Both paths share token recognition and diagnostics, so formatting cannot accept a different lexical
language from parsing.

| Module | Responsibility |
|---|---|
| `mod` | source traversal, trivia handling, identifier and punctuation recognition, and public lexing entry points |
| `token` | token, literal, suffix, radix, and lossless lexeme representations |
| `number` | integer and decimal-float token boundaries and structured literal components |
| `escape` | byte escape decoding and the exact error offset |
| `symbol` | Symbol literal decoding over the shared escape rules |

The lexer classifies syntax only from bytes. Name identity, expected types, literal ranges, and numeric values belong to
later stages.
