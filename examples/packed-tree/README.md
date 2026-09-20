# Packed tree example

This example stores an immutable binary tree as `Packed<TreeNode>`. Child links are stable `USize`
indices into the same packed value, so the structure needs no host allocator, raw `Address`, or
per-node lifetime protocol.

`buildTree` uses `make<TreeNode>` to construct the tree through a scoped `Buffer<TreeNode>`. It appends
parent slots before their child indices are known, then fills the links with `put`.
`incrementRoot` uses `Tree.edit<TreeNode>` to derive a new tree while the original remains unchanged.
Recursive traversal reads nodes through ordinary packed indexing.

Use external storage instead when a data structure genuinely belongs to a host resource or needs
recoverable allocation failure. The neighboring `fallible-tree` example deliberately keeps that
different authority model.

From the repository root in Nushell:

```nu
nix develop --command cargo run --manifest-path compiler/Cargo.toml -- build examples/packed-tree/program.mal --output /tmp/mal-packed-tree
/tmp/mal-packed-tree
```
