# call_pattern

Call-pattern specialization, the one optimization that rewrites the closure program. A top-level function that
receives closures is copied for each distinct set of closures its call sites pass, so the flow of the rewritten program
names one closure at each call the copy makes. The copying stops at a fixed point or when the program has grown past its
budget; parameter lift and lambda lift then turn the captures of closures that reach only known calls into parameters.
The stage runs when `Technique::CallPattern` is enabled, before the program is lowered to control.

| Module | Responsibility |
|---|---|
| `mod` | the round loop and the growth budget |
| `plan` | the call sites that should call a copy |
| `rewrite` | copying the requested functions and redirecting their call sites |
| `clone` | copies of functions and top-level bindings under fresh identities |
| `ids` | fresh identities, and the check that every identity is unique |
| `lambda_lift` | turning the captures of a directly called local closure into parameters |
| `lambda_lift/uses`, `lambda_lift/lift` | where each local closure is created and how it is used, and the lifting rewrite |
| `parameter_lift` | the round of callback parameters lifted together |
| `parameter_lift/analysis` | the callback uses that admit a parameter for lifting |
| `parameter_lift/analysis/index` | the whole-program facts admission reads, collected once per round |
| `parameter_lift/analysis/aliases` | the origin of each callback alias |
| `parameter_lift/analysis/walk` | the read-only walk the analysis uses |
| `parameter_lift/rewrite` | capture propagation through the admitted parameters |
| `parameter_lift/rewrite/pattern` | the patterns that bind a lifted callback |
| `parameter_lift/nested` | callbacks used only as a directly called capture of another closure |
| `walk` | a mutable walk that reports the identities a rewrite touches |
