# LLVM Buffer lowering

This directory lowers core `BufferOperation` values. The C runtime owns capacity changes and byte storage; LLVM
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

`new` and `fill` pass an element through the entry-block `%mal_buffer_value` scratch allocation sized in
`body/setup/scratch.rs`. Runtime function names selected by `ElementStorage::runtime` must remain synchronized with
`runtime/c11/buffer.c`, `runtime/c11/buffer_range.c`, and their declarations in `runtime/c11/runtime.h`. Changes to
the buffer object or byte-owner layout must also preserve the contracts documented by `runtime/c11/README.md` and
the generated declarations in the LLVM module setup. The C ABI suffix `_managed` names its callback-bearing variant;
it is not the compiler's storage classification and does not imply that every `RuntimeOwned` element needs callbacks.
