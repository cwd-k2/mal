# Resizable buffer example

This example combines an opaque C-owned `Allocation` with mal-visible `Buffer` and `Slice`
descriptors. The mal program calculates growth, appends Symbol bytes, checks slice bounds, propagates
recoverable allocation failures, and releases the allocation along every result path.

The host implements resize as allocate-copy-free rather than `realloc`. A successful resize always
moves storage and invalidates every earlier `Buffer` and `Slice`; a failed resize leaves the original
allocation unchanged. The program verifies both cases with host operations that inspect descriptors
without dereferencing their data pointers.

The program also stores a slice descriptor in shared memory. Product values have no canonical memory
representation, so mal writes the `Ptr` and `UInt64` fields separately and C reconstructs them with
the corresponding scalar representations. The predefined memory operations cannot store the opaque
`Allocation`; this example keeps its authority in C and passes the handle as a separate extern
argument. A host could instead define allocation-specific storage operations and their lifetime
contract. An out-of-bounds slice request exercises the checked failure variant without constructing
a pointer outside the allocation.

This is a logical ownership protocol rather than language-enforced safety. `Allocation`, `Buffer`, and
`Slice` remain copyable. Old descriptors can still be passed around after resize, and using their
`Ptr` directly would violate the host contract. `releaseBuffer` must be called exactly once with the
current allocation handle.

From the repository root in Nushell:

```nu
nix develop --command cargo run --manifest-path compiler/Cargo.toml -- build examples/resizable-buffer/program.mal --output /tmp/mal-resizable-buffer --link examples/resizable-buffer/host.c
/tmp/mal-resizable-buffer
```

Expected output:

```text
al
mal-shared-buffer
resize rejected
```
