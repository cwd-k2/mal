# check/operation

Operation-family declarations and implementations, checked so that specialization can select one
implementation per concrete key and expand requirements in finitely many steps.

| Module | Responsibility |
|---|---|
| `mod` | family signatures, implementation bodies, and the requirements an implementation leaves for specialization |
| `key` | the shape and coherence of an implementation key, the decrease of the requirements its body leaves, and the type suggestions added to its failures |
| `pattern` | parameter occurrence in key patterns, nominal heads of constructor keys, and the structural decrease of requirement keys |
| `overlap` | overlap between two keys, by unifying their patterns with each side's variables kept apart |
