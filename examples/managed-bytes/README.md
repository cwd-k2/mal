# Managed bytes

The host initializes a bounded transfer area, and `from<UInt8>` admits that initialized prefix into
a mal-owned `Buffer`. Assignment creates an alias to the same growable mutable identity. Converting
the Buffer to `Symbol` creates an immutable snapshot, so later mutation and append through the alias
do not alter the earlier text value. `buffer.into` copies the final range back to external storage;
no managed identity crosses the host boundary.

> [!NOTE]
> Assigning a Buffer copies its managed identity, while converting it to Symbol copies its current
> bytes. Those two operations deliberately have different aliasing behavior.

```nu
nix develop --command cargo run -p mal-compiler -- build examples/managed-bytes/program.mal --output /tmp/mal-managed-bytes
/tmp/mal-managed-bytes
```

The host validates the bytes and prints `9 bytes`.
