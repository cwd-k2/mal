# Buffer tree example

This example uses `Buffer<NodeRow>` as a finite shared mutable carrier for a binary tree. The carrier
alone is not a tree: `buildTreeRows` establishes that coordinate 0 is the root and that branch fields
form bounded, acyclic left and right relations. `sumTreeAt` interprets those carrier-relative
coordinates while traversing the logical tree.

Within each row, `value` is payload, `kind` selects the row interpretation, and the left and right
coordinates are relation indicators only when `kind` denotes a branch. Physical row order makes the
coordinates cheap to store; adjacency in that order does not itself mean parenthood.

`buildTreeRows` uses `make<NodeRow>` to construct the rows through an ordinary `Buffer<NodeRow>`. It
appends parent slots before their child coordinates are known, then fills the relation with `put`.
`incrementRoot` uses mutation through an alias; both names observe the updated root.
Because it neither resizes nor reorders rows, existing coordinates keep their meaning. A compaction or
sort would instead need to remap every child coordinate.

Use external storage instead when a data structure genuinely belongs to a host resource or needs
recoverable allocation failure. The neighboring `fallible-tree` example deliberately keeps that
different authority model.

From the repository root in Nushell:

```nu
nix develop --command cargo run -p mal-compiler -- build examples/buffer-tree/program.mal --output /tmp/mal-buffer-tree
/tmp/mal-buffer-tree
```

The executable produces no output and exits with status 0 after both aliases
traverse to their expected totals.
