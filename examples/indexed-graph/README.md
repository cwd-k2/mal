# Indexed graph

This example represents a weighted directed graph as CSR columns. `graph.mal` owns the opaque graph,
validates offset monotonicity, joined column lengths, and destination bounds, then snapshots every
input column. Caller-held Buffer aliases therefore cannot invalidate an admitted graph. `USize`
coordinates and `UInt64` costs exclude negative carrier values before validation begins.

> [!NOTE]
> A Buffer index is only a number. Admission and graph operations are what make particular numbers
> node coordinates, edge coordinates, and range boundaries.

Dijkstra's distance and visited Buffers are separate payload columns joined by node coordinate. The
O(V²) selection policy keeps the example focused on indexed representation rather than introducing
a second data structure. `UInt64` maximum is reserved as the unreachable sentinel, so representable
path distances are smaller; addition is checked before relaxation instead of relying on wrapping.

```nu
nix develop --command cargo run -p mal-compiler -- build examples/indexed-graph/program.mal --output /tmp/mal-indexed-graph
/tmp/mal-indexed-graph
```

Success is silent after rejecting an invalid graph and finding distance 7 from node 0 to node 4.
