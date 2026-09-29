# Indexed graph

This example represents a weighted directed graph as CSR columns. Opaque column roles prevent
accidental interchange, while `validWeightedCsr` establishes offset monotonicity, joined column
lengths, destination bounds, and nonnegative weights before traversal. Indices acquire graph meaning
only through these operations and invariants.

> [!NOTE]
> A Buffer index is only a number. `validWeightedCsr` and the opaque column operations are what make
> particular numbers node coordinates, edge coordinates, and range boundaries.

Dijkstra's workspace uses one `Buffer<Int64>` viewed as distance and visited columns joined by node
coordinate. The O(V²) selection policy keeps the example focused on indexed representation rather
than introducing a second data structure.

```nu
nix develop --command cargo run -p mal-compiler -- build examples/indexed-graph/program.mal --output /tmp/mal-indexed-graph
/tmp/mal-indexed-graph
```

Success is silent after rejecting an invalid graph and finding distance 7 from node 0 to node 4.
