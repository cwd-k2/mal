# editor

Builds what editor queries need from the current text and, when analysis succeeds, from resolved and checked programs.

| Module | Responsibility |
|---|---|
| `syntax` | token classification, top-level function candidates, and the first complete `require` path, available even when parsing or checking fails |
| `syntax/declaration` | conservative classification of top-level function declarations from tokens |
| `index` | declaration identity index behind document symbols, completion, and file-local views |
| `index/resolved_ast` | declaration and reference identities, explicit alias names, type references and declared types propagated through binding patterns, and result binders |
| `index/checked_ast` | canonical types of checked expressions and result binders, and where control leaves its result block and to which binders |
| `index/aliases` | value identity aliases introduced by lambda captures |
| `index/type_display` | declared and contextually propagated source type names, so hover keeps aliases |
| `index/predefined` | type details of predefined values from the resolver's predefined table |
| `documentation` | the contiguous `//` comment lines directly above a declaration |

The resolved and checked walks iterate over left-associative operator chains instead of recursing.
