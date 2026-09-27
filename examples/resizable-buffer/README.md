# Resizable Buffer

This example demonstrates that `Buffer<UInt8>` is an ordinary managed value with automatic growth.
Copying it creates an alias to the same mutable buffer, and no explicit release operation is needed.

The C host owns only a small transfer area. `buffer.into(address, offset, length)` copies a selected
range into that host area before an extern writes it. The host never receives or owns the Buffer.

From the repository root in Nushell:

```nu
nix develop --command cargo run -p mal-compiler -- build examples/resizable-buffer/program.mal --output /tmp/mal-resizable-buffer
/tmp/mal-resizable-buffer
```

Expected output:

```text
al
mal-shared-buffer
```
