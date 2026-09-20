# Relation views example

This example builds one immutable `Packed<PairRow>` carrier and interprets it through two unrelated
logical structures. `hasDirectedEdge` reads each pair as an ordered graph edge. `coverageAt` reads the
same pair as a half-open interval and counts the intervals containing a coordinate.

`PairRows` describes only the finite physical row sequence. It does not decide whether a pair is an
edge or an interval, and its transparent alias proves neither graph nor interval invariants. Each
operation owns its interpretation and preconditions. In particular, `validIntervals` is relevant to
the interval view but not to the edge view.

Under the graph interpretation, both fields are node coordinates and the ordered pair is a stored
relation. Under the interval interpretation, they are boundary coordinates and the half-open
membership operation derives a containment relation. The representation is identical; the field
roles and valid invariants are not.

This deliberately small example makes the separation observable: row `(3, 5)` is the directed edge
from 3 to 5, while the interval view says that it contains 3 and 4 but not 5. No carrier conversion or
copy is needed to select an interpretation.

Traversal is separated from those interpretations as `_anyPacked`, `_allPacked`, and `_foldPacked`.
The graph view supplies an existential predicate, interval validation supplies a universal predicate,
and coverage supplies an accumulator step. These names expose short-circuit and empty-input policy at
the call site without making each domain operation manage a cursor.

From the repository root in Nushell:

```nu
nix develop --command cargo run --manifest-path compiler/Cargo.toml -- build examples/relation-views/program.mal --output /tmp/mal-relation-views
/tmp/mal-relation-views
```

The executable produces no output and exits with status 0 after all three interpretations agree with
the expected rows.
