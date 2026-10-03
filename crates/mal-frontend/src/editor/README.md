# editor

Builds what editor queries need from the current text and, when analysis succeeds, from resolved and checked programs.
After a top-level checking error, the semantic index combines all identities from the current resolved program with
types from only the successfully checked top-level items.

| Module | Responsibility |
|---|---|
| `syntax` | what editors can know from tokens alone, even when parsing or checking fails |
| `syntax/declaration` | conservative recognition of top-level function declarations from tokens |
| `index` | the declaration identity index that editor queries read |
| `index/resolved_ast` | identities and declared types from the resolved program |
| `index/resolved_ast/top` | identities introduced by top-level declarations |
| `index/checked_ast` | checked types of expressions and result binders |
| `index/exits` | where control leaves a result block, and the binders it reaches |
| `index/expression_display` | display types of expressions as the source spells them |
| `index/aliases` | value identity aliases introduced by lambda captures |
| `index/type_display` | source type names as written, so hover keeps aliases |
| `index/predefined` | type details of predefined values |
| `documentation` | the comment lines directly above a declaration |

The resolved and checked walks iterate over left-associative operator chains instead of recursing.
