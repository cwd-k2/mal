# External tree example

This example builds an immutable binary tree encoding in external storage. Each source-level tree
value uses a `Tree` alias over an `Address` capability. The alias communicates domain intent without
claiming ownership or nominal safety. The mal program defines the node layout with closed layout shapes,
writes and reads child addresses through typed cursors, recursively sums the values, and releases
every node. The host adapter owns only allocation and deallocation.

From the repository root in Nushell:

```nu
nix develop --command cargo run --manifest-path compiler/Cargo.toml -- build examples/external-tree/program.mal --output /tmp/mal-external-tree
/tmp/mal-external-tree
```
