# parser

Recursive-descent parser with a Pratt loop for expressions. It builds the source-oriented AST and never chooses
syntax from types or names.

| Module | Responsibility |
|---|---|
| `mod` | top-level items, declarations, and type syntax |
| `expression` | Pratt loop, prefix dispatch, and operator precedence |
| `expression/forms` | products, both application directions, receiver-first application, numeric conversion, and zero-continuation application |
| `expression/lambda` | parameters and lambda bodies |
| `expression/control` | `if`, `when`, direct blocks, and direct result blocks |
