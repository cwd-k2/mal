# check/inference

Local inference for generic value references and calls. Type constructors remain rigid: callers provide
constructor arguments explicitly, while ordinary `Type` arguments, and kind-polymorphic phantom arguments met by
a non-constructor type, may be inferred from expected and operand types.

| Module | Responsibility |
|---|---|
| `mod` | generic-reference admission, explicit prefixes, call checking, and finalized specialization keys |
| `arguments` | argument-template selection, flexible parameter sets, and complete kind-checked inferred argument lists |
| `probe` | operand and direct lambda-result probes against partially instantiated templates |
| `memo` | probe transactions that roll back requirement and result-target effects, and the per-call memo through which later probes and the final argument check reuse a probe outcome, failures included |
| `constraint` | structural constraints, substitution resolution, and conflicts for ordinary type parameters |
| `constraint/scheme` | unification between nested generic schemes without leaking inner flexible parameters |
