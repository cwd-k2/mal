# CSR Dijkstra example

This example stores a directed weighted graph in three immutable columns. `NodeOffsets` partitions
edge slots into one range per source node, `NeighborNodes` maps each edge slot to a destination node,
and `EdgeCosts` supplies payload joined by that same edge slot. None of the three carriers is a graph
by itself. `WeightedCsr` and its operations define how their coordinates relate.

`validWeightedCsr` is the admission boundary. It checks the offset extent and monotonicity, the shared
edge-column length, every destination coordinate, and Dijkstra's nonnegative-cost precondition before
traversal. `shortestDistance` additionally expects in-bounds source and target coordinates and path
costs below the example's `infinity` sentinel. The transparent aliases make roles visible to readers
but do not prove these invariants.

Dijkstra's mutable workspace occupies one backing `Buffer<Int64>`, split into `NodeDistances` and
`VisitedFlags`. These are payload columns joined by node coordinate; neither is a relation indicator.
The example deliberately selects the next node with an O(V²) scan so the CSR relation remains the
only data-structure concern. A binary heap could replace that selection policy, but its parent/child
relation would be another algorithmic interpretation of indices rather than a property of CSR.

From the repository root in Nushell:

```nu
nix develop --command cargo run --manifest-path compiler/Cargo.toml -- build examples/csr-dijkstra/program.mal --output /tmp/mal-csr-dijkstra
/tmp/mal-csr-dijkstra
```

The executable produces no output and exits with status 0 after validating the graph and checking
that the shortest distance from node 0 to node 4 is 7.
