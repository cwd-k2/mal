# parser

Recursive-descent parser with a Pratt loop for expressions. It builds the source-oriented AST and never chooses
syntax from types or names.

| Module | Responsibility |
|---|---|
| `mod` | top-level items, declarations, and type syntax |
| `generic` | generic parameter and argument lists, and the `>>` token that closes two directly nested lists |
| `expression` | Pratt loop, prefix dispatch, and operator precedence |
| `expression/forms` | products, both application directions, receiver-first application, numeric conversion, and zero-continuation application |
| `expression/lambda` | parameters and lambda bodies |
| `expression/control` | `if`, `when`, direct blocks, and direct result blocks |

The language grammar is owned by [`docs/spec`](../../../../docs/spec/). The parser is the compiler's accepting
implementation; editor recovery grammar lives in [`editors/tree-sitter-mal`](../../../../editors/tree-sitter-mal/),
and formatting lives in [`crates/mal-fmt`](../../../mal-fmt/). A syntax change normally updates the relevant spec,
parser tests, tree-sitter grammar and corpus, formatter tests, and the verification commands in
[`docs/development/testing.md`](../../../../docs/development/testing.md).
