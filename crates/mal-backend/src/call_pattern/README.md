# call_pattern

Call-pattern specialization, the one optimization that rewrites the closure program. A top-level function that
receives closures is copied for each distinct set of closures its call sites pass, so the flow of the rewritten program
names one closure at each call the copy makes. The copying stops at a fixed point or when the program has grown past its
budget; parameter lift and lambda lift then turn the captures of closures that reach only known calls into parameters.
The stage runs when `Technique::CallPattern` is enabled, before the program is lowered to control.

| Module | Responsibility |
|---|---|
| `mod` | the round loop and the growth budget |
| `plan` | the call sites that should call a copy, from the closure flow of each argument |
| `rewrite` | copies the requested functions, inserts their top-level bindings, and redirects the call sites |
| `clone` | copies of a function and the closures it creates, and of a top-level binding, under fresh identities |
| `ids` | fresh identities and the check that every binder and atom identity is unique |
| `lambda_lift`, `lambda_lift/operation` | capture-to-parameter rewriting for a non-recursive local closure whose aliases are used only as direct callees |
| `parameter_lift` | candidate admission, the footprint each candidate reads and rewrites, and the round of candidates with disjoint footprints |
| `parameter_lift/analysis` | callback uses, forwarding edges, and nested directly called captures admitted for propagation |
| `parameter_lift/analysis/index` | the whole-program facts admission reads (binding types, host calls, uses by alias origin, closure creators), collected in one walk per round |
| `parameter_lift/analysis/aliases` | cycle-safe callback alias origins with path compression |
| `parameter_lift/analysis/walk` | immutable traversal of closure blocks and operation atoms for parameter-lift analysis |
| `parameter_lift/rewrite` | capture propagation through each admitted function parameter and its self-recursive forwarding edges, for a round of candidates with disjoint footprints in one walk |
| `parameter_lift/nested` | analysis and rewriting for a callback used only as a directly called capture of another closure |
| `walk` | a mutable walk over the closure program that reports the identities a rewrite touches |
