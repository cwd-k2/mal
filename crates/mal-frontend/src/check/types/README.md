# check/types

Construction and validation of canonical type-level terms. The checker is the only stage that admits
source type expressions and infers kinds; specialization consumes these terms and emits runtime types.

| Module | Responsibility |
|---|---|
| `kind` | principal kind inference for declarations and value signatures, including dependency components and occurs checking |
| `definitions`, `validation` | source declaration collection and declaration-wide validation after kind inference |
| `expand` | iterative source expansion, alias-cycle rejection, partial application, and opaque constructor formation |
| `term` | budgeted kind-safe application, beta reduction, eta reduction, and de Bruijn substitution |
| `canonical` | generic substitution, runtime erasure, and file-local opaque equivalence |
| `properties`, `representation` | `Storable` and host-memory properties, plus bounded physical representation measurement |
| `display` | bounded diagnostic names for canonical types |
