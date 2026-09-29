# check/inference

Local inference for generic value references and calls. Type constructors remain rigid: callers provide
constructor arguments explicitly, while ordinary `Type` arguments may be inferred from expected and operand types.

| Module | Responsibility |
|---|---|
| `mod` | generic-reference admission, explicit prefixes, call checking, and finalized specialization keys |
| `arguments` | argument-template selection, flexible parameter sets, and complete inferred argument lists |
| `probe` | non-committing operand and direct lambda-result probes against partially instantiated templates |
| `constraint` | structural constraints, substitution resolution, and conflicts for ordinary type parameters |
| `constraint/scheme` | unification between nested generic schemes without leaking inner flexible parameters |
