# call_pattern

Call-pattern specialization, the one optimization that rewrites the closure program. A top-level function that
receives closures is copied for each distinct set of closures its call sites pass, so the flow of the rewritten program
names one closure at each call the copy makes. It runs when `Technique::CallPattern` is enabled, before the program is
lowered to control, and stops at a fixed point or when the program has grown past its budget.

| Module | Responsibility |
|---|---|
| `mod` | the round loop and the growth budget |
| `plan` | the call sites that should call a copy, from the closure flow of each argument |
| `rewrite` | copies the requested functions, inserts their top-level bindings, and redirects the call sites |
| `clone` | copies of a function and the closures it creates, and of a top-level binding, under fresh identities |
| `ids` | fresh identities and the check that every binder and atom identity is unique |
| `walk` | a mutable walk over the closure program that reports the identities a rewrite touches |
