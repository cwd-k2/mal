# closure

Converts ANF lexical lambdas into a program-wide function table and explicit closure construction. Each function records
its capture schema; each `MakeClosure` operation supplies values in that schema, so later stages never infer captures
from free references.

| Module | Responsibility |
|---|---|
| `mod` | conversion of lambdas into lifted functions and closure construction |
| `ast` | the closure-converted program and direct operation-operand traversal |

The stage does not choose direct calls, recursive regions, frames, or ownership. Those decisions require whole-program
flow and belong to `execution`.
