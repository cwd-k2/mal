# check/operation

Operation-family declarations and implementations, checked so that specialization can select one
implementation per concrete key and expand requirements in finitely many steps.

| Module | Responsibility |
|---|---|
| `mod` | family signatures, implementation keys and bodies, and the requirements an implementation leaves for specialization |
| `pattern` | parameter occurrence in key patterns, overlap between two keys, and the structural decrease of requirement keys |
