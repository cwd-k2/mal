# check/types/term

Canonical type-level terms. A term is canonical when it holds no beta-redex (an application whose constructor is an
abstraction) and no eta-redex (an abstraction whose body applies a term free of the bound variable to it). Type
equality is structural, so every term the checker compares or uses as a specialization key must be canonical.

Only two constructors can create or remove a redex: `apply`, which reduces when its constructor is an abstraction,
and `abstraction`, which contracts an eta-redex. Substitution can remove an occurrence of a bound variable and thereby
expose a redex in any enclosing node, so it rebuilds every node it visits through those constructors
(`rebuild_canonical`). Shifting and kind rewriting keep every occurrence and node shape, so they rebuild directly.
A phantom argument is applied with `apply_unused`, which drops the abstraction's binder without forming the argument.

| Module | Responsibility |
|---|---|
| `mod` | the canonical constructors, and the normalizer that carries their budget |
| `substitution` | substitution of a bound variable |
| `indices` | index shifting and occurrence tests |
| `normalization` | the budget of one normalization transaction |
| `kind` | kind unification and substitution |
| `kind_substitution` | rewriting the kinds recorded in a term |
| `kind_variables` | the kind variables a term contains, and their shared rewriting |
| `argument_kinds` | the kinds of type arguments, and the equations a body leaves for specialization to check |
