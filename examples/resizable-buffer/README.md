# Resizable Buffer

This example demonstrates that `Buffer<UInt8>` is an ordinary managed value with automatic growth.
Copying it creates an alias to the same mutable buffer, and no explicit release operation is needed.

The C host owns only a small transfer area. `buffer.into(address, offset, length)` copies a selected
range into that host area before an extern writes it. The host never receives or owns the Buffer.

Run it from the repository root:

```console
malc run examples/resizable-buffer/program.mal
```

Expected output:

```text
al
mal-shared-buffer
```
