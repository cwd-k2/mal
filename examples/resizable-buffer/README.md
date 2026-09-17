# Resizable buffer example

This example combines an opaque C-owned `Allocation` with mal-visible `OwnedBuffer` and `BorrowedBytes`
descriptors. The mal program calculates growth, appends Symbol bytes, checks borrow bounds, propagates
recoverable allocation failures, and releases the allocation along every result path.

The host implements resize as allocate-copy-free rather than `realloc`. A successful resize always
moves storage and invalidates every earlier `OwnedBuffer` and `BorrowedBytes`; a failed resize leaves
the original allocation unchanged. The program verifies both cases with host operations that inspect
descriptors without dereferencing their `Address` values.

The program also stores an `(Address, USize)` borrowed-view descriptor through its canonical product memory
representation and lets C reconstruct that descriptor. The opaque `Allocation` is deliberately not
stored: its authority remains in C and is passed as a separate extern argument. An out-of-bounds
borrow request exercises the checked failure variant without constructing an address outside the
allocation.

This is a logical ownership protocol rather than language-enforced safety. `Allocation`, `OwnedBuffer`, and
`BorrowedBytes` remain copyable. Old descriptors can still be passed around after resize, and using their
`Address` directly would violate the host contract. `releaseBuffer` must be called exactly once with the
current allocation handle.

From the repository root in Nushell:

```nu
nix develop --command cargo run --manifest-path compiler/Cargo.toml -- build examples/resizable-buffer/program.mal --output /tmp/mal-resizable-buffer
/tmp/mal-resizable-buffer
```

Expected output:

```text
al
mal-shared-buffer
resize rejected
```
