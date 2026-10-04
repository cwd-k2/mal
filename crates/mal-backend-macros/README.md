# mal-backend-macros

This crate parses the restricted Rust-shaped syntax used to construct generated C and LLVM IR. The
crate root only exposes procedural entry points; `c` and `llvm` own their respective parsers, while
`shared` contains token-cursor and diagnostic mechanics. Both grammars use `{ expression }` for one
Rust value and `..{ iterator }` for a sequence; no legacy interpolation normalization remains.

The C parser also owns translation-unit and record fragments. `c_items!` covers includes, ordinary
defines, comments, static assertions, aliases, structs/unions, and function declarations or
definitions. `c_record!` and `c_record_fields!` expose the same record grammar where a typed
preprocessor replacement needs a fragment instead of a complete translation unit. `c_items!`
also admits `if defined(...)` and `if !defined(...)` item groups. `c_invocation!` constructs macro
invocations, while `c_initializers!` covers positional, designated, and nested-designated fields.

Expansions call typed nodes owned by `mal-backend`. Rust does not allow a `proc-macro` crate to
export ordinary syntax types, and moving them to another shared crate would expose backend-internal
representation without adding a second consumer. Semantic validation, stateful builders, and
rendering therefore remain in `mal-backend`.
