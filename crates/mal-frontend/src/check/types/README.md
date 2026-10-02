# check/types

Construction and validation of canonical type-level terms. The checker is the only stage that admits
source type expressions and infers kinds; specialization consumes these terms and emits runtime types.

| Module | Responsibility |
|---|---|
| `kind` | principal kind inference for declarations and value signatures |
| `definitions` | the source declarations type checking reads |
| `validation` | declaration-wide validation after kind inference |
| `expand` | expansion of source type expressions into canonical terms, as a stack machine |
| `expand/declaration` | type names resolved to the declarations they stand for, and the cache of finished expansions |
| [`term`](term/README.md) | canonical type-level terms |
| `canonical` | erasure of canonical terms to runtime types |
| `canonical/substitution` | substitution of type parameters |
| `canonical/opaque_view` | file-local views of opaque types, and the type equivalence they give |
| `properties` | `Storable` and host-memory properties |
| `representation` | bounded physical representation measurement |
| `display` | bounded diagnostic names for canonical types |
