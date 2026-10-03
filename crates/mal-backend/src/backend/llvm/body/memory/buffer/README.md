# LLVM Buffer lowering

This directory lowers core `BufferOperation` values. The C runtime owns capacity changes and the choice between small
inline storage and a flat byte owner; LLVM
owns element representation, reference-counting callbacks, and typed loads and stores.

`ElementStorage` is the central classification. `Canonical` elements use the source-memory layout and can be
copied through the host-memory primitives. `RuntimeOwned` elements use their LLVM runtime layout and the
retain/release callbacks collected by `RuntimeOwnedBufferElements`; this includes `Symbol` and nested `Buffer`
carriers.
An element must fit one of those categories before emission. Whether a source type is `Storable` remains a frontend
admission rule and is not inferred from this representation choice.

The implementation is split as follows:

| File | Responsibility |
|---|---|
| `mod.rs` | operation dispatch and the classification of element storage |
| `access.rs` | direct element access |
| `address.rs` | `from` and `into` copies between a `Buffer` and C host storage |
| `runtime_owned.rs` | one retain and release callback pair per runtime-owned element type |

`new` and `put` receive an owned element responsibility selected by the execution plan. `new` transfers it through the
entry-block `%mal_buffer_value` scratch allocation, while `put` moves it directly into the selected place. `fill`
borrows one value and lets the runtime share it for every written element. Runtime function names selected by
`ElementStorage::runtime` must remain synchronized with
`runtime/c11/buffer.c`, `runtime/c11/buffer_range.c`, and their declarations in `runtime/c11/runtime.h`. Changes to
the buffer object or byte-owner layout must also preserve the contracts documented by `runtime/c11/README.md` and
the generated declarations in the LLVM module setup. The C ABI suffix `_managed` names its callback-bearing variant;
it is not the compiler's storage classification and does not imply that every `RuntimeOwned` element needs callbacks.

Canonical and runtime-owned element loads and stores carry the same Buffer-element TBAA and the same no-alias relation
to Buffer object metadata. This lets LLVM keep a stable data view across an inlined retain without claiming that two
arbitrary Buffer elements do not alias. The optional backend optimization separately removes a `get`, move-only aliases,
and a `put` when they return one managed responsibility to the exact same Buffer and coordinate with no intervening
operation.
