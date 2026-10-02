# check/operation

Operation-family declarations and implementations, checked so that specialization can select one
implementation per concrete key and expand requirements in finitely many steps.

| Module | Responsibility |
|---|---|
| `mod` | family signatures and implementation bodies |
| `key` | the keys specialization can select from without ambiguity |
| `termination` | the structural decrease that makes requirement expansion finite |
| `pattern` | where parameters occur in a key, and which constructor heads are nominal |
| `overlap` | whether two keys can match one concrete argument list |
| `spelling` | the types a misspelled key binder may have meant |
