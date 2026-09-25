# editor

Builds what editor queries need from the current text and, when analysis succeeds, from resolved and checked programs.

| Module | Responsibility |
|---|---|
| `syntax` | token classification, top-level function candidates, and the first complete `require` path, available even when parsing or checking fails |
| `syntax/declaration` | conservative classification of top-level function declarations from tokens |
| `index` | declaration identity index behind document symbols, completion, and file-local views |
| `index/resolved_ast` | declaration and reference identities, explicit alias names, and result binders |
| `index/checked_ast` | canonical types of checked expressions and result binders, and where control leaves the enclosing block |
| `index/aliases` | alias names written in declarations, so hover keeps them |
| `index/predefined` | type details of predefined values from the resolver's predefined table |
| `index/type_display` | display names of source type expressions |
| `documentation` | the contiguous `//` comment lines directly above a declaration |

The resolved and checked walks iterate over left-associative operator chains instead of recursing.
