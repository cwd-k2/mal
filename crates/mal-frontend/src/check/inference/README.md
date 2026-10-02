# check/inference

Local inference for generic value references and calls. Type constructors remain rigid: callers provide
constructor arguments explicitly, while ordinary `Type` arguments, and kind-polymorphic phantom arguments met by
a non-constructor type, may be inferred from expected and operand types.

| Module | Responsibility |
|---|---|
| `mod` | the type arguments of one generic reference, inferred or explicit |
| `call` | the type arguments of one generic call, inferred or explicit |
| `reference` | a generic reference whose type arguments are known, and the requirements it passes to the enclosing body |
| `arguments` | the argument lists inference starts from and finishes with |
| `probe` | probes of operands and direct lambda results against a partially instantiated signature |
| `memo` | probes as transactions, remembered per call so a later probe reuses their outcome |
| `constraint` | structural constraints over ordinary type parameters |
| `constraint/scheme` | unification between nested generic schemes |
