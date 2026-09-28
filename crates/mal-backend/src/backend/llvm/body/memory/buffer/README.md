# LLVM Buffer lowering

This directory lowers core `BufferOperation` values. The C runtime owns capacity changes and byte storage; LLVM
owns element representation, reference-counting callbacks, and typed loads and stores.

`ElementStorage` is the central classification. `Canonical` elements use the source-memory layout and can be
copied through the host-memory primitives. Managed elements without a canonical representation use their LLVM
runtime layout and the retain/release callbacks collected by `ManagedBufferElements`. An element must fit one of
those categories before emission.

The implementation is split as follows:

| File | Responsibility |
|---|---|
| `mod.rs` | operation dispatch, element-storage classification, runtime calls, and direct element access |
| `address.rs` | snapshot conversion between `Buffer<UInt8>` and `Symbol` |
| `managed.rs` | one numbered retain/release callback pair per managed element type |

`new` and `fill` pass an element through the entry-block `%mal_buffer_value` scratch allocation sized in
`body/setup.rs`. Runtime function names selected by `ElementStorage::runtime` must remain synchronized with
`runtime/c11/buffer.c`, `runtime/c11/buffer_range.c`, and their declarations in `runtime/c11/runtime.h`. Changes to
the buffer object or byte-owner layout must also preserve the contracts documented by `runtime/c11/README.md` and
the generated declarations in the LLVM module setup.
