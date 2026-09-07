# Pointer tree example

This example builds an immutable binary tree encoding in external storage. The source-level tree
value is a `Ptr` capability, not a product or sum with a canonical memory representation. The mal
program defines the node layout, writes and reads child pointers with `storePtr` and `loadPtr`,
recursively sums the values, and releases every node. The host adapter owns only allocation and
deallocation; the mal program obtains the target ABI's pointer storage width with `@Ptr`.

From the repository root in Nushell:

```nu
nix develop --command cargo run --manifest-path compiler/Cargo.toml -- build examples/pointer-tree/program.mal --output /tmp/mal-pointer-tree
/tmp/mal-pointer-tree
```
