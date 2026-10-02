# editor

Builds what editor queries need from the current text and, when analysis succeeds, from resolved and checked programs.

| Module | Responsibility |
|---|---|
| `syntax` | what editors can know from tokens alone, even when parsing or checking fails |
| `syntax/declaration` | conservative recognition of top-level function declarations from tokens |
| `index` | the declaration identity index that editor queries read |
| `index/resolved_ast` | identities and declared types from the resolved program |
| `index/resolved_ast/top` | identities introduced by top-level declarations |
| `index/checked_ast` | checked types and the result binders control leaves to |
| `index/aliases` | value identity aliases introduced by lambda captures |
| `index/type_display` | source type names as written, so hover keeps aliases |
| `index/predefined` | type details of predefined values |
| `documentation` | the comment lines directly above a declaration |

The resolved and checked walks iterate over left-associative operator chains instead of recursing.
