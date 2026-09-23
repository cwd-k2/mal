# Fallible tree example

Unlike the mal-owned indexed structure in `buffer-tree`, this example deliberately uses external
storage because allocation failure is recoverable and deterministically limited by an opaque host
allocator. A five-node limit exercises
the successful path. A three-node limit fails after a complete left subtree has been built.

`NodeAddress` names only an external-storage coordinate. It does not claim ownership of its referent or
prove that the pointed-to record is initialized, has a known tag, or belongs to a tree. A leaf record
stores `(value, 0)`; a branch record stores `(value, 1, leftAddress, rightAddress)`. The child addresses
are relation indicators, while the value and tag are payload and interpretation metadata. Construction
operations establish this layout and relation as a host-backed protocol.
The mal program treats constructor arguments as logically owned. `createOwnedBranch` either transfers
both children into a new parent or recursively destroys both after allocation failure. Higher
construction layers likewise destroy every completed subtree before propagating the error result.
`NodeBuildResult` reports either a root coordinate or allocation failure; the alias itself does not
certify the complete reachable structure. Traversal copies one `NodeRecord` at a time with
`from<NodeRecord>`. Child `Address` values remain ordinary capability fields; copying them does not
extend the lifetime of their referents.

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
