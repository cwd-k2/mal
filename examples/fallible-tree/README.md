# Fallible tree example

This example builds the same pointer-based binary tree as `pointer-tree`, but allocation is
recoverable and deterministically limited by an opaque host allocator. A five-node limit exercises
the successful path. A three-node limit fails after a complete left subtree has been built.

The mal program treats constructor arguments as logically owned. `createOwnedBranch` either transfers
both children into a new parent or recursively destroys both after allocation failure. Higher
construction layers likewise destroy every completed subtree before propagating the error return binder.

The host tracks every live node and traps if `destroyAllocator` is called before all nodes have been
released. This makes partial-construction leaks observable in the end-to-end test. The protocol is
not enforced by mal's types: `Allocator` and `Ptr` remain copyable, and the variant-0/variant-1
convention and ownership transfer must be followed manually.

From the repository root in Nushell:

```nu
nix develop --command cargo run --manifest-path compiler/Cargo.toml -- build examples/fallible-tree/program.mal --output /tmp/mal-fallible-tree
/tmp/mal-fallible-tree
```

The executable produces no output and exits with status 0 after both scenarios pass.
