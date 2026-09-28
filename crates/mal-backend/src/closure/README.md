# closure

Converts ANF lexical lambdas into a program-wide function table and explicit closure construction. Each function records
its capture schema; each `MakeClosure` operation supplies values in that schema, so later stages never infer captures
from free references.

| Module | Responsibility |
|---|---|
| `mod` | function lifting, capture-reference rewriting, entry-function selection, and fresh atom identities |
| `ast` | functions, closure construction, capture fields, references, blocks, and logical operations |

The stage does not choose direct calls, recursive regions, frames, or ownership. Those decisions require whole-program
flow and belong to `execution`.
