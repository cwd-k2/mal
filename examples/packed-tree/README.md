# Packed tree example

This example uses `Packed<NodeRow>` as the finite carrier for an immutable binary tree. The carrier
alone is not a tree: `buildTreeRows` establishes that coordinate 0 is the root and that branch fields
form bounded, acyclic left and right relations. `sumTreeAt` interprets those carrier-relative
coordinates while traversing the logical tree.

`buildTreeRows` uses `make<NodeRow>` to construct the rows through a scoped `Buffer<NodeRow>`. It
appends parent slots before their child coordinates are known, then fills the relation with `put`.
`incrementRoot` uses `edit<NodeRow>` to derive a new carrier while the original remains unchanged.
Because it neither resizes nor reorders rows, existing coordinates keep their meaning. A compaction or
sort would instead need to remap every child coordinate.

Use external storage instead when a data structure genuinely belongs to a host resource or needs
recoverable allocation failure. The neighboring `fallible-tree` example deliberately keeps that
different authority model.

From the repository root in Nushell:

```nu
nix develop --command cargo run --manifest-path compiler/Cargo.toml -- build examples/packed-tree/program.mal --output /tmp/mal-packed-tree
/tmp/mal-packed-tree
```
