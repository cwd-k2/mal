# Fallible tree example

Unlike the mal-owned indexed structure in `packed-tree`, this example deliberately uses external
storage because allocation failure is recoverable and deterministically limited by an opaque host
allocator. A five-node limit exercises
the successful path. A three-node limit fails after a complete left subtree has been built.

The `Tree` alias names an external-storage capability without claiming ownership of its referent.
The mal program treats constructor arguments as logically owned. `createOwnedBranch` either transfers
both children into a new parent or recursively destroys both after allocation failure. Higher
construction layers likewise destroy every completed subtree before propagating the error result.
Traversal borrows each node through short `Region` callbacks. Child `Address` values are ordinary
capabilities, so they leave the callback as a product without allocating a `Packed<Address>` snapshot.

The host tracks every live node and traps if `destroyAllocator` is called before all nodes have been
released. This makes partial-construction leaks observable in the end-to-end test. The protocol is
not enforced by mal's types: `Allocator` and `Address` remain copyable, and the variant-0/variant-1
convention and ownership transfer must be followed manually.

From the repository root in Nushell:

```nu
nix develop --command cargo run --manifest-path compiler/Cargo.toml -- build examples/fallible-tree/program.mal --output /tmp/mal-fallible-tree
/tmp/mal-fallible-tree
```

The executable produces no output and exits with status 0 after both scenarios pass.
