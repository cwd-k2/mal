# parser

Recursive-descent parser with a Pratt loop for expressions. It builds the source-oriented AST and never chooses
syntax from types or names.

| Module | Responsibility |
|---|---|
| `mod` | the entry points, programs, and top-level items |
| `types` | type syntax |
| `binding` | bindings, their patterns, and the bodies they initialize with |
| `cursor` | the token cursor, diagnostics, and the bound on syntax nesting |
| `generic` | generic parameter and argument lists |
| `expression` | operator precedence and prefix dispatch |
| `expression/forms` | products and the application forms |
| `expression/lambda` | lambdas |
| `expression/control` | `if`, `when`, and direct blocks |

The language grammar is owned by [`docs/spec`](../../../../docs/spec/). The parser is the compiler's accepting
implementation; editor recovery grammar lives in [`editors/tree-sitter-mal`](../../../../editors/tree-sitter-mal/),
and formatting lives in [`crates/mal-fmt`](../../../mal-fmt/). A syntax change normally updates the relevant spec,
parser tests, tree-sitter grammar and corpus, formatter tests, and the verification commands in
[`docs/development/testing.md`](../../../../docs/development/testing.md).
