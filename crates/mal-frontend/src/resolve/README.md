# resolve

Gives every name an identity and infers lexical captures. Later stages read these identities and never re-resolve names.

| Module | Responsibility |
|---|---|
| `mod` | declaration order within a source file, resolved items, and lambda identity |
| `files` | public names introduced by required files, file-private names, and program item order |
| `scope` | declaration identity, name lookup, the scope stack, and duplicate detection |
| `expression` | expressions, iterative walking of left-associative operator chains, transitive captures, result authority, and rejection of captures from nested lambdas |
| `predefined` | the single declaration of predefined names and reserved identities that the resolver, checker, and editor index all read |
