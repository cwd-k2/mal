# resolve

Gives every name an identity and infers lexical captures. Later stages read these identities and never re-resolve names.

| Module | Responsibility |
|---|---|
| `mod` | the resolver state and the order of declarations within a file |
| `top` | one top-level item |
| `types` | type expressions and the type parameters a declaration brings into scope |
| `files` | the names each file introduces, and program item order |
| `scope` | declaration identity and name lookup |
| `key` | the type variables an implementation key introduces |
| `continuation` | the continuations of a continuation application |
| `expression` | expressions and the captures of lambdas |
| `predefined` | the single declaration of predefined names and reserved identities |
| `ast` | the resolved program representation |
